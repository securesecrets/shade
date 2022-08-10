use shade_protocol::{
    c_std::{
        self, to_binary, Addr, Api, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Querier,
        Response, StdError, StdResult, Storage, Uint128,
    },
    contract_interfaces::{
        admin::{validate_admin, AdminPermissions},
        dao::{
            manager,
            treasury::{Allowance, Config, ExecuteAnswer, Manager},
        },
        snip20,
    },
    snip20::helpers::{
        allowance_query, balance_query, decrease_allowance_msg, increase_allowance_msg,
        register_receive, set_viewing_key_msg,
    },
    utils::{
        asset::{set_allowance, Contract},
        cycle::{exceeds_cycle, parse_utc_datetime},
        generic_response::ResponseStatus,
    },
};

use crate::storage::*;

use chrono::prelude::*;
use std::collections::HashMap;

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Addr,
    _from: Addr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<Response> {
    let key = sender.as_str().as_bytes();

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Receive {
        status: ResponseStatus::Success,
    })?))
}

pub fn try_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: Config,
) -> StdResult<Response> {
    let cur_config = CONFIG.load(deps.storage)?;

    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryAdmin,
        &info.sender,
        &env.contract.address,
        &cur_config.admin_auth,
    )?;

    CONFIG.save(deps.storage, &config)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn allowance_last_refresh(
    deps: Deps,
    env: &Env,
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

pub fn rebalance(deps: DepsMut, env: &Env, asset: Addr) -> StdResult<Response> {
    let naive = NaiveDateTime::from_timestamp(env.block.time.seconds() as i64, 0);
    let now: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    //let asset = deps.api.addr_validate(asset.as_str())?;
    let viewing_key = VIEWING_KEY.load(deps.storage)?;
    let self_address = SELF_ADDRESS.load(deps.storage)?;
    let mut messages = vec![];

    let full_asset = match ASSETS.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Not an asset"));
        }
    };

    let allowances = ALLOWANCES.load(deps.storage, asset.clone())?;

    let token_balance = balance_query(
        &deps.querier,
        self_address.clone(),
        viewing_key.clone(),
        &full_asset.contract.clone(),
    )?;

    /*
    let unbonding = unbonding_r(deps.storage).load(&asset.as_str().as_bytes())?;
    if unbonding > balance {
        balance = Uint128::zero();
    }
    else {
        balance = (balance - unbonding)?;
    }
    */

    let managers = MANAGERS.load(deps.storage)?;

    // manager_addr: (balance, allowance)
    let mut manager_data: HashMap<Addr, (Uint128, Uint128)> = HashMap::new();

    // Total amount of funds that are "out" or allocated to an manager (sky, scrt_staking)
    let mut out_balance = Uint128::zero();

    // Fetch balances & allowances
    for manager in managers.clone() {
        let balance = manager::balance_query(
            deps.querier,
            &full_asset.contract.address.clone(),
            self_address.clone(),
            manager.contract.clone(),
        )?;
        out_balance += balance;

        let allowance = allowance_query(
            &deps.querier,
            env.contract.address.clone(),
            manager.contract.address.clone(),
            viewing_key.clone(),
            1,
            &full_asset.contract.clone(),
        )?
        .allowance;

        manager_data.insert(manager.contract.address, (balance, allowance));
    }

    // Total for "amount" allowances (govt, assemblies, etc.)
    let mut amount_total = Uint128::zero();

    MANAGERS.save(deps.storage, &managers)?;
    //let _config = CONFIG.load(deps.storage)?;

    let (amount_allowances, portion_allowances): (Vec<Allowance>, Vec<Allowance>) =
        allowances.into_iter().partition(|a| match a {
            Allowance::Amount { .. } => true,
            Allowance::Portion { .. } => false,
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
                    } else {
                        cur_allowance = allowance_query(
                            &deps.querier,
                            env.contract.address.clone(),
                            spender.clone(),
                            viewing_key.clone(),
                            1,
                            &full_asset.contract.clone(),
                        )?
                        .allowance;

                        // hasn't been accounted for by manager data
                        amount_total += cur_allowance;
                    }

                    amount_total += cur_allowance;

                    match amount.cmp(&cur_allowance) {
                        // Decrease Allowance
                        std::cmp::Ordering::Less => {
                            messages.push(decrease_allowance_msg(
                                spender.clone(),
                                cur_allowance - amount,
                                //TODO impl expiration
                                None,
                                None,
                                1,
                                &full_asset.contract.clone(),
                                vec![],
                            )?);
                        }
                        // Increase Allowance
                        std::cmp::Ordering::Greater => {
                            messages.push(increase_allowance_msg(
                                spender.clone(),
                                amount - cur_allowance,
                                None,
                                None,
                                1,
                                &full_asset.contract.clone(),
                                vec![],
                            )?);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    // Total for "portion" allowances (managers for farming mostly & reallocating)
    let portion_total = (token_balance + out_balance) - amount_total;

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

                let manager = managers
                    .clone()
                    .into_iter()
                    .find(|m| m.contract.address == spender)
                    .unwrap();

                /* NOTE: remove claiming if rebalance tx becomes too heavy
                 * alternatives:
                 *  - separate rebalance & update,
                 *  - update could do an manager.update on all "children"
                 *  - rebalance can be unique as its not needed as an manager
                 */
                if manager::claimable_query(
                    deps.querier,
                    &asset,
                    self_address.clone(),
                    manager.contract.clone(),
                )? > Uint128::zero()
                {
                    messages.push(manager::claim_msg(&asset, manager.contract.clone())?);
                };

                let cur_allowance = allowance_query(
                    &deps.querier,
                    env.contract.address.clone(),
                    spender.clone(),
                    viewing_key.clone(),
                    1,
                    &full_asset.contract.clone(),
                )?
                .allowance;

                // UnderFunded
                if cur_allowance + manager.balance < desired_amount {
                    let increase = desired_amount - (manager.balance + cur_allowance);
                    if increase < threshold {
                        continue;
                    }
                    messages.push(increase_allowance_msg(
                        spender.clone(),
                        increase,
                        None,
                        None,
                        1,
                        &full_asset.contract.clone(),
                        vec![],
                    )?);
                }
                // Overfunded
                else if cur_allowance + manager.balance > desired_amount {
                    let mut decrease = (manager.balance + cur_allowance) - desired_amount;
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
                                &full_asset.contract.clone(),
                                vec![],
                            )?);
                            decrease = decrease - cur_allowance;
                        } else {
                            messages.push(decrease_allowance_msg(
                                spender,
                                decrease,
                                None,
                                None,
                                1,
                                &full_asset.contract.clone(),
                                vec![],
                            )?);
                            decrease = Uint128::zero();
                        }
                    }

                    // Unbond remaining
                    if decrease > Uint128::zero() {
                        messages.push(manager::unbond_msg(&asset, decrease, manager.contract)?);
                    }
                }
            }
            _ => {}
        }
    }

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::Rebalance {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn try_register_asset(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    contract: &Contract,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryAdmin,
        &info.sender,
        &env.contract.address,
        &config.admin_auth,
    )?;

    let mut asset_list = ASSET_LIST.load(deps.storage)?;
    asset_list.push(contract.address.clone());
    ASSET_LIST.save(deps.storage, &asset_list)?;
    /*
    ASSET_LIST.update(deps.storage, |mut list| {
        list.push(contract.address.clone());
        Ok(list)
    })?;
    */

    ASSETS.save(
        deps.storage,
        contract.address.clone(),
        &snip20::helpers::fetch_snip20(contract, &deps.querier)?,
    )?;

    ALLOWANCES.save(deps.storage, contract.address.clone(), &Vec::new())?;

    Ok(Response::new()
        .add_message(
            // Register contract in asset
            register_receive(env.contract.code_hash.clone(), None, contract)?,
        )
        .add_message(
            // Set viewing key
            set_viewing_key_msg(VIEWING_KEY.load(deps.storage)?, None, &contract.clone())?,
        )
        .set_data(to_binary(&ExecuteAnswer::RegisterAsset {
            status: ResponseStatus::Success,
        })?))
}

