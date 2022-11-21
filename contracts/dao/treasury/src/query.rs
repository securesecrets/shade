use crate::storage::*;
use shade_protocol::{
    c_std::{Addr, Deps, Env, StdError, StdResult, Uint128},
    contract_interfaces::dao::{adapter, manager, treasury},
    snip20::helpers::{allowance_query, balance_query},
    utils::{asset::Contract, cycle::parse_utc_datetime, storage::plus::period_storage::Period},
};
use std::collections::HashSet;

pub fn config(deps: Deps) -> StdResult<treasury::QueryAnswer> {
    Ok(treasury::QueryAnswer::Config {
        config: CONFIG.load(deps.storage)?,
    })
}

pub fn metrics(
    deps: Deps,
    env: Env,
    date: Option<String>,
    epoch: Option<Uint128>,
    period: Period,
) -> StdResult<treasury::QueryAnswer> {
    if date.is_some() && epoch.is_some() {
        return Err(StdError::generic_err("cannot pass both epoch and date"));
    }
    let key = {
        if let Some(d) = date {
            parse_utc_datetime(&d)?.timestamp() as u64
        } else if let Some(e) = epoch {
            e.u128() as u64
        } else {
            env.block.time.seconds()
        }
    };
    Ok(treasury::QueryAnswer::Metrics {
        metrics: METRICS.load_period(deps.storage, key, period)?,
    })
}

pub fn batch_balance(deps: Deps, env: Env, assets: Vec<Addr>) -> StdResult<Vec<Uint128>> {
    let mut balances = vec![];
    let mut managers: HashSet<Contract> = HashSet::new();

    for asset in assets.clone() {
        let full_asset = match ASSET.may_load(deps.storage, asset.clone())? {
            Some(a) => a,
            None => {
                return Err(StdError::generic_err("Unrecognized Asset"));
            }
        };

        let balance = balance_query(
            &deps.querier,
            env.contract.address.clone(),
            VIEWING_KEY.load(deps.storage)?,
            &full_asset.contract.clone(),
        )?;

        balances.push(balance);

        // build list of unique managers to query balances
        for allowance in ALLOWANCES.load(deps.storage, asset.clone())? {
            if let Some(m) = MANAGER.may_load(deps.storage, allowance.spender)? {
                managers.insert(m);
            }
        }
    }
    for manager in managers {
        let manager_balances = manager::batch_balance_query(
            deps.querier,
            &assets.clone(),
            env.contract.address.clone(),
            manager,
        )?;
        balances = balances
            .into_iter()
            .zip(manager_balances.into_iter())
            .map(|(a, b)| a + b)
            .collect();
    }

    Ok(balances)
}

pub fn balance(deps: Deps, env: Env, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let full_asset = match ASSET.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Unrecognized Asset"));
        }
    };

    let allowances = ALLOWANCES.load(deps.storage, asset.clone())?;

    let mut balance = balance_query(
        &deps.querier,
        env.contract.address.clone(),
        VIEWING_KEY.load(deps.storage)?,
        &full_asset.contract.clone(),
    )?;

    for allowance in allowances {
        if let Some(m) = MANAGER.may_load(deps.storage, allowance.spender)? {
            balance += manager::balance_query(
                deps.querier,
                &asset.clone(),
                env.contract.address.clone(),
                m,
            )?;
        }
    }
    Ok(adapter::QueryAnswer::Balance { amount: balance })
}

pub fn reserves(deps: Deps, env: Env, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    //TODO: restrict to admin?

    let full_asset = match ASSET.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Unrecognized Asset"));
        }
    };

    let reserves = balance_query(
        &deps.querier,
        env.contract.address.clone(),
        VIEWING_KEY.load(deps.storage)?,
        &full_asset.contract.clone(),
    )?;

    Ok(adapter::QueryAnswer::Reserves { amount: reserves })
}

pub fn allowance(
    deps: Deps,
    env: Env,
    asset: Addr,
    spender: Addr,
) -> StdResult<treasury::QueryAnswer> {
    let key = VIEWING_KEY.load(deps.storage)?;

    let full_asset = match ASSET.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Unrecognized Asset"));
        }
    };

    return Ok(treasury::QueryAnswer::Allowance {
        amount: allowance_query(
            &deps.querier,
            env.contract.address,
            spender.clone(),
            key,
            1,
            &full_asset.contract.clone(),
        )?
        .allowance,
    });
}

pub fn assets(deps: Deps) -> StdResult<treasury::QueryAnswer> {
    Ok(treasury::QueryAnswer::Assets {
        assets: ASSET_LIST.iter(deps.storage).collect(),
    })
}

pub fn allowances(deps: Deps, asset: Addr) -> StdResult<treasury::QueryAnswer> {
    Ok(treasury::QueryAnswer::Allowances {
        allowances: ALLOWANCES.may_load(deps.storage, asset)?.unwrap_or(vec![]),
    })
}
