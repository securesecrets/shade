use cosmwasm_std;
use cosmwasm_std::{
    from_binary, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
    Querier, StdError, StdResult, Storage, Uint128,
};
use secret_toolkit::{
    snip20::{
        register_receive_msg, allowance_query,
        decrease_allowance_msg, increase_allowance_msg,
        set_viewing_key_msg, balance_query,
    },
    utils::Query,
};

use shade_protocol::{
    snip20,
    treasury::{
        Allowance, Config, Flag, Cycle, Manager,
        HandleAnswer, QueryAnswer,
    },
    adapter,
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
        if flag.flag == "unallowanceated" {
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

    if let Some(allowances) = allowances_r(&deps.storage).may_load(asset.contract.address.as_str().as_bytes())? {
        for allowance in allowances {
            match allowance {
                Allowance::Reserves { .. }  => {},
                Allowance::Amount { .. } => { },
                Allowance::Portion {
                    spender,
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
    asset: HumanAddr,
) -> StdResult<HandleResponse> {

    let naive = NaiveDateTime::from_timestamp(env.block.time as i64, 0);
    let now: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    let key = viewing_key_r(&deps.storage).load()?;
    let self_address = self_address_r(&deps.storage).load()?;
    let mut messages = vec![];

    let full_asset = match assets_r(&deps.storage).may_load(asset.as_str().as_bytes())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Not an asset"));
        }
    };
    let allowances = allowances_r(&deps.storage).load(asset.as_str().as_bytes())?;

    let balance = balance_query(
                    &deps.querier,
                    self_address,
                    key.clone(),
                    1,
                    full_asset.contract.code_hash.clone(),
                    full_asset.contract.address.clone(),
                  )?.amount;

    let mut amount_total = Uint128::zero();
    let mut portion_total = Uint128::zero();
    let mut out_balance = Uint128::zero();

    //Build metadata
    for allowance in &allowances {
        match allowance {
            Allowance::Amount {
                spender,
                cycle,
                amount,
                last_refresh,
            } => {
                //TODO: Query allowance
                //amount_total += allowance_amount(allowance);
            },
            Allowance::Portion {
                spender,
                portion,
                last_refresh,
            } => {
                portion_total += allowance_portion(allowance);
                let adapter = match managers_r(&deps.storage).load()?.into_iter().find(|m| m.contract.address == *spender) {
                    Some(adapter) => adapter,
                    None => {
                        return Err(StdError::generic_err(format!("{} is not a adapter", spender)));
                    }
                };
                out_balance += adapter::balance_query(&deps, 
                                     &full_asset.contract.address.clone(),
                                     adapter.contract.clone())?;
            },
            Allowance::Reserves { .. } => { },
        }
    }

    // Perform rebalance
    for allowance in allowances {

        match allowance {

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
                portion,
                last_refresh,
            } => {
                let desired_amount = (balance + out_balance).multiply_ratio(portion, 10u128.pow(18));

                let datetime = parse_utc_datetime(&last_refresh)?;
                let adapter = match managers_r(&deps.storage).load()?.into_iter().find(|m| m.contract.address == spender) {
                    Some(adapter) => adapter,
                    None => {
                        return Err(StdError::generic_err(format!("{} is not a adapter", spender)));
                    }
                };

                let cur_allowance = allowance_query(
                    &deps.querier,
                    env.contract.address.clone(),
                    spender.clone(),
                    key.clone(),
                    1,
                    full_asset.contract.code_hash.clone(),
                    full_asset.contract.address.clone(),
                )?.allowance;

                let adapter_balance = adapter::balance_query(&deps, 
                                     &full_asset.contract.address.clone(),
                                     adapter.contract.clone())?;

                if cur_allowance + adapter_balance < desired_amount {
                    let increase = (desired_amount - (adapter_balance + cur_allowance))?;
                    messages.push(
                        increase_allowance_msg(
                            spender,
                            increase,
                            None,
                            None,
                            1,
                            full_asset.contract.code_hash.clone(),
                            full_asset.contract.address.clone(),
                        )?
                    );
                }
                else if cur_allowance + adapter_balance > desired_amount {
                    let mut decrease = ((adapter_balance + cur_allowance) - desired_amount)?;
                    //TODO: Implement rebalance threshould to minimize thrashing

                    // Remove allowance first
                    if cur_allowance > Uint128::zero() {

                        if cur_allowance < decrease {
                            messages.push(
                                decrease_allowance_msg(
                                    spender,
                                    cur_allowance,
                                    None,
                                    None,
                                    1,
                                    full_asset.contract.code_hash.clone(),
                                    full_asset.contract.address.clone(),
                                )?
                            );
                            decrease = (decrease - cur_allowance)?;
                        }
                        else {
                            messages.push(
                                decrease_allowance_msg(
                                    spender,
                                    decrease,
                                    None,
                                    None,
                                    1,
                                    full_asset.contract.code_hash.clone(),
                                    full_asset.contract.address.clone(),
                                )?
                            );
                            decrease = Uint128::zero();
                        }
                    }

                    // Unbond
                    if decrease > Uint128::zero() {
                        if adapter_balance > decrease {
                            //return Err(StdError::generic_err(format!("OverFunded, Unbonding {}", decrease)));
                            messages.push(
                                adapter::unbond_msg(asset.clone(), decrease, adapter.contract)?
                            );
                            decrease = Uint128::zero();
                        }
                        else {
                            //return Err(StdError::generic_err(format!("OverFunded, Unbonding full balance {}", adapter_balance)));
                            messages.push(
                                adapter::unbond_msg(asset.clone(), adapter_balance, adapter.contract)?
                            );
                            decrease = (decrease - adapter_balance)?;
                        }
                    }

                    if decrease == Uint128::zero() {
                        break;
                    }
                }
            },
            Allowance::Reserves { .. } => { },
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

    let allowances = reserves
        .map(|r| vec![Allowance::Reserves { portion: r }])
        .unwrap_or_default();

    allowances_w(&mut deps.storage).save(contract.address.as_str().as_bytes(), &allowances)?;

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

    managers_w(&mut deps.storage).update(|mut adapters| {
        if adapters.iter().map(|m| m.contract.clone()).collect::<Vec<_>>().contains(&contract) {
            return Err(StdError::generic_err("Manager already registered"));
        }
        adapters.push(Manager {
            contract: contract.clone(),
            balance: Uint128::zero(),
        });
        Ok(adapters)
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

// extract allowanceaiton portion
fn allowance_portion(allowance: &Allowance) -> Uint128 {
    match allowance {
        Allowance::Reserves { portion } => *portion,
        Allowance::Portion { portion, .. } => *portion,
        Allowance::Amount { .. } => Uint128::zero(),
    }
}

fn allowance_amount(allowance: &Allowance) -> Uint128 {
    match allowance {
        Allowance::Amount { amount, .. } => *amount,
        Allowance::Portion { .. }
        | Allowance::Reserves { .. } => Uint128::zero(),
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

    let adapters = managers_r(&deps.storage).load()?;

    // Disallow Portion on non-adapters
    match allowance {
        Allowance::Portion {
            ref spender, ..
        } => {
            if adapters.into_iter().find(|m| m.contract.address == *spender).is_none() {
                return Err(StdError::generic_err("Portion allowances to adapters only"));
            }
        }
        _ => {}
    };

    let key = asset.as_str().as_bytes();

    let mut apps = allowances_r(&deps.storage)
        .may_load(key)?
        .unwrap_or_default();

    let allow_address = allowance_address(&allowance);

    // find any old allowances with the same contract address & sum current allowances in one loop.
    // saves looping twice in the worst case
    // TODO: Remove Reserves if this would be one of those
    let (stale_allowance, cur_allowance_portion) =
        apps.iter()
            .enumerate()
            .fold((None, 0u128), |(stale_allowance, cur_allowances), (idx, a)| {
                if stale_allowance.is_none() && allowance_address(a) == allow_address {
                    (Some(idx), cur_allowances)
                } else {
                    (stale_allowance, cur_allowances + allowance_portion(a).u128())
                }
            });

    if let Some(old_allowance_idx) = stale_allowance {
        apps.remove(old_allowance_idx);
    }

    let new_allowance_portion = allowance_portion(&allowance).u128();

    if cur_allowance_portion + new_allowance_portion > ONE_HUNDRED_PERCENT {
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
            spender, portion, last_refresh
        } => {
            apps.push(Allowance::Portion {
                spender: spender.clone(),
                portion: portion.clone(),
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
                spender: spender.clone(),
                cycle: cycle.clone(),
                amount: amount.clone(),
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


