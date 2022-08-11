use shade_protocol::{
    c_std::{
        self, to_binary, Addr, Api, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Querier,
        Response, StdError, StdResult, Storage, Uint128,
    },
    contract_interfaces::{
        admin::{validate_admin, AdminPermissions},
        dao::{
            manager,
            treasury::{Allowance, AllowanceMeta, AllowanceType, Config, ExecuteAnswer, RunLevel},
        },
        snip20,
    },
    snip20::helpers::{
        allowance_query, balance_query, decrease_allowance_msg, increase_allowance_msg,
        register_receive, send_msg, set_viewing_key_msg,
    },
    utils::{
        asset::{set_allowance, Contract},
        cycle::{exceeds_cycle, parse_utc_datetime, utc_now},
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

pub fn update(deps: DepsMut, env: &Env, info: MessageInfo, asset: Addr) -> StdResult<Response> {
    match RUN_LEVEL.load(deps.storage)? {
        RunLevel::Migrating => migrate(deps, env, info, asset),
        RunLevel::Deactivated => {
            return Err(StdError::generic_err("Contract Deactivated"));
        }
        RunLevel::Normal => rebalance(deps, env, info, asset),
    }
}

pub fn rebalance(deps: DepsMut, env: &Env, info: MessageInfo, asset: Addr) -> StdResult<Response> {
    let viewing_key = VIEWING_KEY.load(deps.storage)?;
    let self_address = SELF_ADDRESS.load(deps.storage)?;

    let full_asset = match ASSET.may_load(deps.storage, asset.clone())? {
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

    // Total for "amount" allowances (govt, assemblies, etc.)
    let mut amount_total = Uint128::zero();
    //let mut amount_allowance = Uint128::zero();

    // Total for "portion" allowances
    let mut portion_total = Uint128::zero(); //(token_balance + out_balance) - amount_total;
                                             //let mut portion_allowance = Uint128::zero();

    // { spender: (balance, allowance) }
    let mut metadata: HashMap<Addr, (Uint128, Uint128)> = HashMap::new();

    for a in allowances.clone() {
        let balance = match MANAGER.may_load(deps.storage, a.spender.clone())? {
            Some(m) => {
                manager::balance_query(deps.querier, &asset, env.contract.address.clone(), m)?
            }
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

        metadata.insert(a.spender.clone(), (balance, allowance));

        match a.allowance_type {
            AllowanceType::Amount => {
                amount_total += balance + allowance;
            }
            AllowanceType::Portion => {
                portion_total += balance + allowance;
            }
        }
    }

    let mut messages = vec![];

    for allowance in allowances {
        let last_refresh = parse_utc_datetime(&allowance.last_refresh)?;
        // Claim from managers
        let manager = MANAGER.may_load(deps.storage, allowance.spender.clone())?;
        if let Some(m) = manager.clone() {
            if !manager::claimable_query(deps.querier, &asset, self_address.clone(), m.clone())?
                .is_zero()
            {
                messages.push(manager::claim_msg(&asset, m.clone())?);
            }
        }

        let now = utc_now(&env);

        match allowance.allowance_type {
            AllowanceType::Amount => {
                // Refresh allowance if cycle is exceeded
                if exceeds_cycle(&last_refresh, &now, allowance.cycle) {
                    let (_, cur_allowance) = metadata[&allowance.spender];
                    let threshold = allowance
                        .amount
                        .multiply_ratio(allowance.tolerance, 10u128.pow(18));

                    match allowance.amount.cmp(&cur_allowance) {
                        // Decrease Allowance
                        std::cmp::Ordering::Less => {
                            messages.push(decrease_allowance_msg(
                                allowance.spender.clone(),
                                cur_allowance - allowance.amount,
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
                                allowance.spender.clone(),
                                allowance.amount - cur_allowance,
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
            AllowanceType::Portion => {
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

                        // Unbond remaining
                        if decrease > Uint128::zero() {
                            match manager {
                                Some(m) => messages.push(manager::unbond_msg(&asset, decrease, m)?),
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
                            allowance.spender,
                            decrease,
                            None,
                            None,
                            1,
                            &full_asset.contract.clone(),
                            vec![],
                        )?);
                    }
                }
            }
        }
    }

    Ok(Response::new()
        .add_messages(messages)
        .set_data(to_binary(&ExecuteAnswer::Rebalance {
            status: ResponseStatus::Success,
        })?))
}

pub fn migrate(deps: DepsMut, env: &Env, info: MessageInfo, asset: Addr) -> StdResult<Response> {
    let mut messages = vec![];

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

            messages.push(manager::unbond_msg(&asset, unbondable, m.clone())?);
            let claimable = manager::claimable_query(
                deps.querier,
                &asset,
                env.contract.address.clone(),
                m.clone(),
            )?;

            if !claimable.is_zero() {
                claimed += claimable;
                messages.push(manager::claim_msg(&asset, m.clone())?);
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
        }
    }

    let balance = balance_query(
        &deps.querier,
        env.contract.address.clone(),
        viewing_key.clone(),
        &full_asset.contract.clone(),
    )?;

    todo!("need to send tokens to multisig");

    if !(balance + claimed).is_zero() {
        let config = CONFIG.load(deps.storage)?;

        //TODO: send to super admin from admin_auth
        messages.push(send_msg(
            config.multisig, //unbonder.clone(),
            balance + claimed,
            None,
            None,
            None,
            &full_asset.contract.clone(),
        )?);
    }

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

    // TODO force super-admin
    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryAdmin,
        &info.sender,
        &env.contract.address,
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
        &env.contract.address,
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
        &env.contract.address,
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
    let config = CONFIG.load(deps.storage)?;
    /* ADMIN ONLY */
    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryAdmin,
        &info.sender,
        &env.contract.address,
        &config.admin_auth,
    )?;

    let full_asset = match ASSET.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Not an asset"));
        }
    };

    let mut allowances = ALLOWANCES
        .may_load(deps.storage, asset.clone())?
        .unwrap_or(vec![]);

    let last_refresh: DateTime<Utc> = DateTime::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc);

    allowances.push(AllowanceMeta {
        spender: allowance.spender.clone(),
        amount: allowance.amount,
        cycle: allowance.cycle,
        allowance_type: allowance.allowance_type,
        // "zero/null" datetime
        last_refresh: last_refresh.to_rfc3339(),
        tolerance: allowance.tolerance,
    });

    ALLOWANCES.save(deps.storage, asset, &allowances)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::Allowance {
            status: ResponseStatus::Success,
        })?),
    )
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

            if claimable.is_zero() {
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
        &env.contract.address,
        &CONFIG.load(deps.storage)?.admin_auth,
    )?;

    let self_address = SELF_ADDRESS.load(deps.storage)?;

    let mut messages = vec![];

    let mut unbond_amount = amount;
    let mut unbonded = Uint128::zero();

    for allowance in ALLOWANCES.load(deps.storage, asset.clone())? {
        if let Some(m) = MANAGER.may_load(deps.storage, allowance.spender)? {
            let unbondable =
                manager::unbondable_query(deps.querier, &asset, self_address.clone(), m.clone())?;

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

    Ok(
        Response::new().set_data(to_binary(&manager::ExecuteAnswer::Unbond {
            status: ResponseStatus::Success,
            amount,
        })?),
    )
}
