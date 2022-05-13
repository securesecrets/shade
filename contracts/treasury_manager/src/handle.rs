use cosmwasm_std;
use cosmwasm_std::{
    from_binary, to_binary, Api, Binary, CosmosMsg, WasmMsg, Env, Extern, HandleResponse, HumanAddr,
    Querier, StdError, StdResult, Storage, Uint128,
};
use secret_toolkit::{
    utils::{
        Query, HandleCallback,
    },
    snip20::{
        allowance_query, decrease_allowance_msg,
        increase_allowance_msg, register_receive_msg,
        send_msg, batch_send_from_msg,
        set_viewing_key_msg, batch_send_msg,
        balance_query,
        batch::{ SendFromAction },
    },
};

use shade_protocol::{
    snip20,
    adapter,
    treasury_manager::{
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
        config_w, viewing_key_r, self_address_r,
    },
};
use chrono::prelude::*;
use std::convert::TryFrom;

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
    }).sum::<u128>()) > ONE_HUNDRED_PERCENT {
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

pub fn claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: HumanAddr,
) -> StdResult<HandleResponse> {

    if assets_r(&deps.storage).may_load(asset.as_str().as_bytes())?.is_none() {
        return Err(StdError::generic_err("Not an asset"));
    }

    let mut total_claimable = Uint128::zero();
    let mut messages = vec![];

    for alloc in allocations_r(&deps.storage).load(asset.to_string().as_bytes())? {

        let claim = adapter::claimable_query(deps, &asset.clone(), alloc.contract.clone())?;

        if claim > Uint128::zero() {
            total_claimable += claim;
            messages.push(adapter::claim_msg(asset.clone(), alloc.contract)?);
        }
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Claim {
            status: ResponseStatus::Success,
            amount: total_claimable,
        })?),
    })
}

pub fn update<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: HumanAddr,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    let full_asset = assets_r(&deps.storage).load(asset.to_string().as_bytes())?;

    let mut allocations = allocations_r(&mut deps.storage).load(asset.to_string().as_bytes())?;

    // Build metadata
    let mut amount_total = Uint128::zero();
    let mut portion_total = Uint128::zero();

    for i in 0..allocations.len() {
        match allocations[i].alloc_type {
            AllocationType::Amount => amount_total += allocations[i].balance,
            AllocationType::Portion => {
                allocations[i].balance = adapter::balance_query(deps, 
                                                   &full_asset.contract.address,
                                                   allocations[i].contract.clone())?;
                portion_total += allocations[i].balance;
            }
        };
    }

    // Batch send_from actions
    let mut send_actions = vec![];
    let mut messages = vec![];

    let mut allowance = allowance_query(
        &deps.querier,
        config.treasury.clone(),
        env.contract.address.clone(),
        viewing_key_r(&deps.storage).load()?,
        1,
        full_asset.contract.code_hash.clone(),
        full_asset.contract.address.clone(),
    )?.allowance;

    let total = portion_total + allowance;

    let mut total_unbond = Uint128::zero();
    let mut total_input = Uint128::zero();

    for adapter in allocations.clone() {
        match adapter.alloc_type {
            // TODO Separate handle for amount refresh
            AllocationType::Amount => { },
            AllocationType::Portion => {

                let desired_amount = adapter.amount.multiply_ratio(
                    total, 10u128.pow(18)
                );

                // .05 || 5%
                //let REBALANCE_THRESHOLD = Uint128(5u128 * 10u128.pow(16));

                if adapter.balance < desired_amount {
                    // Need to add more from allowance
                    let input_amount = (desired_amount - adapter.balance)?;

                    if input_amount <= allowance {
                        total_input += input_amount;
                        send_actions.push(
                            SendFromAction {
                                owner: config.treasury.clone(),
                                recipient: adapter.contract.address,
                                recipient_code_hash: Some(adapter.contract.code_hash),
                                amount: input_amount,
                                msg: None,
                                memo: None,
                            }
                        );
                        allowance = (allowance - input_amount)?;
                    }
                    else {
                        total_input += allowance;
                        // Send all allowance
                        send_actions.push(SendFromAction {
                            owner: config.treasury.clone(),
                            recipient: adapter.contract.address,
                            recipient_code_hash: Some(adapter.contract.code_hash),
                            amount: allowance,
                            msg: None,
                            memo: None,
                        });

                        allowance = Uint128::zero();
                        break;
                    }
                }
            },
        };
    }

    if !send_actions.is_empty() {
        messages.push(
            batch_send_from_msg(
                send_actions,
                None,
                1,
                full_asset.contract.code_hash.clone(),
                full_asset.contract.address.clone(),
            )?
        );
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Update {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn unbond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: HumanAddr,
    amount: Uint128,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    let full_asset = assets_r(&deps.storage).load(asset.to_string().as_bytes())?;

    let mut allocations = allocations_r(&mut deps.storage).load(asset.to_string().as_bytes())?;

    // Build metadata
    let mut amount_total = Uint128::zero();
    let mut portion_total = Uint128::zero();

    for i in 0..allocations.len() {
        match allocations[i].alloc_type {
            AllocationType::Amount => amount_total += allocations[i].balance,
            AllocationType::Portion => {
                allocations[i].balance = adapter::balance_query(deps, 
                                                   &full_asset.contract.address,
                                                   allocations[i].contract.clone())?;
                portion_total += allocations[i].balance;
            }
        };
    }

    let mut messages = vec![];

    let mut reserves = balance_query(
        &deps.querier,
        self_address_r(&deps.storage).load()?,
        viewing_key_r(&deps.storage).load()?,
        1,
        full_asset.contract.code_hash.clone(),
        full_asset.contract.address.clone(),
    )?.amount;

    if reserves > Uint128::zero() {
        messages.push(
            send_msg(
                config.treasury.clone(),
                reserves,
                None,
                None,
                None,
                1,
                full_asset.contract.code_hash.clone(),
                full_asset.contract.address.clone(),
            )?
        );
    }

    let mut allowance = allowance_query(
        &deps.querier,
        config.treasury.clone(),
        env.contract.address.clone(),
        viewing_key_r(&deps.storage).load()?,
        1,
        full_asset.contract.code_hash.clone(),
        full_asset.contract.address.clone(),
    )?.allowance;

    let total = portion_total + allowance;
    let mut total_unbond = (amount - reserves)?;

    allocations.sort_by(|a, b| a.balance.cmp(&b.balance));

    for i in 0..allocations.len() {

        if total_unbond == Uint128::zero() {
            break;
        }

        match allocations[i].alloc_type {
            // TODO Separate handle for amount refresh
            //      Or just do cycle::constant amounts
            AllocationType::Amount => { },
            AllocationType::Portion => {

                let desired_amount = allocations[i].amount.multiply_ratio(
                    total, 10u128.pow(18)
                );

                let unbondable = adapter::unbondable_query(&deps,
                                      &asset,
                                      allocations[i].contract.clone(),
                                      )?;

                if total_unbond > unbondable {
                    messages.push(
                        adapter::unbond_msg(
                            asset.clone(),
                            unbondable,
                            allocations[i].contract.clone()
                        )?
                    );
                    total_unbond = (total_unbond - unbondable)?;
                }
                else {
                    messages.push(
                        adapter::unbond_msg(
                            asset.clone(),
                            total_unbond, 
                            allocations[i].contract.clone()
                        )?
                    );
                    total_unbond = Uint128::zero()
                }
            },
        };
    }


    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Unbond {
            status: ResponseStatus::Success,
            amount: total_unbond
        })?),
    })
}
