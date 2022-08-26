use shade_protocol::{
    c_std::{
        self,
        to_binary,
        Addr,
        Api,
        Binary,
        CosmosMsg,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Querier,
        Response,
        StdError,
        StdResult,
        Storage,
        Uint128,
    },
    contract_interfaces::{
        admin::helpers::{validate_admin, AdminPermissions},
        dao::{
            manager,
            treasury::{
                Action,
                Allowance,
                AllowanceMeta,
                AllowanceType,
                Config,
                Context,
                ExecuteAnswer,
                Metric,
                RunLevel,
            },
        },
        snip20,
    },
    snip20::helpers::{
        allowance_query,
        balance_query,
        decrease_allowance_msg,
        increase_allowance_msg,
        register_receive,
        send_msg,
        set_viewing_key_msg,
    },
    utils::{
        asset::{set_allowance, Contract},
        cycle::{exceeds_cycle, parse_utc_datetime, utc_from_seconds, utc_from_timestamp, utc_now},
        generic_response::ResponseStatus,
        storage::plus::period_storage::PeriodStorage,
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
    _msg: Option<Binary>,
) -> StdResult<Response> {
    METRICS.pushf(deps.storage, env.block.time, Metric {
        action: Action::FundsReceived,
        context: Context::Receive,
        timestamp: env.block.time.seconds(),
        token: info.sender,
        amount,
        user: sender,
    })?;

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
        &cur_config.admin_auth,
    )?;

    CONFIG.save(deps.storage, &config)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn update(deps: DepsMut, env: &Env, info: MessageInfo, asset: Addr) -> StdResult<Response> {
    println!("UPDATE");
    match RUN_LEVEL.load(deps.storage)? {
        RunLevel::Migrating => {
            println!("MIGRATING");
            migrate(deps, env, info, asset)
        }
        RunLevel::Deactivated => {
            return Err(StdError::generic_err("Contract Deactivated"));
        }
        RunLevel::Normal => {
            println!("REBALANCING");
            rebalance(deps, env, info, asset)
        }
    }
}

pub fn rebalance(deps: DepsMut, env: &Env, info: MessageInfo, asset: Addr) -> StdResult<Response> {
    println!("\n\t\t\t\t\tTREASURY REBALANCE\n");
    let viewing_key = VIEWING_KEY.load(deps.storage)?;
    let self_address = SELF_ADDRESS.load(deps.storage)?;

    let full_asset = match ASSET.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Not an asset"));
        }
    };

    let mut allowances = ALLOWANCES.load(deps.storage, asset.clone())?;

    let mut token_balance = balance_query(
        &deps.querier,
        self_address.clone(),
        viewing_key.clone(),
        &full_asset.contract.clone(),
    )?;

    // Total for "amount" allowances (govt, assemblies, etc.)
    let mut amount_balance = Uint128::zero();
    let mut amount_unbonding = Uint128::zero();
    let mut amount_allowance = Uint128::zero();

    // Total for "portion" allowances
    let mut portion_balance = Uint128::zero();
    let mut portion_unbonding = Uint128::zero();
    let mut portion_allowance = Uint128::zero();

    // { spender: (balance, allowance) }
    let mut metadata: HashMap<Addr, (Uint128, Uint128)> = HashMap::new();

    let mut messages = vec![];
    let mut metrics = vec![];

    let now = utc_now(&env);

    let mut stale_allowances = vec![];

    for (i, a) in allowances.clone().iter().enumerate() {
        let manager = MANAGER.may_load(deps.storage, a.spender.clone())?;
        let mut claimable = Uint128::zero();
        let mut unbonding = Uint128::zero();
        let mut unbondable = Uint128::zero();
        if let Some(m) = manager.clone() {
            claimable = manager::claimable_query(
                deps.querier,
                &asset.clone(),
                env.contract.address.clone(),
                m.clone(),
            )?;
            println!("CLAIMABLE {}", claimable);

            unbonding = manager::unbonding_query(
                deps.querier,
                &asset.clone(),
                env.contract.address.clone(),
                m.clone(),
            )?;

            unbondable = manager::unbondable_query(
                deps.querier,
                &asset.clone(),
                env.contract.address.clone(),
                m.clone(),
            )?;

            if !claimable.is_zero() {
                println!("CLAIM MSG HERE");
                messages.push(manager::claim_msg(&asset.clone(), m.clone())?);
                metrics.push(Metric {
                    action: Action::ManagerClaim,
                    context: Context::Rebalance,
                    timestamp: env.block.time.seconds(),
                    token: asset.clone(),
                    amount: claimable,
                    user: m.address,
                });
            }
        }
        let balance = match manager {
            Some(m) => manager::balance_query(
                deps.querier,
                &asset.clone(),
                env.contract.address.clone(),
                m,
            )?,
            None => Uint128::zero(),
        };

        let allowance = allowance_query(
            &deps.querier,
            env.contract.address.clone(),
            a.spender.clone(),
            viewing_key.clone(),
            1,
            &full_asset.contract.clone(),
        )?
        .allowance;

        if balance.is_zero()
            && unbonding.is_zero()
            && unbondable.is_zero()
            && claimable.is_zero()
            && allowance.is_zero()
            && a.amount.is_zero()
        {
            stale_allowances.push(i);
        }

        metadata.insert(a.spender.clone(), (balance, allowance));

        match a.allowance_type {
            AllowanceType::Amount => {
                amount_balance += balance;
                amount_unbonding += unbonding;
                amount_allowance += allowance;
            }
            AllowanceType::Portion => {
                portion_balance += balance;
                portion_unbonding += unbonding;
                portion_allowance += allowance;
            }
        }
    }
    if !stale_allowances.is_empty() {
        for index in stale_allowances.iter().rev() {
            allowances.remove(index.clone());
        }
        ALLOWANCES.save(deps.storage, asset.clone(), &allowances)?;
    }
    let mut total_balance =
        token_balance + portion_balance + amount_balance + amount_unbonding + portion_unbonding;

    println!("token balance {}", total_balance);
    let portions = allowances
        .clone()
        .into_iter()
        .filter(|a| a.allowance_type == AllowanceType::Portion)
        .collect::<Vec<AllowanceMeta>>();
    let amounts = allowances
        .clone()
        .into_iter()
        .filter(|a| a.allowance_type == AllowanceType::Amount)
        .collect::<Vec<AllowanceMeta>>();

    //TODO switch to partition
    //let (amount_allowances, portion_allowances) = allowances.iter().partition(|

    // Iterate amount allows first to determine portion total
    for allowance in amounts {
        println!("AMOUNT ALLOW");
        total_balance -= allowance.amount;
        let last_refresh = parse_utc_datetime(&allowance.last_refresh)?;
        // Claim from managers
        let manager = MANAGER.may_load(deps.storage, allowance.spender.clone())?;

        // Refresh allowance if cycle is exceeded
        if !exceeds_cycle(&last_refresh, &now, allowance.cycle.clone()) {
            continue;
        }

        let (balance, cur_allowance) = metadata[&allowance.spender];

        let threshold = allowance
            .amount
            .multiply_ratio(allowance.tolerance, 10u128.pow(18));

        match allowance.amount.cmp(&(cur_allowance + balance)) {
            // Decrease Allowance
            std::cmp::Ordering::Less => {
                let mut decrease = (cur_allowance + balance) - allowance.amount;
                if decrease <= threshold {
                    continue;
                }
                messages.push(decrease_allowance_msg(
                    allowance.spender.clone(),
                    decrease,
                    //TODO impl expiration
                    None,
                    None,
                    1,
                    &full_asset.contract.clone(),
                    vec![],
                )?);
                metrics.push(Metric {
                    action: Action::DecreaseAllowance,
                    context: Context::Rebalance,
                    timestamp: env.block.time.seconds(),
                    token: asset.clone(),
                    amount: decrease,
                    user: allowance.spender.clone(),
                });
                if decrease > amount_allowance {
                    amount_allowance = Uint128::zero();
                } else {
                    amount_allowance -= decrease;
                }
                if decrease > cur_allowance {
                    match MANAGER.may_load(deps.storage, allowance.spender)? {
                        Some(m) => {
                            messages.push(manager::unbond_msg(
                                &asset.clone(),
                                decrease - cur_allowance,
                                m.clone(),
                            )?);
                            metrics.push(Metric {
                                action: Action::ManagerUnbond,
                                context: Context::Rebalance,
                                timestamp: env.block.time.seconds(),
                                token: asset.clone(),
                                amount: decrease - cur_allowance,
                                user: m.address.clone(),
                            });
                        }
                        None => {}
                    }
                }
            }
            // Increase Allowance
            std::cmp::Ordering::Greater => {
                let increase = allowance.amount - (cur_allowance + balance);
                if increase <= threshold {
                    continue;
                }
                messages.push(increase_allowance_msg(
                    allowance.spender.clone(),
                    increase,
                    None,
                    None,
                    1,
                    &full_asset.contract.clone(),
                    vec![],
                )?);
                metrics.push(Metric {
                    action: Action::IncreaseAllowance,
                    context: Context::Rebalance,
                    timestamp: env.block.time.seconds(),
                    token: asset.clone(),
                    amount: increase,
                    user: allowance.spender.clone(),
                });
                amount_allowance += increase;
            }
            _ => {}
        }
    }

    let mut portion_total = portion_balance; // + (token_balance - amount_allowance);
    if amount_allowance > token_balance {
        portion_total -= amount_allowance - token_balance;
    } else {
        portion_total += token_balance - amount_allowance;
    }
    if total_balance > portion_total {
        portion_total = total_balance;
    }

    for allowance in portions {
        println!("portion total {}", portion_total);
        let last_refresh = parse_utc_datetime(&allowance.last_refresh)?;
        if !exceeds_cycle(&last_refresh, &now, allowance.cycle.clone()) {
            continue;
        }
        // Claim from managers
        let desired_amount = portion_total.multiply_ratio(allowance.amount, 10u128.pow(18));
        let threshold = desired_amount.multiply_ratio(allowance.tolerance, 10u128.pow(18));

        /* NOTE: remove claiming if rebalance tx becomes too heavy
         * alternatives:
         *  - separate rebalance & update,
         *  - update could do an manager.update on all "children"
         *  - rebalance can be unique as its not needed as an manager
         */

        let (balance, cur_allowance) = metadata[&allowance.spender];
        let total = balance + cur_allowance;
        println!("TOTAL: {}, DESIRED: {}", total, desired_amount);

        // UnderFunded
        if total < desired_amount {
            let increase = desired_amount - total;
            if increase <= threshold {
                continue;
            }
            messages.push(increase_allowance_msg(
                allowance.spender.clone(),
                increase,
                None,
                None,
                1,
                &full_asset.contract.clone(),
                vec![],
            )?);
            metrics.push(Metric {
                action: Action::IncreaseAllowance,
                context: Context::Rebalance,
                timestamp: env.block.time.seconds(),
                token: asset.clone(),
                amount: increase,
                user: allowance.spender.clone(),
            });
        }
        // Overfunded
        else if total > desired_amount {
            let mut decrease = total - desired_amount;

            if decrease <= threshold {
                continue;
            }

            // need to remove more than allowance
            if cur_allowance < decrease {
                messages.push(decrease_allowance_msg(
                    allowance.spender.clone(),
                    cur_allowance,
                    None,
                    None,
                    1,
                    &full_asset.contract.clone(),
                    vec![],
                )?);
                metrics.push(Metric {
                    action: Action::DecreaseAllowance,
                    context: Context::Rebalance,
                    timestamp: env.block.time.seconds(),
                    token: asset.clone(),
                    amount: cur_allowance,
                    user: allowance.spender.clone(),
                });

                decrease -= cur_allowance;

                // Unbond remaining
                if !decrease.is_zero() {
                    match MANAGER.may_load(deps.storage, allowance.spender.clone())? {
                        Some(m) => {
                            messages.push(manager::unbond_msg(
                                &asset.clone(),
                                decrease,
                                m.clone(),
                            )?);
                            metrics.push(Metric {
                                action: Action::ManagerUnbond,
                                context: Context::Rebalance,
                                timestamp: env.block.time.seconds(),
                                token: asset.clone(),
                                amount: decrease,
                                user: m.address.clone(),
                            });
                        }
                        None => {
                            return Err(StdError::generic_err(format!(
                                "Can't unbond from non-manager {}",
                                allowance.spender.clone()
                            )));
                        }
                    }
                }
            } else {
                messages.push(decrease_allowance_msg(
                    allowance.spender.clone(),
                    decrease,
                    None,
                    None,
                    1,
                    &full_asset.contract.clone(),
                    vec![],
                )?);
                metrics.push(Metric {
                    action: Action::DecreaseAllowance,
                    context: Context::Rebalance,
                    timestamp: env.block.time.seconds(),
                    token: asset.clone(),
                    amount: decrease,
                    user: allowance.spender.clone(),
                });
            }
        }
    }

    METRICS.append(deps.storage, env.block.time, &mut metrics)?;
    METRICS.flush(deps.storage)?;

    Ok(Response::new()
        .add_messages(messages)
        .set_data(to_binary(&ExecuteAnswer::Rebalance {
            status: ResponseStatus::Success,
        })?))
}

