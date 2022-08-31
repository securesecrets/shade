use shade_protocol::{
    c_std::{
        self,
        to_binary,
        Addr,
        Api,
        Binary,
        Coin,
        CosmosMsg,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Querier,
        QuerierWrapper,
        Response,
        StdError,
        StdResult,
        Storage,
        Uint128,
    },
    chrono::prelude::*,
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
        asset::Contract,
        cycle::{exceeds_cycle, parse_utc_datetime, utc_from_seconds, utc_now, Cycle},
        generic_response::ResponseStatus,
        wrap::wrap_coin,
    },
};

use crate::storage::*;

use itertools::{Either, Itertools};
use std::collections::HashMap;

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _sender: Addr,
    from: Addr,
    amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<Response> {
    METRICS.pushf(deps.storage, env.block.time, Metric {
        action: Action::FundsReceived,
        context: Context::Receive,
        timestamp: env.block.time.seconds(),
        token: info.sender,
        amount,
        user: from,
    })?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Receive {
        status: ResponseStatus::Success,
    })?))
}

pub fn try_update_config(
    deps: DepsMut,
    _env: Env,
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
    match RUN_LEVEL.load(deps.storage)? {
        RunLevel::Migrating => migrate(deps, env, info, asset),
        RunLevel::Deactivated => {
            return Err(StdError::generic_err("Contract Deactivated"));
        }
        RunLevel::Normal => rebalance(deps, env, info, asset),
    }
}