pub fn register_manager(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    contract: &mut Contract,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryAdmin,
        &info.sender,
        &env.contract.address,
        &config.admin_auth,
    )?;

    MANAGERS.update(deps.storage, |mut managers| {
        if managers
            .iter()
            .map(|m| m.contract.clone())
            .collect::<Vec<_>>()
            .contains(&contract)
        {
            return Err(StdError::generic_err("Manager already registered"));
        }
        managers.push(Manager {
            contract: contract.clone(),
            balance: Uint128::zero(),
            desired: Uint128::zero(),
        });
        Ok(managers)
    })?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::RegisterAsset {
            status: ResponseStatus::Success,
        })?),
    )
}

// extract contract address if any
fn allowance_address(allowance: &Allowance) -> Option<&Addr> {
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

pub fn allowance(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    asset: Addr,
    allowance: Allowance,
) -> StdResult<Response> {
    static ONE_HUNDRED_PERCENT: u128 = 10u128.pow(18);

    let config = CONFIG.load(deps.storage)?;
    //let asset = deps.api.addr_validate(asset.as_str())?;
    /* ADMIN ONLY */
    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryAdmin,
        &info.sender,
        &env.contract.address,
        &config.admin_auth,
    )?;

    let full_asset = match ASSETS.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Not an asset"));
        }
    };

    let managers = MANAGERS.load(deps.storage)?;

    // Disallow Portion on non-managers
    match allowance {
        Allowance::Portion { ref spender, .. } => {
            if managers
                .clone()
                .into_iter()
                .find(|m| m.contract.address == *spender)
                .is_none()
            {
                return Err(StdError::generic_err("Portion allowances to managers only"));
            }
        }
        _ => {}
    };

    let mut apps = ALLOWANCES
        .may_load(deps.storage, asset.clone())?
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

    ALLOWANCES.save(deps.storage, asset, &apps)?;
    /*
    set_allowance(
        &deps,
        &env,
        spender,
        amount.clone(),
        VIEWING_KEY.load(deps.storage)?,
        full_asset.contract,
        None,
    )?,
    */

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::Allowance {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn claim(deps: DepsMut, _env: &Env, info: MessageInfo, asset: Addr) -> StdResult<Response> {
    //let asset = deps.api.addr_validate(asset.as_str())?;
    let managers = MANAGERS.load(deps.storage)?;
    let allowances = ALLOWANCES.load(deps.storage, asset.clone())?;
    let self_address = SELF_ADDRESS.load(deps.storage)?;

    let mut messages = vec![];

    let mut claimed = Uint128::zero();

    for manager in managers {
        let claimable = manager::claimable_query(
            deps.querier,
            &asset,
            self_address.clone(),
            manager.contract.clone(),
        )?;

        if claimable > Uint128::zero() {
            messages.push(manager::claim_msg(&asset, manager.contract.clone())?);
            claimed += claimable;
        }
    }

    Ok(
        Response::new().set_data(to_binary(&manager::ExecuteAnswer::Claim {
            status: ResponseStatus::Success,
            amount: claimed,
        })?),
    )
}

pub fn unbond(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    asset: Addr,
    amount: Uint128,
) -> StdResult<Response> {
    //let asset = deps.api.addr_validate(asset.as_str())?;
    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryAdmin,
        &info.sender,
        &env.contract.address,
        &CONFIG.load(deps.storage)?.admin_auth,
    )?;

    let managers = MANAGERS.load(deps.storage)?;
    let self_address = SELF_ADDRESS.load(deps.storage)?;

    let mut messages = vec![];

    let mut unbond_amount = amount;
    let mut unbonded = Uint128::zero();

    for allowance in ALLOWANCES.load(deps.storage, asset.clone())? {
        match allowance {
            Allowance::Amount { .. } => {}
            Allowance::Portion { spender, .. } => {
                if let Some(manager) = managers.iter().find(|m| m.contract.address == spender) {
                    let unbondable = manager::unbondable_query(
                        deps.querier,
                        &asset,
                        self_address.clone(),
                        manager.contract.clone(),
                    )?;

                    if unbondable > unbond_amount {
                        messages.push(manager::unbond_msg(
                            &asset,
                            unbond_amount,
                            manager.contract.clone(),
                        )?);
                        unbond_amount = Uint128::zero();
                        unbonded = unbond_amount;
                    } else {
                        messages.push(manager::unbond_msg(
                            &asset,
                            unbondable,
                            manager.contract.clone(),
                        )?);
                        unbond_amount = unbond_amount - unbondable;
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
            amount - unbond_amount
        )));
    }

    Ok(
        Response::new().set_data(to_binary(&manager::ExecuteAnswer::Unbond {
            status: ResponseStatus::Success,
            amount,
        })?),
    )
}