pub fn migrate(deps: DepsMut, env: &Env, info: MessageInfo, asset: Addr) -> StdResult<Response> {
    let mut messages = vec![];
    let mut metrics = vec![];

    let allowances = ALLOWANCES.load(deps.storage, asset.clone())?;
    let full_asset = ASSET.load(deps.storage, asset.clone())?;
    let viewing_key = VIEWING_KEY.load(deps.storage)?;

    let mut claimed = Uint128::zero();

    for allowance in allowances {
        if let Some(m) = MANAGER.may_load(deps.storage, allowance.spender.clone())? {
            let unbondable = manager::unbondable_query(
                deps.querier,
                &asset,
                env.contract.address.clone(),
                m.clone(),
            )?;

            if !unbondable.is_zero() {
                messages.push(manager::unbond_msg(&asset, unbondable, m.clone())?);
                metrics.push(Metric {
                    action: Action::ManagerUnbond,
                    context: Context::Migration,
                    timestamp: env.block.time.seconds(),
                    token: asset.clone(),
                    amount: unbondable,
                    user: m.address.clone(),
                });
            }
            let claimable = manager::claimable_query(
                deps.querier,
                &asset,
                env.contract.address.clone(),
                m.clone(),
            )?;

            if !claimable.is_zero() {
                messages.push(manager::claim_msg(&asset, m.clone())?);
                metrics.push(Metric {
                    action: Action::ManagerClaim,
                    context: Context::Migration,
                    timestamp: env.block.time.seconds(),
                    token: asset.clone(),
                    amount: claimable,
                    user: m.address.clone(),
                });
                claimed += claimable;
            }
        }

        let cur_allowance = allowance_query(
            &deps.querier,
            env.contract.address.clone(),
            allowance.spender.clone(),
            viewing_key.clone(),
            1,
            &full_asset.contract.clone(),
        )?
        .allowance;

        if !cur_allowance.is_zero() {
            messages.push(decrease_allowance_msg(
                allowance.spender.clone(),
                cur_allowance,
                None,
                None,
                1,
                &full_asset.contract.clone(),
                vec![],
            )?);
            metrics.push(Metric {
                action: Action::DecreaseAllowance,
                context: Context::Migration,
                timestamp: env.block.time.seconds(),
                token: asset.clone(),
                amount: cur_allowance,
                user: allowance.spender.clone(),
            });
        }
    }

    let balance = balance_query(
        &deps.querier,
        env.contract.address.clone(),
        viewing_key.clone(),
        &full_asset.contract.clone(),
    )?;

    if !(balance + claimed).is_zero() {
        let config = CONFIG.load(deps.storage)?;

        //TODO: send to super admin from admin_auth
        messages.push(send_msg(
            config.multisig.clone(),
            balance + claimed,
            None,
            None,
            None,
            &full_asset.contract.clone(),
        )?);
        metrics.push(Metric {
            action: Action::SendFunds,
            context: Context::Migration,
            timestamp: env.block.time.seconds(),
            token: asset.clone(),
            amount: balance + claimed,
            user: config.multisig.clone(),
        });
    }

    METRICS.append(deps.storage, env.block.time, &mut metrics)?;
    METRICS.flush(deps.storage)?;

    Ok(Response::new()
        .add_messages(messages)
        .set_data(to_binary(&ExecuteAnswer::Migration {
            status: ResponseStatus::Success,
        })?))
}

