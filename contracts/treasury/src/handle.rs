use cosmwasm_std;
use cosmwasm_std::{
    from_binary, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
    Querier, StdError, StdResult, Storage, Uint128,
};
use secret_toolkit;
use secret_toolkit::{
    snip20::{
        allowance_query, decrease_allowance_msg, increase_allowance_msg, register_receive_msg,
        send_msg, set_viewing_key_msg, batch_send_msg,
    },
    utils::Query,
};

use shade_protocol::{
    snip20,
    treasury::{
        Allocation, Config, Flag, Cycle, 
        HandleAnswer, QueryAnswer, RefreshTracker
    },
    manager,
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
        outstanding_allowances_r, outstanding_allowances_w,
        rewards_tracking_r, rewards_tracking_w,
        self_address_r,
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

    if let Some(f) = msg {
        let flag: Flag = from_binary(&f)?;
        // NOTE: would this be better as a non-exhaustive enum?
        // https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute
        if flag.flag == "unallocated" {
            return Ok(HandleResponse {
                messages: vec![],
                log: vec![],
                data: Some(to_binary(&HandleAnswer::Receive {
                    status: ResponseStatus::Success,
                })?),
            });
        }
    };

    let asset = assets_r(&deps.storage).load(env.message.sender.as_str().as_bytes())?;

    let mut messages = vec![];
    //let mut send_actions = vec![];

    if let Some(allocs) = allocations_r(&deps.storage).may_load(asset.contract.address.as_str().as_bytes())? {
        for alloc in allocs {
            match alloc {
                Allocation::Reserves { .. }  => {},
                Allocation::Amount { .. } => { },
                Allocation::Portion {
                    spender,
                    cycle,
                    portion,
                    last_refresh,
                } => {
                    messages.push(
                        increase_allowance_msg(
                            spender.clone(),
                            amount.multiply_ratio(portion, 10u128.pow(18)),
                            None,
                            None,
                            1,
                            asset.contract.code_hash.clone(),
                            asset.contract.address.clone(),
                        )?
                    );
                },
            }
        }
    }

    Ok(HandleResponse {
        messages,
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

pub fn parse_utc_datetime(
    last_refresh: &String,
) -> StdResult<DateTime<Utc>> {

    DateTime::parse_from_rfc3339(&last_refresh)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| 
            StdError::generic_err(
                format!("Failed to parse datetime {}", last_refresh)
            )
        )
}
pub fn allocation_last_refresh<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: &Env,
    allocation: &Allocation
) -> StdResult<Option<DateTime<Utc>>> {

    //let naive = NaiveDateTime::from_timestamp(env.block.time as i64, 0);
    //let now: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    // Parse previous refresh datetime
    let rfc3339 = match allocation {
        Allocation::Reserves { .. } => { return Ok(None); }
        Allocation::Amount { last_refresh, .. } => last_refresh,
        Allocation::Portion { last_refresh, .. } => last_refresh,
    };

    DateTime::parse_from_rfc3339(&rfc3339)
        .map(|dt| Some(dt.with_timezone(&Utc)))
        .map_err(|_| StdError::generic_err(
            format!("Failed to parse datetime {}", rfc3339)
        ))
}

pub fn rebalance<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: Option<HumanAddr>,
) -> StdResult<HandleResponse> {

    let naive = NaiveDateTime::from_timestamp(env.block.time as i64, 0);
    let now: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    let key = viewing_key_r(&deps.storage).load()?;
    let mut messages = vec![];

    // Configured for single-asset
    let asset_list = match asset {
        None => asset_list_r(&deps.storage).load()?,
        Some(a) => vec![a],
    };

    for asset in asset_list {

        for alloc in allocations_r(&deps.storage).load(asset.as_str().as_bytes())? {

            match alloc {

                Allocation::Amount {
                    spender,
                    cycle,
                    amount,
                    last_refresh,
                } => {
                    let datetime = parse_utc_datetime(&last_refresh)?;

                    let full_asset = assets_r(&deps.storage).load(asset.as_str().as_bytes())?;

                    if needs_refresh(datetime, now, cycle) {
                        if let Some(msg) = set_allowance(&deps, env,
                                                  spender, amount,
                                                  key.clone(), full_asset.contract)? {
                            messages.push(msg);
                        }
                    }
                },
                Allocation::Portion {
                    spender,
                    cycle,
                    portion,
                    last_refresh,
                } => {
                    let datetime = parse_utc_datetime(&last_refresh)?;
                    let full_asset = assets_r(&deps.storage).load(asset.as_str().as_bytes())?;
                    // TODO: Calculate manager balances/portions to determine balance
                    if needs_refresh(datetime, now, cycle) {
                        // convert portion -> amount
                        /*
                        if let Some(msg) = set_allowance(&deps, env,
                                                  spender, amount,
                                                  key, full_asset.contract)? {
                            messages.push(msg);
                        }
                        */
                    }

                },
                Allocation::Reserves { .. } => { },
            }
        }
    };

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Rebalance {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn needs_refresh(
    last_refresh: DateTime<Utc>,
    now: DateTime<Utc>,
    cycle: Cycle,
) -> bool {

    match cycle {
        Cycle::Once => false,
        Cycle::Constant => true,
        Cycle::Daily { days } => now.num_days_from_ce() - last_refresh.num_days_from_ce() > days.u128() as i32,
        Cycle::Monthly { months } => {
            let mut month_diff = 0u32;

            if now.year() > last_refresh.year() {
                month_diff = (12u32 - last_refresh.month()) + now.month();
            }
            else {
                month_diff = now.month() - last_refresh.month();
            }

            month_diff > months.u128() as u32
        }
    }
}

pub fn set_allowance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: &Env,
    spender: HumanAddr,
    amount: Uint128,
    key: String,
    asset: Contract,
) -> StdResult<Option<CosmosMsg>> {

    let cur_allowance = allowance_query(
        &deps.querier,
        env.contract.address.clone(),
        spender.clone(),
        key,
        1,
        asset.code_hash.clone(),
        asset.address.clone(),
    )?;

    match amount.cmp(&cur_allowance.allowance) {
        // Decrease Allowance
        std::cmp::Ordering::Less => {
            Ok(Some(
                decrease_allowance_msg(
                    spender.clone(),
                    (cur_allowance.allowance - amount)?,
                    None,
                    None,
                    1,
                    asset.code_hash.clone(),
                    asset.address.clone(),
                )?
            ))
        },
        // Increase Allowance
        std::cmp::Ordering::Greater => {
            Ok(Some(
                increase_allowance_msg(
                    spender.clone(),
                    (amount - cur_allowance.allowance)?,
                    None,
                    None,
                    1,
                    asset.code_hash.clone(),
                    asset.address.clone(),
                )?
            ))
        },
        _ => { Ok(None) }
    }
}

/* Gets the outstanding balance of a specific asset from a specific manager
 */
pub fn manager_allowance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    manager: Contract,
    asset: Contract,
) -> StdResult<Uint128> {

    let self_address = self_address_r(&deps.storage).load()?;
    let key = viewing_key_r(&deps.storage).load()?;

    Ok(allowance_query(
        &deps.querier,
        self_address,
        manager.address,
        key,
        1,
        asset.code_hash.clone(),
        asset.address.clone(),
    )?.allowance)
}

