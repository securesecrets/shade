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
    adapter,
    treasury::{
        Allowance, Config, Flag, Manager, Account, Status,
        HandleAnswer, QueryAnswer, Balance,
    },
    utils::{
        asset::Contract, 
        generic_response::ResponseStatus,
        cycle::{ Cycle, parse_utc_datetime, exceeds_cycle },
    },
};

use crate::{
    query,
    state::{
        allowances_r, allowances_w, 
        asset_list_r, asset_list_w, 
        assets_r, assets_w, 
        config_r, config_w, 
        viewing_key_r, self_address_r,
        managers_r, managers_w,
        account_r, account_w,
        account_list_r, account_list_w,
        total_unbonding_r,
        total_unbonding_w,
    },
};
use chrono::prelude::*;

pub fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    sender: HumanAddr,
    _from: HumanAddr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<HandleResponse> {

    let key = sender.as_str().as_bytes();

    if let Some(mut account) = account_r(&deps.storage).may_load(&key)? {

        if let Some(i) = account.balances.iter()
                                .position(|b| b.token == env.message.sender) {
            account.balances[i].amount += amount;
        }
        else {
            account.balances.push(Balance {
                token: env.message.sender,
                amount,
            });
        }

        account_w(&mut deps.storage).save(&key, &account)?;
    }

    /* Probably can just wait until rebalance
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

    if let Some(allowances) = allowances_r(&deps.storage).may_load(asset.contract.address.as_str().as_bytes())? {
        for allowance in allowances {
            match allowance {
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
    */

    Ok(HandleResponse {
        messages: vec![],
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

pub fn allowance_last_refresh<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: &Env,
    allowance: &Allowance
) -> StdResult<Option<DateTime<Utc>>> {

    // Parse previous refresh datetime
    let rfc3339 = match allowance {
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
    //let mut portion_total = Uint128::zero();
    let mut out_balance = Uint128::zero();

    let mut managers = managers_r(&deps.storage).load()?;

    // Fetch & sum balances
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
                tolerance,
            } => {
                //portion_total += *portion; //allowance_portion(allowance);
                let i = managers.iter().position(|m| m.contract.address == *spender).unwrap();
                managers[i].balance = adapter::balance_query(&deps,
                                         &full_asset.contract.address.clone(),
                                         managers[i].contract.clone())?;
                out_balance += managers[i].balance;
            },
        }
    }

    managers_w(&mut deps.storage).save(&managers)?;
    let config = config_r(&deps.storage).load()?;

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

                if exceeds_cycle(&datetime, &now, cycle) {
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
                tolerance,
            } => {
                let desired_amount = (balance + out_balance)
                    .multiply_ratio(
                        portion, 
                        10u128.pow(18)
                    );

                let threshold = (balance + out_balance)
                    .multiply_ratio(tolerance, 10u128.pow(18));

                let adapter = managers.clone()
                    .into_iter()
                    .find(|m| m.contract.address == spender)
                    .unwrap();

                let cur_allowance = allowance_query(
                    &deps.querier,
                    env.contract.address.clone(),
                    spender.clone(),
                    key.clone(),
                    1,
                    full_asset.contract.code_hash.clone(),
                    full_asset.contract.address.clone(),
                )?.allowance;

                if cur_allowance + adapter.balance < desired_amount {
                    let increase = (desired_amount - (adapter.balance + cur_allowance))?;
                    if increase < threshold {
                        continue;
                    }
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
                else if cur_allowance + adapter.balance > desired_amount {
                    let mut decrease = ((adapter.balance + cur_allowance) - desired_amount)?;
                    if decrease < threshold {
                        continue;
                    }

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

                    // Unbond remaining
                    if decrease > Uint128::zero() {

                        messages.push(
                            adapter::unbond_msg(
                                asset.clone(), 
                                decrease, 
                                adapter.contract,
                            )?
                        );
                    }

                }
            },
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

    allowances_w(&mut deps.storage).save(contract.address.as_str().as_bytes(), &Vec::new())?;
    total_unbonding_w(&mut deps.storage).save(contract.address.as_str().as_bytes(), &Uint128::zero())?;

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
            desired: Uint128::zero(),
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
        Allowance::Portion { portion, .. } => *portion,
        Allowance::Amount { .. } => Uint128::zero(),
    }
}

