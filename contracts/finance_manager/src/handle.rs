use cosmwasm_std;
use cosmwasm_std::{
    from_binary, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
    Querier, StdError, StdResult, Storage, Uint128,
};
use secret_toolkit::{
    utils::Query,
    snip20::{
        allowance_query, decrease_allowance_msg,
        increase_allowance_msg, register_receive_msg,
        send_msg, batch_send_from_msg,
        set_viewing_key_msg, batch_send_msg,
        batch::{ SendFromAction },
    },
};

use shade_protocol::{
    snip20,
    adapter,
    finance_manager::{
        Allocation, AllocationMeta,
        AllocationType, Config, 
        HandleAnswer, QueryAnswer,
    },
    utils::{
        asset::Contract,
        generic_response::ResponseStatus
    },
};

use crate::{
    query,
    state::{
        allocations_r, allocations_w, asset_list_r, asset_list_w, assets_r, assets_w, config_r,
        config_w, viewing_key_r,
    },
};
use chrono::prelude::*;

/*
pub fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _sender: HumanAddr,
    _from: HumanAddr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<HandleResponse> {

    /* TODO
     * This should never receive funds, maybe should not even register receieve
     * Could potentially register receive when registering an asset to forward to treasury
     */

    let config = config_r(&deps.storage).load()?;
    let asset = assets_r(&deps.storage).load(env.message.sender.to_string().as_bytes())?;

    Ok(HandleResponse {
        messages: vec![
            send_msg(
                config.treasury,
                amount,
                None,
                None,
                None,
                1,
                asset.contract.code_hash.clone(),
                asset.contract.address.clone(),
            )?
        ],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Receive {
            status: ResponseStatus::Success,
        })?),
    })
}
*/

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: Config,
) -> StdResult<HandleResponse> {
    let cur_config = config_r(&deps.storage).load()?;

    if env.message.sender != cur_config.admin {
        return Err(StdError::unauthorized());
    }

    config_w(&mut deps.storage).save(&config)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_register_asset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    contract: &Contract,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    if env.message.sender != config.admin {
        return Err(StdError::unauthorized());
    }

    asset_list_w(&mut deps.storage).update(|mut list| {
        list.push(contract.address.clone());
        Ok(list)
    })?;

    assets_w(&mut deps.storage).save(
        contract.address.to_string().as_bytes(),
        &snip20::fetch_snip20(contract, &deps.querier)?,
    )?;

    allocations_w(&mut deps.storage).save(contract.address.as_str().as_bytes(), &Vec::new())?;

    Ok(HandleResponse {
        messages: vec![
            // Register contract in asset
            register_receive_msg(
                env.contract_code_hash.clone(),
                None,
                256,
                contract.code_hash.clone(),
                contract.address.clone(),
            )?,
            // Set viewing key
            set_viewing_key_msg(
                viewing_key_r(&deps.storage).load()?,
                None,
                256,
                contract.code_hash.clone(),
                contract.address.clone(),
            )?,
        ],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RegisterAsset {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn allocate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: HumanAddr,
    allocation: Allocation,
) -> StdResult<HandleResponse> {

    static ONE_HUNDRED_PERCENT: u128 = 10u128.pow(18);

    let config = config_r(&deps.storage).load()?;

    /* ADMIN ONLY */
    if env.message.sender != config.admin {
        return Err(StdError::unauthorized());
    }

    let key = asset.as_str().as_bytes();

    let mut apps = allocations_r(&deps.storage)
        .may_load(key)?
        .unwrap_or_default();

    let stale_alloc = apps.iter().position(|a| a.contract.address == allocation.contract.address);

    match stale_alloc {
        Some(i) => { apps.remove(i); }
        None => { }
    };

    apps.push(
        AllocationMeta {
            nick: allocation.nick,
            contract: allocation.contract,
            amount: allocation.amount,
            alloc_type: allocation.alloc_type,
            balance: Uint128::zero(),
        }
    );

    if (apps.iter().map(|a| {
        if a.alloc_type == AllocationType::Portion {
            a.amount.u128()
        } else {
            0
        }
    }).sum::<u128>()) >= ONE_HUNDRED_PERCENT {
        return Err(StdError::generic_err(
            "Invalid allocation total exceeding 100%",
        ));
    }

    allocations_w(&mut deps.storage).save(key, &apps)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Allocate{
            status: ResponseStatus::Success,
        })?),
    })
}

/*
pub fn refresh_adapter_balances<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    adapters: Vec<AllocationMeta>,
    asset: Contract,
) -> StdResult<Vec<AllocationMeta>> {

    Ok(adapters.iter().map(|mut a| {
        let mut new_a = a.clone();
        new_a.balance = adapter_balance(deps, a.contract.clone(), &asset).ok().unwrap();
        new_a
    }).collect())
}
*/

pub fn rebalance<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: HumanAddr,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    let full_asset = assets_r(&deps.storage).load(asset.to_string().as_bytes())?;
    let allocations = allocations_r(&mut deps.storage).load(asset.to_string().as_bytes())?;

    let mut allowance = allowance_query(
        &deps.querier,
        config.treasury.clone(),
        env.contract.address.clone(),
        viewing_key_r(&deps.storage).load()?,
        1,
        full_asset.contract.code_hash.clone(),
        full_asset.contract.address.clone(),
    )?.allowance;

    // For refreshed allocation data
    let mut fresh_allocs = allocations.to_vec();
    // Build metadata
    let mut amount_total = Uint128::zero();
    let mut portion_total = Uint128::zero();

    for mut a in fresh_allocs.clone() {
        match a.alloc_type {
            AllocationType::Amount => amount_total += a.balance,
            AllocationType::Portion => {
                a.balance = query::adapter_balance(deps, a.contract, 
                                            &full_asset.contract.address)?;
                portion_total += a.balance;
            }
        };
    }

    let alloc_total = amount_total + portion_total;

    // To be spent in order to fill amounts before unbonding
    //let mut available = cur_allowance.allowance;
    // Batch send_from actions
    let mut actions = vec![];

    for a in fresh_allocs {
        match a.alloc_type {
            // TODO Separate handle for amount refresh
            AllocationType::Amount => { },
            AllocationType::Portion => {

                let amount = a.amount.multiply_ratio(
                    portion_total,
                    1u128.pow(18)
                );
                // Enough allowance to cover
                if amount <= allowance {
                    actions.push(
                        SendFromAction {
                            owner: config.treasury.clone(),
                            recipient: a.contract.address,
                            recipient_code_hash: None,
                            amount,
                            msg: None,
                            memo: None,
                        }
                    );
                    allowance = (allowance - amount)?;
                }
                // Need to unbond from somewhere
                else {
                    // Send all available
                    actions.push(SendFromAction {
                        owner: config.treasury.clone(),
                        recipient: a.contract.address,
                        recipient_code_hash: None,
                        amount: allowance,
                        msg: None,
                        memo: None,
                    });

                    let unbond_amount = (amount - allowance)?;
                    allowance = Uint128::zero();

                    //TODO: unbond(unbond_amount) from adapters
                }
            },
        };
    }

    Ok(HandleResponse {
        messages: vec![
            batch_send_from_msg(
                actions,
                None,
                1,
                full_asset.contract.code_hash.clone(),
                full_asset.contract.address.clone(),
            )?
        ],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Rebalance {
            status: ResponseStatus::Success,
        })?),
    })
}