pub fn rebalance(deps: DepsMut, env: &Env, _info: MessageInfo, asset: Addr) -> StdResult<Response> {
    let viewing_key = VIEWING_KEY.load(deps.storage)?;
    let self_address = SELF_ADDRESS.load(deps.storage)?;

    let full_asset = match ASSET.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Not an asset"));
        }
    };

    let mut allowances = ALLOWANCES.load(deps.storage, asset.clone())?;

    let token_balance = balance_query(
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
                messages.push(manager::claim_msg(&asset.clone(), m.clone())?);
                metrics.push(Metric {
                    action: Action::Claim,
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

    let (amounts, portions): (Vec<AllowanceMeta>, Vec<AllowanceMeta>) = allowances
        .clone()
        .into_iter()
        .partition_map(|a| match a.allowance_type {
            AllowanceType::Amount => Either::Left(a),
            AllowanceType::Portion => Either::Right(a),
        });

    /* Amounts given priority
     * portions are calculated after amounts are taken from total
     */
    for allowance in amounts {
        total_balance -= allowance.amount;

        let last_refresh = parse_utc_datetime(&allowance.last_refresh)?;
        let manager = MANAGER.may_load(deps.storage, allowance.spender.clone())?;

        // Refresh allowance if cycle is exceeded
        if !exceeds_cycle(&last_refresh, &now, allowance.cycle.clone()) {
            // Once allowances need 1 refresh if last_refresh == 'null'
            // TODO allowance needs to be removed once it is used up
            if allowance.cycle == Cycle::Once {
                if last_refresh != utc_from_seconds(0) {
                    continue;
                }
            } else {
                continue;
            }
        }

        let (balance, cur_allowance) = metadata[&allowance.spender];

        let threshold = allowance
            .amount
            .multiply_ratio(allowance.tolerance, 10u128.pow(18));

        match allowance.amount.cmp(&(cur_allowance + balance)) {
            // Decrease Allowance
            std::cmp::Ordering::Less => {
                let decrease = (cur_allowance + balance) - allowance.amount;
                if decrease <= threshold {
                    continue;
                }
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

                if decrease > amount_allowance {
                    amount_allowance = Uint128::zero();
                } else {
                    amount_allowance -= decrease;
                }

                if decrease > cur_allowance {
                    if let Some(m) = MANAGER.may_load(deps.storage, allowance.spender)? {
                        messages.push(manager::unbond_msg(
                            &asset.clone(),
                            decrease - cur_allowance,
                            m.clone(),
                        )?);
                        metrics.push(Metric {
                            action: Action::Unbond,
                            context: Context::Rebalance,
                            timestamp: env.block.time.seconds(),
                            token: asset.clone(),
                            amount: decrease - cur_allowance,
                            user: m.address.clone(),
                        });
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
        let last_refresh = parse_utc_datetime(&allowance.last_refresh)?;
        if !exceeds_cycle(&last_refresh, &now, allowance.cycle.clone()) {
            continue;
        }

        let desired_amount = portion_total.multiply_ratio(allowance.amount, 10u128.pow(18));
        let threshold = desired_amount.multiply_ratio(allowance.tolerance, 10u128.pow(18));

        let (balance, cur_allowance) = metadata[&allowance.spender];
        let total = balance + cur_allowance;

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

            // Allowance fully covers amount needed
            if cur_allowance >= decrease {
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
            // Reduce allowance then unbond
            else {
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
                    if let Some(m) = MANAGER.may_load(deps.storage, allowance.spender.clone())? {
                        messages.push(manager::unbond_msg(&asset.clone(), decrease, m.clone())?);
                        metrics.push(Metric {
                            action: Action::Unbond,
                            context: Context::Rebalance,
                            timestamp: env.block.time.seconds(),
                            token: asset.clone(),
                            amount: decrease,
                            user: m.address.clone(),
                        });
                    } else {
                        return Err(StdError::generic_err(format!(
                            "Can't unbond from non-manager {}",
                            allowance.spender.clone()
                        )));
                    }
                }
            }
        }
    }

    METRICS.appendf(deps.storage, env.block.time, &mut metrics)?;

    Ok(Response::new()
        .add_messages(messages)
        .set_data(to_binary(&ExecuteAnswer::Rebalance {
            status: ResponseStatus::Success,
        })?))
}

pub fn migrate(deps: DepsMut, env: &Env, _info: MessageInfo, asset: Addr) -> StdResult<Response> {
    let mut messages = vec![];
    let mut metrics = vec![];

    let allowances = ALLOWANCES.load(deps.storage, asset.clone())?;
    let full_asset = ASSET.load(deps.storage, asset.clone())?;
    let viewing_key = VIEWING_KEY.load(deps.storage)?;

    let mut claimed = Uint128::zero();

    for allowance in allowances {
        if let Some(m) = MANAGER.may_load(deps.storage, allowance.spender.clone())? {
            // TODO store in metadata object for re-use
            let unbondable = manager::unbondable_query(
                deps.querier,
                &asset,
                env.contract.address.clone(),
                m.clone(),
            )?;

            // Unbond all if any
            if !unbondable.is_zero() {
                messages.push(manager::unbond_msg(&asset, unbondable, m.clone())?);
                metrics.push(Metric {
                    action: Action::Unbond,
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

            // Claim if any
            if !claimable.is_zero() {
                messages.push(manager::claim_msg(&asset, m.clone())?);
                metrics.push(Metric {
                    action: Action::Claim,
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

        // Reduce all allowance if any
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

    // Send full balance to multisig
    let balance = balance_query(
        &deps.querier,
        env.contract.address.clone(),
        viewing_key.clone(),
        &full_asset.contract.clone(),
    )?;

    if !(balance + claimed).is_zero() {
        let config = CONFIG.load(deps.storage)?;

        //TODO: send to super admin from admin_auth -- remove multisig from config
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

    METRICS.appendf(deps.storage, env.block.time, &mut metrics)?;

    Ok(Response::new()
        .add_messages(messages)
        .set_data(to_binary(&ExecuteAnswer::Migration {
            status: ResponseStatus::Success,
        })?))
}

pub fn set_run_level(
    deps: DepsMut,
    _env: &Env,
    info: MessageInfo,
    run_level: RunLevel,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    // TODO force super-admin?
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

    ASSET.save(
        deps.storage,
        contract.address.clone(),
        &snip20::helpers::fetch_snip20(contract, &deps.querier)?,
    )?;

    ALLOWANCES.save(deps.storage, contract.address.clone(), &Vec::new())?;

    Ok(Response::new()
        .add_message(register_receive(
            env.contract.code_hash.clone(),
            None,
            contract,
        )?)
        .add_message(set_viewing_key_msg(
            VIEWING_KEY.load(deps.storage)?,
            None,
            &contract.clone(),
        )?)
        .set_data(to_binary(&ExecuteAnswer::RegisterAsset {
            status: ResponseStatus::Success,
        })?))
}

pub fn register_wrap(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    denom: String,
    contract: &Contract,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryAdmin,
        &info.sender,
        &config.admin_auth,
    )?;

    // Asset must be registered
    if let Some(a) = ASSET.may_load(deps.storage, contract.address.clone())? {
        // Must have a token config (required for deposit)
        if let Some(conf) = a.token_config {
            // Must have deposit enabled
            if !conf.deposit_enabled {
                return Err(StdError::generic_err("Asset must have deposit enabled"));
            }
        } else {
            return Err(StdError::generic_err("Asset has no config"));
        }
    } else {
        return Err(StdError::generic_err("Unrecognized Asset"));
    }

    WRAP.save(deps.storage, denom, &contract.address)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::RegisterWrap {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn register_manager(
    deps: DepsMut,
    _env: &Env,
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

    // Ensure it isn't already registered
    if let Some(_) = MANAGER.may_load(deps.storage, contract.address.clone())? {
        return Err(StdError::generic_err("Manager already registered"));
    }

    MANAGER.save(deps.storage, contract.address.clone(), &contract)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::RegisterAsset {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn allowance(
    deps: DepsMut,
    _env: &Env,
    info: MessageInfo,
    asset: Addr,
    allowance: Allowance,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryAdmin,
        &info.sender,
        &config.admin_auth,
    )?;

    if ASSET.may_load(deps.storage, asset.clone())?.is_none() {
        return Err(StdError::generic_err("Not an asset"));
    }

    let mut allowances = ALLOWANCES
        .may_load(deps.storage, asset.clone())?
        .unwrap_or(vec![]);

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
        // "zero/null" datetime, guarantees refresh next update
        last_refresh: utc_from_seconds(0).to_rfc3339(),
        tolerance: allowance.tolerance,
    });

    if allowance.allowance_type == AllowanceType::Portion {
        let portion_sum: u128 = allowances
            .iter()
            .filter(|a| a.allowance_type == AllowanceType::Portion)
            .map(|a| a.amount.u128())
            .sum();

        // Total cannot exceed %100
        if portion_sum > 10u128.pow(18) {
            return Err(StdError::generic_err(format!(
                "Total portion allowances cannot exceed %100 ({})",
                portion_sum
            )));
        }
    }

    ALLOWANCES.save(deps.storage, asset, &allowances)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::Allowance {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn wrap_coins(deps: DepsMut, env: &Env, info: MessageInfo) -> StdResult<Response> {
    let coins = deps.querier.query_all_balances(&env.contract.address)?;

    let mut messages = vec![];
    let mut success = vec![];
    let mut failed = vec![];

    for coin in coins {
        if let Some(asset) = WRAP.may_load(deps.storage, coin.denom.clone())? {
            let token = ASSET.load(deps.storage, asset)?;
            messages.push(wrap_coin(coin.clone(), token.contract.clone())?);
            success.push(coin.clone());
            METRICS.pushf(deps.storage, env.block.time, Metric {
                action: Action::Wrap,
                context: Context::Wrap,
                timestamp: env.block.time.seconds(),
                token: token.contract.address,
                amount: coin.amount,
                user: info.sender.clone(),
            })?;
        } else {
            failed.push(coin);
        }
    }

    Ok(Response::new()
        .add_messages(messages)
        .set_data(to_binary(&ExecuteAnswer::WrapCoins { success, failed })?))
}
