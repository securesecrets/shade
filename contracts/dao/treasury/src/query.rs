use crate::storage::*;
use shade_protocol::{
    c_std::{Addr, Deps, StdError, StdResult, Uint128},
    contract_interfaces::dao::{adapter, manager, treasury},
    snip20::helpers::{allowance_query, balance_query},
    utils::asset::Contract,
};
use std::collections::HashSet;

pub fn config(deps: Deps) -> StdResult<treasury::QueryAnswer> {
    Ok(treasury::QueryAnswer::Config {
        config: CONFIG.load(deps.storage)?,
    })
}

pub fn batch_balance(deps: Deps, assets: Vec<Addr>) -> StdResult<Vec<Uint128>> {
    let self_address = SELF_ADDRESS.load(deps.storage)?;

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
            self_address.clone(),
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
            self_address.clone(),
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

pub fn balance(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let full_asset = match ASSET.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Unrecognized Asset"));
        }
    };

    let self_address = SELF_ADDRESS.load(deps.storage)?;
    let allowances = ALLOWANCES.load(deps.storage, asset.clone())?;

    let mut balance = balance_query(
        &deps.querier,
        self_address.clone(),
        VIEWING_KEY.load(deps.storage)?,
        &full_asset.contract.clone(),
    )?;

    for allowance in allowances {
        if let Some(m) = MANAGER.may_load(deps.storage, allowance.spender)? {
            balance +=
                manager::balance_query(deps.querier, &asset.clone(), self_address.clone(), m)?;
        }
    }
    Ok(adapter::QueryAnswer::Balance { amount: balance })
}

pub fn reserves(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    //TODO: restrict to admin?

    let self_address = SELF_ADDRESS.load(deps.storage)?;

    let full_asset = match ASSET.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Unrecognized Asset"));
        }
    };

    let reserves = balance_query(
        &deps.querier,
        self_address.clone(),
        VIEWING_KEY.load(deps.storage)?,
        &full_asset.contract.clone(),
    )?;

    Ok(adapter::QueryAnswer::Reserves { amount: reserves })
}

pub fn allowance(deps: Deps, asset: Addr, spender: Addr) -> StdResult<treasury::QueryAnswer> {
    let self_address = SELF_ADDRESS.load(deps.storage)?;
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
            self_address,
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
