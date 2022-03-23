use cosmwasm_std;
use cosmwasm_std::{
    from_binary, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
    Querier, StdError, StdResult, Storage, Uint128,
};
use secret_toolkit;
use secret_toolkit::snip20::{
    allowance_query, decrease_allowance_msg, increase_allowance_msg, register_receive_msg,
    send_msg, set_viewing_key_msg, batch_send_msg,
};

use shade_protocol::{
    snip20,
    treasury::{Allocation, Config, Flag, HandleAnswer, QueryAnswer, RefreshTracker},
    utils::{asset::Contract, generic_response::ResponseStatus},
};

use crate::{
    query,
    state::{
        allocations_r, allocations_w, asset_list_r, asset_list_w, assets_r, assets_w, config_r,
        config_w, last_allowance_refresh_r, last_allowance_refresh_w, viewing_key_r,
        rewards_tracking_r, rewards_tracking_w,
    },
};
use chrono::prelude::*;

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
     * Could potentiall register receive (cheap) when registering an asset
     * In case of an error forward funds to treasury
     */

    let config = config_r(&deps.storage).load()?;

    Ok(HandleResponse {
        messages: vec![
            send_msg(
                    config.treasury,
                    to_allocate,
                    None,
                    None,
                    1,
                    full_asset.contract.code_hash.clone(),
                    full_asset.contract.address.clone(),
            )?
        ],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Receive {
            status: ResponseStatus::Success,
        })?),
    })
}

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

pub fn refresh_allowance<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
) -> StdResult<HandleResponse> {
    let naive = NaiveDateTime::from_timestamp(env.block.time as i64, 0);
    let now: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    // Parse previous refresh datetime
    let last_refresh = last_allowance_refresh_r(&deps.storage)
        .load()
        .and_then(|rfc3339| {
            DateTime::parse_from_rfc3339(&rfc3339)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|_| StdError::generic_err("Failed to parse previous datetime"))
        })?;

    // Fail if we have already refreshed this month
    if now.year() <= last_refresh.year() && now.month() <= last_refresh.month() {
        return Err(StdError::generic_err(
            format!( "Last refresh too recent: {}", last_refresh.to_rfc3339())
        ));
    }

    last_allowance_refresh_w(&mut deps.storage).save(&now.to_rfc3339())?;

    Ok(HandleResponse {
        messages: do_allowance_refresh(deps, env)?,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RefreshAllowance {
            status: ResponseStatus::Success,
        })?),
    })
}

/* Not exposed as a tx
 */