pub fn manager_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    manager: Contract,
    asset: Contract,
) -> StdResult<Uint128> {

    match (manager::QueryMsg::Balance {
        asset: asset.address
    }.query(&deps.querier, manager.code_hash, manager.address.clone())?) {
        manager::QueryAnswer::Balance { amount } => Ok(amount),
        _ => Err(
            StdError::generic_err(
                format!("Failed to query manager balance from {}", manager.address)
            )
        )
    }
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
        .map(|r| vec![Allocation::Reserves { portion: r }])
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
        Allocation::Amount { spender, .. } => Some(&spender),
        Allocation::Portion { spender, .. } => Some(&spender),
        _ => None,
    }
}

// extract allocaiton portion
fn allocation_portion(allocation: &Allocation) -> u128 {
    match allocation {
        Allocation::Reserves { portion } => portion.u128(),
        Allocation::Portion { portion, .. } => portion.u128(),
        Allocation::Amount { .. } => 0,
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

    // Disallow Portion with Cycle::Once
    match &allocation {
        Allocation::Portion {
            cycle, ..
        } => {
            match cycle {
                Cycle::Once => {
                    return Err(StdError::generic_err("Cannot give a one-time portion allowance"));
                }
                _ => {}
            }
        }
        _ => {}
    };

    let key = asset.as_str().as_bytes();

    let mut apps = allocations_r(&deps.storage)
        .may_load(key)?
        .unwrap_or_default();

    let alloc_address = allocation_address(&allocation);

    // find any old allocations with the same contract address & sum current allocations in one loop.
    // saves looping twice in the worst case
    // TODO: Remove Reserves if this would be one of those
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

    if curr_alloc_portion + new_alloc_portion > ONE_HUNDRED_PERCENT {
        return Err(StdError::generic_err(
            "Invalid allocation total exceeding 100%",
        ));
    }

    // Zero the last-refresh
    let datetime: DateTime<Utc> = DateTime::from_utc(
        NaiveDateTime::from_timestamp(0, 0),
        Utc
    );

    match allocation {
        Allocation::Portion {
            spender, cycle, portion, last_refresh
        } => {
            apps.push(Allocation::Portion {
                spender, 
                cycle, 
                portion, 
                last_refresh: datetime.to_rfc3339()
            });
        },
        Allocation::Amount {
            spender,
            cycle,
            amount,
            last_refresh,
        }=> {
            apps.push(Allocation::Amount {
                spender, 
                cycle, 
                amount, 
                last_refresh: datetime.to_rfc3339()
            });
        }
        Allocation::Reserves {
            portion,
        } => {
            apps.push(Allocation::Reserves {
                portion
            });
        }
    };

    allocations_w(&mut deps.storage).save(key, &apps)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RegisterAllocation {
            status: ResponseStatus::Success,
        })?),
    })
}