fn allowance_amount(allowance: &Allowance) -> Uint128 {
    match allowance {
        Allowance::Amount { amount, .. } => *amount,
        Allowance::Portion { .. } => Uint128::zero(),
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
            if adapters.clone().into_iter().find(|m| m.contract.address == *spender).is_none() {
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

    let spender = match allowance {

        Allowance::Portion {
            spender, portion, last_refresh, tolerance,
        } => {
            apps.push(Allowance::Portion {
                spender: spender.clone(),
                portion: portion.clone(),
                last_refresh: datetime.to_rfc3339(),
                tolerance,
            });
            spender
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
            spender
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

pub fn add_account<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    holder: HumanAddr,
) -> StdResult<HandleResponse> {

    if env.message.sender != config_r(&deps.storage).load()?.admin {
        return Err(StdError::unauthorized());
    }

    let key = holder.as_str().as_bytes();

    account_list_w(&mut deps.storage).update(|mut accounts| {
        if accounts.contains(&holder.clone()) {
            return Err(StdError::generic_err("Account already exists"));
        }
        accounts.push(holder.clone());
        Ok(accounts)
    })?;

    account_w(&mut deps.storage).save(key, 
        &Account {
            balances: Vec::new(),
            unbondings: Vec::new(),
            claimable: Vec::new(),
            status: Status::Active,
        }
    )?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddAccount {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn close_account<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    holder: HumanAddr,
) -> StdResult<HandleResponse> {

    if env.message.sender != config_r(&deps.storage).load()?.admin {
        return Err(StdError::unauthorized());
    }

    let key = holder.as_str().as_bytes();

    if let Some(mut account) = account_r(&deps.storage).may_load(key)? {
        account.status = Status::Closed;
        account_w(&mut deps.storage).save(key, &account)?;
    } else {
        return Err(StdError::generic_err("Account doesn't exist"));
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RemoveAccount {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: HumanAddr,
) -> StdResult<HandleResponse> {

    if env.message.sender != config_r(&deps.storage).load()?.admin {
        return Err(StdError::unauthorized());
    }

    let key = asset.as_str().as_bytes();

    let managers = managers_r(&deps.storage).load()?;
    let allowances = allowances_r(&deps.storage).load(&key)?;

    let mut messages = vec![];

    let mut claimed = Uint128::zero();

    for allowance in allowances {
        match allowance {
            Allowance::Amount { .. } => {},
            Allowance::Portion { spender, .. } => {
                if let Some(manager) = managers.iter().find(|m| m.contract.address == spender) {

                    let claimable = adapter::claimable_query(&deps, &asset, manager.contract.clone())?;

                    if claimable > Uint128::zero() {
                        messages.push(
                            adapter::claim_msg(
                                asset.clone(),
                                manager.contract.clone()
                            )?
                        );
                        claimed += claimable;
                    }
                }
            }
        }
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Claim {
            status: ResponseStatus::Success,
            amount: claimed,
        })?),
    })
}

pub fn unbond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: HumanAddr,
    amount: Uint128,
) -> StdResult<HandleResponse> {

    /*
    if env.message.sender != config_r(&deps.storage).load()?.admin {
        return Err(StdError::unauthorized());
    }
    */

    let account = match account_r(&deps.storage).may_load(&env.message.sender.as_str().as_bytes())? {
        Some(a) => a,
        None => {
            return Err(StdError::unauthorized());
        }
    };

    let managers = managers_r(&deps.storage).load()?;

    let mut messages = vec![];

    let mut unbond_amount = amount;

    for allowance in allowances_r(&deps.storage).load(asset.as_str().as_bytes())? {
        match allowance {
            Allowance::Amount { .. } => {},
            Allowance::Portion { spender, .. } => {
                if let Some(manager) = managers.iter().find(|m| m.contract.address == spender) {
                    let balance = adapter::balance_query(&deps, &asset.clone(), manager.contract.clone())?;

                    if balance > unbond_amount {
                        messages.push(
                            adapter::unbond_msg(
                                asset.clone(),
                                unbond_amount,
                                manager.contract.clone(),
                            )?
                        );
                        unbond_amount = Uint128::zero();
                    }
                    else {
                        messages.push(
                            adapter::unbond_msg(
                                asset.clone(),
                                balance,
                                manager.contract.clone(),
                            )?
                        );
                        unbond_amount = (unbond_amount - balance)?;
                    }
                }
            }
        }

        if unbond_amount == Uint128::zero() {
            break;
        }
    }

    if unbond_amount > Uint128::zero() {
        return Err(StdError::generic_err(
            format!("Failed to fully unbond {}, {} available", 
                    amount, (amount - unbond_amount)?)
        ));
    }

    total_unbonding_w(&mut deps.storage)
        .update(
            asset.as_str().as_bytes(), 
            |u| Ok(u.or(Some(Uint128::zero())).unwrap() + amount)
        )?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Claim {
            status: ResponseStatus::Success,
            amount,
        })?),
    })
}
