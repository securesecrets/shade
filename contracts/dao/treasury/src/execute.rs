use crate::storage::*;
use shade_protocol::{
    c_std::{
        to_binary,
        Addr,
        Binary,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
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
        asset::{Contract, RawContract},
        cycle::{exceeds_cycle, parse_utc_datetime, utc_from_seconds, utc_now, Cycle},
        generic_response::ResponseStatus,
        wrap::wrap_coin,
    },
};
use std::collections::HashMap;

const ONE_HUNDRED_PERCENT: Uint128 = Uint128::new(10u128.pow(18u32));

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _sender: Addr,
    from: Addr,
    amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<Response> {
    // Only store metrics for registered assets
    if ASSET.may_load(deps.storage, info.sender.clone())?.is_some() {
        METRICS.push(deps.storage, env.block.time, Metric {
            action: Action::FundsReceived,
            context: Context::Receive,
            timestamp: env.block.time.seconds(),
            token: info.sender,
            amount,
            user: from,
        })?;
    }

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Receive {
        status: ResponseStatus::Success,
    })?))
}

pub fn try_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin_auth: Option<RawContract>,
    multisig: Option<String>,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;

    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryAdmin,
        &info.sender,
        &config.admin_auth,
    )?;

    if let Some(admin_auth) = admin_auth {
        config.admin_auth = admin_auth.into_valid(deps.api)?;
    }
    if let Some(multisig) = multisig {
        config.multisig = deps.api.addr_validate(&multisig)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::UpdateConfig {
            config,
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn update(deps: DepsMut, env: &Env, info: MessageInfo, asset: Addr) -> StdResult<Response> {
    match RUN_LEVEL.load(deps.storage)? {
        RunLevel::Migrating => {
            let config = CONFIG.load(deps.storage)?;
            validate_admin(
                &deps.querier,
                AdminPermissions::TreasuryAdmin,
                &info.sender,
                &config.admin_auth,
            )?;
            migrate(deps, env, info, asset)
        }
        RunLevel::Deactivated => {
            return Err(StdError::generic_err("Contract Deactivated"));
        }
        RunLevel::Normal => rebalance(deps, env, info, asset),
    }
}

fn rebalance(deps: DepsMut, env: &Env, _info: MessageInfo, asset: Addr) -> StdResult<Response> {
    let viewing_key = VIEWING_KEY.load(deps.storage)?;

    let full_asset = match ASSET.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Not a registered asset"));
        }
    };

    let mut allowances = ALLOWANCES.load(deps.storage, asset.clone())?;

    let mut total_balance = balance_query(
        &deps.querier,
        env.contract.address.clone(),
        viewing_key.clone(),
        &full_asset.contract.clone(),
    )?;
    let mut token_balance = total_balance;

    // { spender: (balance, allowance) }
    let mut metadata: HashMap<Addr, (Uint128, Uint128)> = HashMap::new();

    let mut messages = vec![];
    let mut metrics = vec![];

    let now = utc_now(&env);

    // allowances marked for removal
    let mut stale_allowances = vec![];

    for (i, a) in allowances.clone().iter().enumerate() {
        let manager = MANAGER.may_load(deps.storage, a.spender.clone())?;
        let mut claimable = Uint128::zero();
        let mut unbonding = Uint128::zero();
        let mut balance = Uint128::zero();
        // we can only get some of these numbers when it's a treasury manager
        if let Some(m) = manager.clone() {
            claimable = manager::claimable_query(
                deps.querier,
                &asset.clone(),
                env.contract.address.clone(),
                m.clone(),
            )?;
            // claim when not zero
            if !claimable.is_zero() {
                messages.push(manager::claim_msg(&asset.clone(), m.clone())?);
                metrics.push(Metric {
                    action: Action::Claim,
                    context: Context::Rebalance,
                    timestamp: env.block.time.seconds(),
                    token: asset.clone(),
                    amount: claimable,
                    user: m.address.clone(),
                });
            }

            unbonding = manager::unbonding_query(
                deps.querier,
                &asset.clone(),
                env.contract.address.clone(),
                m.clone(),
            )?;

            balance = manager::balance_query(
                deps.querier,
                &asset.clone(),
                env.contract.address.clone(),
                m,
            )?
        }

        // can allways get allowance for everyone
        let allowance = allowance_query(
            &deps.querier,
            env.contract.address.clone(),
            a.spender.clone(),
            viewing_key.clone(),
            1,
            &full_asset.contract.clone(),
        )?
        .allowance;

        if token_balance > allowance {
            token_balance -= allowance;
        } else {
            token_balance = Uint128::zero();
        }

        // if all of these are zero then we need to remove the allowance at the end of the fn
        if balance.is_zero()
            && unbonding.is_zero()
            && claimable.is_zero()
            && allowance.is_zero()
            && a.amount.is_zero()
        {
            stale_allowances.push(i);
        }

        metadata.insert(a.spender.clone(), (balance, allowance));
        total_balance += balance + unbonding;
    }

    /* Amounts given priority sice the array is sorted
     * portions are calculated after amounts are taken from total
     */
    for (i, allowance) in allowances.clone().iter().enumerate() {
        let last_refresh = parse_utc_datetime(&allowance.last_refresh)?;

        // Refresh allowance if cycle is exceeded
        if !exceeds_cycle(&now, &last_refresh, allowance.cycle.clone()) {
            // Once allowances need 1 refresh if last_refresh == 'null'
            if allowance.cycle == Cycle::Once {
                if last_refresh.timestamp() != 0 {
                    if stale_allowances.iter().find(|&&x| x == i) == None {
                        stale_allowances.push(i);
                        stale_allowances.sort();
                    }
                    continue;
                }
            } else {
                continue;
            }
        }

        allowances[i].last_refresh = now.to_rfc3339();

        // calculate the desired amount for the manager
        let desired_amount = match allowance.allowance_type {
            AllowanceType::Amount => {
                // reduce total_balance so amount allowances are not used in the calculation for
                // portion allowances
                if total_balance >= allowance.amount {
                    total_balance -= allowance.amount;
                } else {
                    total_balance = Uint128::zero();
                }
                allowance.amount
            }
            AllowanceType::Portion => {
                // This just gives a ratio of total balance where allowance.amount is the percent
                total_balance.multiply_ratio(allowance.amount, ONE_HUNDRED_PERCENT)
            }
        };

        let (balance, cur_allowance) = metadata[&allowance.spender];
        let total = balance + cur_allowance;

        // calculate threshold
        let threshold = desired_amount.multiply_ratio(allowance.tolerance, ONE_HUNDRED_PERCENT);

        match desired_amount.cmp(&total) {
            // Decrease Allowance
            std::cmp::Ordering::Less => {
                // decrease is cur_allow + bal - allow.amount because the current amount of funds
                // the spender has access to is it's current allowance plus it balance, so to
                // find the decrease, we subtract that by the amount the allowance is set to
                let mut decrease = total - desired_amount;
                // threshold check
                if decrease <= threshold {
                    continue;
                }
                // Allowance fully covers amount needed
                if cur_allowance >= decrease {
                    if !decrease.is_zero() {
                        messages.push(decrease_allowance_msg(
                            allowance.spender.clone(),
                            decrease,
                            None,
                            None,
                            1,
                            &full_asset.contract.clone(),
                            vec![],
                        )?);
                        token_balance += decrease;
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
                // Reduce allowance then unbond
                else {
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
                        token_balance += cur_allowance;
                        metrics.push(Metric {
                            action: Action::DecreaseAllowance,
                            context: Context::Rebalance,
                            timestamp: env.block.time.seconds(),
                            token: asset.clone(),
                            amount: cur_allowance,
                            user: allowance.spender.clone(),
                        });
                    }

                    decrease -= cur_allowance;

                    // Unbond remaining
                    if !decrease.is_zero() {
                        if let Some(m) =
                            MANAGER.may_load(deps.storage, allowance.spender.clone())?
                        {
                            messages.push(manager::unbond_msg(
                                &asset.clone(),
                                decrease,
                                m.clone(),
                            )?);
                            metrics.push(Metric {
                                action: Action::Unbond,
                                context: Context::Rebalance,
                                timestamp: env.block.time.seconds(),
                                token: asset.clone(),
                                amount: decrease,
                                user: m.address.clone(),
                            });
                        }
                    }
                }
            }
            // Increase Allowance
            std::cmp::Ordering::Greater => {
                let mut increase = desired_amount - total;
                if increase > token_balance {
                    increase = token_balance;
                }
                token_balance -= increase;

                // threshold check
                if increase <= threshold {
                    continue;
                }
                if !increase.is_zero() {
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
            }
            _ => {}
        }
    }

    if !stale_allowances.is_empty() {
        for index in stale_allowances.iter().rev() {
            allowances.remove(index.clone());
        }
    }
    ALLOWANCES.save(deps.storage, asset.clone(), &allowances)?;

    METRICS.append(deps.storage, env.block.time, &mut metrics)?;

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

    METRICS.append(deps.storage, env.block.time, &mut metrics)?;

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
    _env: &Env,
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
        // Deposit mut be enabled
        if let Some(conf) = a.token_config {
            if conf.deposit_enabled {
                WRAP.save(deps.storage, denom, &contract.address)?;
                Ok(
                    Response::new().set_data(to_binary(&ExecuteAnswer::RegisterWrap {
                        status: ResponseStatus::Success,
                    })?),
                )
            } else {
                Err(StdError::generic_err("Asset deposit not enabled"))
            }
        } else {
            Err(StdError::generic_err("Asset has no token config"))
        }
    } else {
        Err(StdError::generic_err("Unrecognized Asset"))
    }
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
        AdminPermissions::TreasuryAdmin,
        &info.sender,
        &config.admin_auth,
    )?;

    // Ensure it isn't already registered
    if let Some(_) = MANAGER.may_load(deps.storage, contract.address.clone())? {
        return Err(StdError::generic_err("Manager already registered"));
    }

    MANAGER.save(deps.storage, contract.address.clone(), &contract)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::RegisterManager {
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
    refresh_now: bool,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryAdmin,
        &info.sender,
        &config.admin_auth,
    )?;

    if ASSET.may_load(deps.storage, asset.clone())?.is_none() {
        return Err(StdError::generic_err("Not a registered asset"));
    }

    if allowance.tolerance >= ONE_HUNDRED_PERCENT {
        return Err(StdError::generic_err(format!(
            "Tolerance {} >= 100%",
            allowance.tolerance
        )));
    }

    let mut allowances = ALLOWANCES
        .may_load(deps.storage, asset.clone())?
        .unwrap_or(vec![]);

    // This will cause allowance refresh asap, changed below if !refresh_now
    let mut last_refresh = utc_from_seconds(0).to_rfc3339();

    // remove duplicated allowance
    match allowances
        .iter()
        .position(|a| a.spender == allowance.spender)
    {
        Some(i) => {
            if !refresh_now {
                last_refresh = allowances[i].last_refresh.clone();
            }
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
        last_refresh,
        tolerance: allowance.tolerance,
    });

    // ensure that the portion allocations don't go above 100%
    if allowances
        .iter()
        .map(|a| {
            if a.allowance_type == AllowanceType::Portion {
                a.amount
            } else {
                Uint128::zero()
            }
        })
        .sum::<Uint128>()
        > ONE_HUNDRED_PERCENT
    {
        return Err(StdError::generic_err(
            "Invalid allowance total exceeding 100%",
        ));
    }

    // Sort list before going into storage
    allowances.sort_by(|a, b| match a.allowance_type {
        AllowanceType::Amount => match b.allowance_type {
            AllowanceType::Amount => std::cmp::Ordering::Equal,
            AllowanceType::Portion => std::cmp::Ordering::Less,
        },
        AllowanceType::Portion => match b.allowance_type {
            AllowanceType::Amount => std::cmp::Ordering::Greater,
            AllowanceType::Portion => std::cmp::Ordering::Equal,
        },
    });

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
            METRICS.push(deps.storage, env.block.time, Metric {
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