pub fn set_run_level(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    run_level: RunLevel,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    println!("Setting Run Level");

    // TODO force super-admin
    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryAdmin,
        &info.sender,
        &config.admin_auth,
    )?;

    RUN_LEVEL.save(deps.storage, &run_level)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::RunLevel { run_level })?))
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
        &config.admin_auth,
    )?;

    ASSET_LIST.push(deps.storage, &contract.address.clone())?;
    /*
    asset_list.push(contract.address.clone());
    ASSET_LIST.save(deps.storage, &asset_list)?;
    */

    ASSET.save(
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
        AdminPermissions::TreasuryManager,
        &info.sender,
        &config.admin_auth,
    )?;

    if let Some(m) = MANAGER.may_load(deps.storage, contract.address.clone())? {
        return Err(StdError::generic_err("Manager already registered"));
    } else {
        MANAGER.save(deps.storage, contract.address.clone(), &contract)?;
    }

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::RegisterAsset {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn allowance(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    asset: Addr,
    allowance: Allowance,
) -> StdResult<Response> {
    println!(
        "TREASURY ALLOWANCE {}, {}",
        allowance.amount,
        allowance.allowance_type == AllowanceType::Portion,
    );
    let config = CONFIG.load(deps.storage)?;
    /* ADMIN ONLY */
    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryAdmin,
        &info.sender,
        &config.admin_auth,
    )?;

    let viewing_key = VIEWING_KEY.load(deps.storage)?;
    let full_asset = match ASSET.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Not an asset"));
        }
    };

    let mut allowances = ALLOWANCES
        .may_load(deps.storage, asset.clone())?
        .unwrap_or(vec![]);

    let last_refresh: DateTime<Utc> = utc_from_seconds(0);

    let stale_allow = allowances
        .iter()
        .position(|a| a.spender == allowance.spender);

    match stale_allow {
        Some(i) => {
            allowances.swap_remove(i);
        }
        None => {}
    };

    allowances.push(AllowanceMeta {
        spender: allowance.spender.clone(),
        amount: allowance.amount,
        cycle: allowance.cycle,
        allowance_type: allowance.allowance_type.clone(),
        // "zero/null" datetime
        last_refresh: last_refresh.to_rfc3339(),
        tolerance: allowance.tolerance,
    });

    let portion_sum: u128 = allowances
        .iter()
        .filter(|a| a.allowance_type == AllowanceType::Portion)
        .map(|a| a.amount.u128())
        .sum();

    if portion_sum > 10u128.pow(18) {
        return Err(StdError::generic_err(format!(
            "Total portion allowances cannot exceed %100 ({})",
            portion_sum
        )));
    }
    ALLOWANCES.save(deps.storage, asset, &allowances)?;

    /*let messages = match allowance.allowance_type {
        AllowanceType::Amount => set_allowance(
            deps,
            env,
            allowance.spender,
            allowance.amount,
            viewing_key,
            &full_asset.contract,
            None,
        )?,
        AllowanceType::Portion => vec![],
    };*/
    let messages: Vec<CosmosMsg> = vec![];

    Ok(Response::new()
        .add_messages(messages)
        .set_data(to_binary(&ExecuteAnswer::Allowance {
            status: ResponseStatus::Success,
        })?))
}

