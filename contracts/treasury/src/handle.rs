use cosmwasm_std::{
    self,
    to_binary,
    Api,
    Binary,
    CosmosMsg,
    Env,
    Extern,
    HandleResponse,
    HumanAddr,
    Querier,
    StdError,
    StdResult,
    Storage,
    Uint128,
};
use secret_toolkit::{
    snip20::{
        allowance_query,
        balance_query,
        decrease_allowance_msg,
        increase_allowance_msg,
        register_receive_msg,
        set_viewing_key_msg,
    },
};

use shade_protocol::{
    contract_interfaces::{
        dao::treasury::{
            Allowance,
            Config,
            HandleAnswer,
            Manager,
            storage::*,
        },
        snip20,
    },
    utils::{
        asset::{Contract, set_allowance},
        cycle::{exceeds_cycle, parse_utc_datetime},
        generic_response::ResponseStatus,
    },
};

use chrono::prelude::*;
use shade_protocol::contract_interfaces::dao::adapter;
use std::collections::HashMap;

pub fn receive<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    sender: HumanAddr,
    _from: HumanAddr,
    _amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<HandleResponse> {
    //let _key = sender.as_str().as_bytes();

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
    let cur_config = CONFIG.load(&deps.storage)?;

    if env.message.sender != cur_config.admin {
        return Err(StdError::unauthorized());
    }

    CONFIG.save(&mut deps.storage, &config)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn allowance_last_refresh<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    _env: &Env,
    allowance: &Allowance,
) -> StdResult<Option<DateTime<Utc>>> {
    // Parse previous refresh datetime
    let rfc3339 = match allowance {
        Allowance::Amount { last_refresh, .. } => last_refresh,
        Allowance::Portion { last_refresh, .. } => last_refresh,
    };

    DateTime::parse_from_rfc3339(&rfc3339)
        .map(|dt| Some(dt.with_timezone(&Utc)))
        .map_err(|_| StdError::generic_err(format!("Failed to parse datetime {}", rfc3339)))
}

