use cosmwasm_std;
use cosmwasm_std::{
    from_binary, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
    Querier, StdError, StdResult, Storage, Uint128,
};
use secret_toolkit::{
    snip20::{
        register_receive_msg, allowance_query,
        decrease_allowance_msg, increase_allowance_msg,
        set_viewing_key_msg,
    },
    utils::Query,
};

use shade_protocol::{
    snip20,
    treasury::{
        Allowance, Config, Flag, Cycle, 
        HandleAnswer, QueryAnswer,
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
        allowances_r, allowances_w, asset_list_r, asset_list_w, assets_r, assets_w, config_r,
        config_w, viewing_key_r,
        current_allowances_r, current_allowances_w,
        self_address_r,
        managers_r, managers_w,
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

    if let Some(allocs) = allowances_r(&deps.storage).may_load(asset.contract.address.as_str().as_bytes())? {
        for alloc in allocs {
            match alloc {
                Allowance::Reserves { .. }  => {},
                Allowance::Amount { .. } => { },
                Allowance::Portion {
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
pub fn allowance_last_refresh<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: &Env,
    allowance: &Allowance
) -> StdResult<Option<DateTime<Utc>>> {

    //let naive = NaiveDateTime::from_timestamp(env.block.time as i64, 0);
    //let now: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    // Parse previous refresh datetime
    let rfc3339 = match allowance {
        Allowance::Reserves { .. } => { return Ok(None); }
        Allowance::Amount { last_refresh, .. } => last_refresh,
        Allowance::Portion { last_refresh, .. } => last_refresh,
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

        let full_asset = assets_r(&deps.storage).load(asset.as_str().as_bytes())?;
        let allowances = allowances_r(&deps.storage).load(asset.as_str().as_bytes())?;

        let mut amount_total = Uint128::zero();
        let mut portion_total = Uint128::zero();

        //Build metadata
        for alloc in &allowances {
            match alloc {
                Allowance::Amount { amount, .. } => {
                    amount_total += *amount;
                },
                Allowance::Portion { portion, .. }
                | Allowance::Reserves { portion, .. } => { portion_total += *portion; }
            }
        }

        // Perform rebalance
        for alloc in allowances {

            match alloc {

                Allowance::Amount {
                    spender,
                    cycle,
                    amount,
                    last_refresh,
                } => {
                    let datetime = parse_utc_datetime(&last_refresh)?;


                    if needs_refresh(datetime, now, cycle) {
                        if let Some(msg) = set_allowance(&deps, env,
                                                  spender, amount,
                                                  key.clone(), full_asset.contract.clone())? {
                            messages.push(msg);
                        }
                    }
                },
                Allowance::Portion {
                    spender,
                    cycle,
                    portion,
                    last_refresh,
                } => {
                    let amount = portion.multiply_ratio(1u128, portion_total);

                    let datetime = parse_utc_datetime(&last_refresh)?;
                    let managers = managers_r(&deps.storage).load()?;
                    let balance = match managers.into_iter().find(|m| m.address == spender) {
                        Some(m) => manager_balance(&deps, m, full_asset.contract.clone())?,
                        None => { 
                            return Err(StdError::generic_err("Cannot portion to a non-manager"));
                        }
                    };

                    if balance < amount {
                        if let Some(msg) = set_allowance(&deps, env,
                                                  spender, amount,
                                                  key.clone(), full_asset.contract.clone())? {
                            messages.push(msg);
                        }
                    }

                    // TODO: Calculate manager balances/portions to determine balance
                    // Maybe we don't need a "refresh" cycle on managers?
                    // Force portions to be registered managers?
                    /*
                    if needs_refresh(datetime, now, cycle) {
                        // convert portion -> amount
                        if let Some(msg) = set_allowance(&deps, env,
                                                  spender, amount,
                                                  key, full_asset.contract)? {
                            messages.push(msg);
                        }
                    }
                    */
                },
                Allowance::Reserves { .. } => { },
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
        // NOTE: idk about this one
        Cycle::Constant => true,
        Cycle::Daily { days } => now.num_days_from_ce() - last_refresh.num_days_from_ce() >= days.u128() as i32,
        Cycle::Monthly { months } => {
            let mut month_diff = 0u32;

            if now.year() > last_refresh.year() {
                month_diff = (12u32 - last_refresh.month()) + now.month();
            }
            else {
                month_diff = now.month() - last_refresh.month();
            }

            month_diff >= months.u128() as u32
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
        .map(|r| vec![Allowance::Reserves { portion: r }])
        .unwrap_or_default();

    allowances_w(&mut deps.storage).save(contract.address.as_str().as_bytes(), &allocs)?;

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

pub fn register_manager<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    contract: &mut Contract,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    if env.message.sender != config.admin {
        return Err(StdError::unauthorized());
    }

    managers_w(&mut deps.storage).update(|mut managers| {
        if managers.contains(&contract) {
            return Err(StdError::generic_err("Manager already registered"));
        }
        managers.push(contract.clone());
        Ok(managers)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RegisterAsset {
            status: ResponseStatus::Success,
        })?),
    })
}

// extract contract address if any
fn allowance_address(allowance: &Allowance) -> Option<&HumanAddr> {
    match allowance {
        Allowance::Amount { spender, .. } => Some(&spender),
        Allowance::Portion { spender, .. } => Some(&spender),
        _ => None,
    }
}

// extract allocaiton portion
fn allowance_portion(allowance: &Allowance) -> u128 {
    match allowance {
        Allowance::Reserves { portion } => portion.u128(),
        Allowance::Portion { portion, .. } => portion.u128(),
        Allowance::Amount { .. } => 0,
    }
}

pub fn allowance<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: HumanAddr,
    allowance: Allowance,
) -> StdResult<HandleResponse> {
    static ONE_HUNDRED_PERCENT: u128 = 10u128.pow(18);

    let config = config_r(&deps.storage).load()?;

    /* ADMIN ONLY */
    if env.message.sender != config.admin {
        return Err(StdError::unauthorized());
    }

    // Disallow Portion with Cycle::Once
    match &allowance {
        Allowance::Portion {
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

    let mut apps = allowances_r(&deps.storage)
        .may_load(key)?
        .unwrap_or_default();

    let alloc_address = allowance_address(&allowance);

    // find any old allowances with the same contract address & sum current allowances in one loop.
    // saves looping twice in the worst case
    // TODO: Remove Reserves if this would be one of those
    let (stale_alloc, curr_alloc_portion) =
        apps.iter()
            .enumerate()
            .fold((None, 0u128), |(stale_alloc, curr_allocs), (idx, a)| {
                if stale_alloc.is_none() && allowance_address(a) == alloc_address {
                    (Some(idx), curr_allocs)
                } else {
                    (stale_alloc, curr_allocs + allowance_portion(a))
                }
            });

    if let Some(old_alloc_idx) = stale_alloc {
        apps.remove(old_alloc_idx);
    }

    let new_alloc_portion = allowance_portion(&allowance);

    if curr_alloc_portion + new_alloc_portion > ONE_HUNDRED_PERCENT {
        return Err(StdError::generic_err(
            "Invalid allowance total exceeding 100%",
        ));
    }

    // Zero the last-refresh
    let datetime: DateTime<Utc> = DateTime::from_utc(
        NaiveDateTime::from_timestamp(0, 0),
        Utc
    );

    match allowance {
        Allowance::Portion {
            spender, cycle, portion, last_refresh
        } => {
            apps.push(Allowance::Portion {
                spender, 
                cycle, 
                portion, 
                last_refresh: datetime.to_rfc3339()
            });
        },
        Allowance::Amount {
            spender,
            cycle,
            amount,
            last_refresh,
        }=> {
            apps.push(Allowance::Amount {
                spender, 
                cycle, 
                amount, 
                last_refresh: datetime.to_rfc3339()
            });
        }
        Allowance::Reserves {
            portion,
        } => {
            apps.push(Allowance::Reserves {
                portion
            });
        }
    };

    allowances_w(&mut deps.storage).save(key, &apps)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Allowance {
            status: ResponseStatus::Success,
        })?),
    })
}