pub fn claim(deps: DepsMut, _env: &Env, info: MessageInfo, asset: Addr) -> StdResult<Response> {
    // TODO iterate manager storage
    let self_address = SELF_ADDRESS.load(deps.storage)?;

    let mut messages = vec![];

    let mut claimed = Uint128::zero();

    for allowance in ALLOWANCES.load(deps.storage, asset.clone())? {
        if let Some(m) = MANAGER.may_load(deps.storage, allowance.spender)? {
            let claimable = manager::claimable_query(
                deps.querier,
                &asset.clone(),
                self_address.clone(),
                m.clone(),
            )?;
            claimed += claimable;

            if !claimable.is_zero() {
                messages.push(manager::claim_msg(&asset, m.clone())?);
            }
        }
    }

    Ok(Response::new().add_messages(messages).set_data(to_binary(
        &manager::ExecuteAnswer::Claim {
            status: ResponseStatus::Success,
            amount: claimed,
        },
    )?))
}

pub fn unbond(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    asset: Addr,
    amount: Uint128,
) -> StdResult<Response> {
    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryAdmin,
        &info.sender,
        &CONFIG.load(deps.storage)?.admin_auth,
    )?;

    let self_address = SELF_ADDRESS.load(deps.storage)?;

    let mut messages = vec![];

    let mut unbond_amount = amount;
    let mut unbonded = Uint128::zero();

    for allowance in ALLOWANCES.load(deps.storage, asset.clone())? {
        if let Some(m) = MANAGER.may_load(deps.storage, allowance.spender)? {
            println!("HERE TERA 884");
            let unbondable =
                manager::unbondable_query(deps.querier, &asset, self_address.clone(), m.clone())?;
            println!("HERE TERA 887");

            if unbondable > unbond_amount {
                messages.push(manager::unbond_msg(&asset, unbond_amount, m.clone())?);
                unbond_amount = Uint128::zero();
                unbonded = unbond_amount;
            } else {
                messages.push(manager::unbond_msg(&asset, unbondable, m)?);
                unbond_amount = unbond_amount - unbondable;
                unbonded = unbonded + unbondable;
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

    Ok(Response::new().add_messages(messages).set_data(to_binary(
        &manager::ExecuteAnswer::Unbond {
            status: ResponseStatus::Success,
            amount,
        },
    )?))
}