pub fn rebalance<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: HumanAddr,
) -> StdResult<HandleResponse> {
    let naive = NaiveDateTime::from_timestamp(env.block.time as i64, 0);
    let now: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    let key = VIEWING_KEY.load(&deps.storage)?;
    let self_address = SELF_ADDRESS.load(&deps.storage)?;
    let mut messages = vec![];

    let full_asset = match ASSETS.may_load(&deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Not an asset"));
        }
    };

    let allowances = ALLOWANCES.load(&deps.storage, asset.clone())?;

    let token_balance = balance_query(
        &deps.querier,
        self_address,
        key.clone(),
        1,
        full_asset.contract.code_hash.clone(),
        full_asset.contract.address.clone(),
    )?
    .amount;

    /*
    let unbonding = unbonding_r(&deps.storage).load(&asset.as_str().as_bytes())?;
    if unbonding > balance {
        balance = Uint128::zero();
    }
    else {
        balance = (balance - unbonding)?;
    }
    */


    let managers = MANAGERS.load(&deps.storage)?;

    // manager_addr: (balance, allowance)
    let mut manager_data: HashMap<HumanAddr, (Uint128, Uint128)> = HashMap::new();

    // Total amount of funds that are "out" or allocated to an adapter (sky, scrt_staking)
    let mut out_balance = Uint128::zero();

    // Fetch balances & allowances
    for manager in managers.clone() {

        let balance = adapter::balance_query(
            &deps,
            &full_asset.contract.address.clone(),
            manager.contract.clone(),
        )?;
        out_balance += balance;

        let allowance = allowance_query(
            &deps.querier,
            env.contract.address.clone(),
            manager.contract.address.clone(),
            key.clone(),
            1,
            full_asset.contract.code_hash.clone(),
            full_asset.contract.address.clone(),
        )?
        .allowance;

        manager_data.insert(manager.contract.address, (balance, allowance));
    }

    // Total for "amount" allowances (govt, assemblies, etc.)
    let mut amount_total = Uint128::zero();

    MANAGERS.save(&mut deps.storage, &managers)?;
    //let _config = CONFIG.load(&deps.storage)?;

    let (
        amount_allowances, 
        portion_allowances
    ): (Vec<Allowance>, Vec<Allowance>) = allowances
        .into_iter()
        .partition(|a| match a { 
            Allowance::Amount { .. } => true, 
            Allowance::Portion { .. } => false 
        });

    for allowance in amount_allowances {
        match allowance {
            // TODO: change this to a "flag" instead of type
            Allowance::Amount {
                spender,
                cycle,
                amount,
                last_refresh,
            } => {
                let datetime = parse_utc_datetime(&last_refresh)?;

                // Refresh allowance if cycle is exceeded
                if exceeds_cycle(&datetime, &now, cycle) {

                    let mut cur_allowance = Uint128::zero();
                    if let Some(m) = manager_data.get(&spender) {
                        cur_allowance = m.1;
                    }
                    else {
                        cur_allowance = allowance_query(
                            &deps.querier,
                            env.contract.address.clone(),
                            spender.clone(),
                            key.clone(),
                            1,
                            full_asset.contract.code_hash.clone(),
                            full_asset.contract.address.clone(),
                        )?.allowance;

                        // hasn't been accounted for by manager data
                        amount_total += cur_allowance;
                    }

                    amount_total += cur_allowance;

                    match amount.cmp(&cur_allowance) {
                        // Decrease Allowance
                        std::cmp::Ordering::Less => {
                            messages.push(
                                decrease_allowance_msg(
                                    spender.clone(),
                                    (cur_allowance - amount)?,
                                    None,
                                    None,
                                    1,
                                    full_asset.contract.code_hash.clone(),
                                    full_asset.contract.address.clone(),
                                )?
                            );
                        },
                        // Increase Allowance
                        std::cmp::Ordering::Greater => {
                            messages.push(
                                increase_allowance_msg(
                                    spender.clone(),
                                    (amount - cur_allowance)?,
                                    None,
                                    None,
                                    1,
                                    full_asset.contract.code_hash.clone(),
                                    full_asset.contract.address.clone(),
                                )?
                            );
                        },
                        _ => {},
                    }
                }
            }
            _ => {}
        }
    }

    // Total for "portion" allowances (managers for farming mostly & reallocating)
    let portion_total = ((token_balance + out_balance) - amount_total)?;

    for allowance in portion_allowances {
        match allowance {
            Allowance::Portion {
                spender,
                portion,
                last_refresh: _,
                tolerance,
            } => {
                let desired_amount = portion_total.multiply_ratio(portion, 10u128.pow(18));
                let threshold = desired_amount.multiply_ratio(tolerance, 10u128.pow(18));

                let adapter = managers
                    .clone()
                    .into_iter()
                    .find(|m| m.contract.address == spender)
                    .unwrap();

                /* NOTE: remove claiming if rebalance tx becomes too heavy
                 * alternatives:
                 *  - separate rebalance & update,
                 *  - update could do an adapter.update on all "children"
                 *  - rebalance can be unique as its not needed as an adapter
                 */
                if adapter::claimable_query(&deps, 
                                            &asset, 
                                            adapter.contract.clone()
                                    )? > Uint128::zero() {
                    messages.push(adapter::claim_msg(
                        asset.clone(),
                        adapter.contract.clone()
                    )?);
                };

                let cur_allowance = allowance_query(
                    &deps.querier,
                    env.contract.address.clone(),
                    spender.clone(),
                    key.clone(),
                    1,
                    full_asset.contract.code_hash.clone(),
                    full_asset.contract.address.clone(),
                )?
                .allowance;

                // UnderFunded
                if cur_allowance + adapter.balance < desired_amount {
                    let increase = (desired_amount - (adapter.balance + cur_allowance))?;
                    if increase < threshold {
                        continue;
                    }
                    messages.push(increase_allowance_msg(
                        spender,
                        increase,
                        None,
                        None,
                        1,
                        full_asset.contract.code_hash.clone(),
                        full_asset.contract.address.clone(),
                    )?);
                }
                // Overfunded
                else if cur_allowance + adapter.balance > desired_amount {
                    let mut decrease = ((adapter.balance + cur_allowance) - desired_amount)?;
                    if decrease < threshold {
                        continue;
                    }

                    // Remove allowance first
                    if cur_allowance > Uint128::zero() {
                        if cur_allowance < decrease {
                            messages.push(decrease_allowance_msg(
                                spender,
                                cur_allowance,
                                None,
                                None,
                                1,
                                full_asset.contract.code_hash.clone(),
                                full_asset.contract.address.clone(),
                            )?);
                            decrease = (decrease - cur_allowance)?;
                        } else {
                            messages.push(decrease_allowance_msg(
                                spender,
                                decrease,
                                None,
                                None,
                                1,
                                full_asset.contract.code_hash.clone(),
                                full_asset.contract.address.clone(),
                            )?);
                            decrease = Uint128::zero();
                        }
                    }

                    // Unbond remaining
                    if decrease > Uint128::zero() {
                        messages.push(adapter::unbond_msg(
                            asset.clone(),
                            decrease,
                            adapter.contract,
                        )?);
                    }
                }
            },
            _ => {},
        }
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Rebalance {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_register_asset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    contract: &Contract,
) -> StdResult<HandleResponse> {
    let config = CONFIG.load(&deps.storage)?;

    if env.message.sender != config.admin {
        return Err(StdError::unauthorized());
    }

    let mut asset_list = ASSET_LIST.load(&deps.storage)?;
    asset_list.push(contract.address.clone());
    ASSET_LIST.save(&mut deps.storage, &asset_list)?;
    /*
    ASSET_LIST.update(&mut deps.storage, |mut list| {
        list.push(contract.address.clone());
        Ok(list)
    })?;
    */

    ASSETS.save(&mut deps.storage,
                contract.address.clone(),
                &snip20::helpers::fetch_snip20(contract, &deps.querier)?,
    )?;

    ALLOWANCES.save(&mut deps.storage, contract.address.clone(), &Vec::new())?;

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
                VIEWING_KEY.load(&deps.storage)?,
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
    let config = CONFIG.load(&deps.storage)?;

    if env.message.sender != config.admin {
        return Err(StdError::unauthorized());
    }

    MANAGERS.update(&mut deps.storage, |mut adapters| {
        if adapters
            .iter()
            .map(|m| m.contract.clone())
            .collect::<Vec<_>>()
            .contains(&contract)
        {
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

    let config = CONFIG.load(&deps.storage)?;

    /* ADMIN ONLY */
    if env.message.sender != config.admin {
        return Err(StdError::unauthorized());
    }

    let full_asset = match ASSETS.may_load(&deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Not an asset"));
        }
    };

    let adapters = MANAGERS.load(&deps.storage)?;

    // Disallow Portion on non-adapters
    match allowance {
        Allowance::Portion { ref spender, .. } => {
            if adapters
                .clone()
                .into_iter()
                .find(|m| m.contract.address == *spender)
                .is_none()
            {
                return Err(StdError::generic_err("Portion allowances to adapters only"));
            }
        }
        _ => {}
    };

    let mut apps = ALLOWANCES.may_load(&deps.storage, asset.clone())?
        .unwrap_or_default();

    let allow_address = allowance_address(&allowance);

    // find any old allowances with the same contract address & sum current allowances in one loop.
    // saves looping twice in the worst case
    // TODO: Remove Reserves if this would be one of those
    let (stale_allowance, cur_allowance_portion) = apps.iter().enumerate().fold(
        (None, 0u128),
        |(stale_allowance, cur_allowances), (idx, a)| {
            if stale_allowance.is_none() && allowance_address(a) == allow_address {
                (Some(idx), cur_allowances)
            } else {
                (
                    stale_allowance,
                    cur_allowances + allowance_portion(a).u128(),
                )
            }
        },
    );

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
    let datetime: DateTime<Utc> = DateTime::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc);

    let spender = match allowance {
        Allowance::Portion {
            spender,
            portion,
            last_refresh: _,
            tolerance,
        } => {
            apps.push(Allowance::Portion {
                spender: spender.clone(),
                portion: portion.clone(),
                last_refresh: datetime.to_rfc3339(),
                tolerance,
            });
            spender
        }
        Allowance::Amount {
            spender,
            cycle,
            amount,
            last_refresh: _,
        } => {
            apps.push(Allowance::Amount {
                spender: spender.clone(),
                cycle: cycle.clone(),
                amount: amount.clone(),
                last_refresh: datetime.to_rfc3339(),
            });
            spender
        }
    };

    ALLOWANCES.save(&mut deps.storage, asset, &apps)?;
    /*
    set_allowance(
        &deps,
        &env,
        spender,
        amount.clone(),
        VIEWING_KEY.load(&deps.storage)?,
        full_asset.contract,
        None,
    )?,
    */

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Allowance {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: &Env,
    asset: HumanAddr,
) -> StdResult<HandleResponse> {

    let key = asset.as_str().as_bytes();

    let managers = MANAGERS.load(&deps.storage)?;
    let allowances = ALLOWANCES.load(&deps.storage, asset.clone())?;

    let mut messages = vec![];

    let mut claimed = Uint128::zero();

    for allowance in allowances {
        match allowance {
            Allowance::Amount { .. } => {}
            Allowance::Portion { spender, .. } => {
                if let Some(manager) = managers.iter().find(|m| m.contract.address == spender) {
                    let claimable =
                        adapter::claimable_query(&deps, &asset.clone(), manager.contract.clone())?;

                    if claimable > Uint128::zero() {
                        messages.push(adapter::claim_msg(asset.clone(), manager.contract.clone())?);
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

    if env.message.sender != CONFIG.load(&deps.storage)?.admin {
        return Err(StdError::unauthorized());
    }

    let managers = MANAGERS.load(&deps.storage)?;

    let mut messages = vec![];

    let mut unbond_amount = amount;
    let mut unbonded = Uint128::zero();

    for allowance in ALLOWANCES.load(&deps.storage, asset.clone())? {
        match allowance {
            Allowance::Amount { .. } => {}
            Allowance::Portion { spender, .. } => {
                if let Some(manager) = managers.iter().find(|m| m.contract.address == spender) {
                    let unbondable = adapter::unbondable_query(&deps, &asset.clone(), manager.contract.clone())?;

                    if unbondable > unbond_amount {
                        messages.push(
                            adapter::unbond_msg(
                                asset.clone(),
                                unbond_amount,
                                manager.contract.clone(),
                            )?
                        );
                        unbond_amount = Uint128::zero();
                        unbonded = unbond_amount;
                    }
                    else {
                        messages.push(
                            adapter::unbond_msg(
                                asset.clone(),
                                unbondable,
                                manager.contract.clone(),
                            )?
                        );
                        unbond_amount = (unbond_amount - unbondable)?;
                        unbonded = unbonded + unbondable;
                    }
                }
            }
        }

        if unbond_amount == Uint128::zero() {
            break;
        }
    }

    // TODO: Shouldn't be an error, need to log somehow
    if unbond_amount > Uint128::zero() {
        return Err(StdError::generic_err(format!(
            "Failed to fully unbond {}, {} available",
            amount,
            (amount - unbond_amount)?
        )));
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Unbond {
            status: ResponseStatus::Success,
            amount,
        })?),
    })
}