pub fn do_allowance_refresh<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: &Env,
) -> StdResult<Vec<CosmosMsg>> {
    let mut messages = vec![];

    let key = viewing_key_r(&deps.storage).load()?;

    for asset in asset_list_r(&deps.storage).load()? {
        for alloc in allocations_r(&deps.storage).load(asset.as_str().as_bytes())? {
            if let Allocation::Allowance { spender, amount } = alloc {
                let full_asset = assets_r(&deps.storage).load(asset.as_str().as_bytes())?;
                // Determine current allowance
                let cur_allowance = allowance_query(
                    &deps.querier,
                    env.contract.address.clone(),
                    spender.clone(),
                    key.clone(),
                    1,
                    full_asset.contract.code_hash.clone(),
                    full_asset.contract.address.clone(),
                )?;

                match amount.cmp(&cur_allowance.allowance) {
                    // decrease allowance
                    std::cmp::Ordering::Less => {
                        messages.push(decrease_allowance_msg(
                            spender.clone(),
                            (cur_allowance.allowance - amount)?,
                            None,
                            None,
                            1,
                            full_asset.contract.code_hash.clone(),
                            full_asset.contract.address.clone(),
                        )?);
                    }
                    // increase allowance
                    std::cmp::Ordering::Greater => {
                        messages.push(increase_allowance_msg(
                            spender.clone(),
                            (amount - cur_allowance.allowance)?,
                            None,
                            None,
                            1,
                            full_asset.contract.code_hash.clone(),
                            full_asset.contract.address.clone(),
                        )?);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(messages)
}

pub fn one_time_allowance<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: HumanAddr,
    spender: HumanAddr,
    amount: Uint128,
    expiration: Option<u64>,
) -> StdResult<HandleResponse> {
    let cur_config = config_r(&deps.storage).load()?;

    if env.message.sender != cur_config.admin {
        return Err(StdError::unauthorized());
    }

    let full_asset = assets_r(&deps.storage)
        .may_load(asset.as_str().as_bytes())?
        .ok_or_else(|| StdError::generic_err(format!("Unknown Asset: {}", asset)))?;

    Ok(HandleResponse {
        messages: vec![
            increase_allowance_msg(
                spender,
                amount,
                expiration,
                None,
                1,
                full_asset.contract.code_hash.clone(),
                full_asset.contract.address,
            )?
        ],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::OneTimeAllowance {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_register_asset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    contract: &Contract,
    reserves: Option<Uint128>,
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

    let allocs = reserves
        .map(|r| vec![Allocation::Reserves { allocation: r }])
        .unwrap_or_default();

    allocations_w(&mut deps.storage).save(contract.address.as_str().as_bytes(), &allocs)?;

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

// extract contract address if any
fn allocation_address(allocation: &Allocation) -> Option<&HumanAddr> {
    match allocation {
        Allocation::Allowance { spender, .. } => Some(&spender),
        Allocation::Rewards { contract, .. }
        | Allocation::SingleAsset { contract, .. }
        //| Allocation::MultiAsset { contract, .. } 
        => Some(&contract.address),
        _ => None,
    }
}

// extract allocaiton portion
fn allocation_portion(allocation: &Allocation) -> u128 {
    match allocation {
        Allocation::Reserves { allocation }
        | Allocation::SingleAsset { allocation, .. }
        //| Allocation::MultiAsset { allocation, .. } 
        => allocation.u128(),
        Allocation::Allowance { .. }
        | Allocation::Rewards { .. } => 0,
    }
}

pub fn register_allocation<S: Storage, A: Api, Q: Querier>(
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

    // might be used later
    let _full_asset = assets_r(&deps.storage)
        .may_load(asset.to_string().as_bytes())
        .and_then(|asset| {
            asset.ok_or_else(|| StdError::generic_err("Unexpected response for balance"))
        })?;

    // might be used later
    let _liquid_balance = query::balance(deps, &asset).and_then(|r| match r {
        QueryAnswer::Balance { amount } => Ok(amount),
        _ => Err(StdError::generic_err("Unexpected response for balance")),
    })?;

    let key = asset.as_str().as_bytes();

    let mut apps = allocations_r(&deps.storage)
        .may_load(key)?
        .unwrap_or_default();

    let alloc_address = allocation_address(&allocation);

    // find any old allocations with the same contract address & sum current allocations in one loop.
    // saves looping twice in the worst case
    // TODO: Remove Rewards/Reserves if this would be one of those
    let (stale_alloc, curr_alloc_portion) =
        apps.iter()
            .enumerate()
            .fold((None, 0u128), |(stale_alloc, curr_allocs), (idx, a)| {
                if stale_alloc.is_none() && allocation_address(a) == alloc_address {
                    (Some(idx), curr_allocs)
                } else {
                    (stale_alloc, curr_allocs + allocation_portion(a))
                }
            });

    if let Some(old_alloc_idx) = stale_alloc {
        apps.remove(old_alloc_idx);
    }

    let new_alloc_portion = allocation_portion(&allocation);

    // NOTE: should this be '>' if 1e18 == 100%?
    if curr_alloc_portion + new_alloc_portion > ONE_HUNDRED_PERCENT {
        return Err(StdError::generic_err(
            "Invalid allocation total exceeding 100%",
        ));
    }

    apps.push(allocation.clone());

    allocations_w(&mut deps.storage).save(key, &apps)?;

    // Init the rewards to 0, to be refreshed next opportunity
    match allocation {
        Allocation::Rewards { contract, daily_amount } => {
            let naive = NaiveDateTime::from_timestamp(0, 0);
            let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
            let tracker = RefreshTracker {
               amount: Uint128::zero(),
               limit: daily_amount,
               last_refresh: datetime.to_rfc3339(),
            };
            rewards_tracking_w(&mut deps.storage).save(key, &tracker)?;
        },
        _ => {}
    }

    /*TODO: Need to re-allocate/re-balance funds based on the new addition
     * get Uint128 math functions to do these things (untested)
     * re-add send_msg below
     */

    /*
    let liquid_portion = (allocated_portion * liquid_balance) / allocated_portion;

    // Determine how much of current balance is to be allocated
    let to_allocate = liquid_balance - (alloc_portion / liquid_portion);
    */

    Ok(HandleResponse {
        messages: vec![
            /*
            send_msg(
                    alloc_address,
                    to_allocate,
                    None,
                    None,
                    1,
                    full_asset.contract.code_hash.clone(),
                    full_asset.contract.address.clone(),
            )?
            */
        ],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RegisterApp {
            status: ResponseStatus::Success,
        })?),
    })
}
