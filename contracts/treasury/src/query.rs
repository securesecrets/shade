use shade_protocol::{
    c_std::{Addr, Api, Deps, Querier, StdError, StdResult, Storage, Uint128},
    contract_interfaces::dao::{adapter, manager, treasury},
    snip20::helpers::{allowance_query, balance_query},
};

use crate::storage::*;

pub fn config(deps: Deps) -> StdResult<treasury::QueryAnswer> {
    Ok(treasury::QueryAnswer::Config {
        config: CONFIG.load(deps.storage)?,
    })
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
    let allowances = ALLOWANCES.load(deps.storage, asset.clone())?;

    let full_asset = match ASSET.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Unrecognized Asset"));
        }
    };

    let mut reserves = balance_query(
        &deps.querier,
        self_address.clone(),
        VIEWING_KEY.load(deps.storage)?,
        &full_asset.contract.clone(),
    )?;

    for allowance in allowances {
        if let Some(m) = MANAGER.may_load(deps.storage, allowance.spender)? {
            reserves +=
                manager::reserves_query(deps.querier, &asset.clone(), self_address.clone(), m)?;
        }
    }

    Ok(adapter::QueryAnswer::Reserves { amount: reserves })
}

pub fn unbonding(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let self_address = SELF_ADDRESS.load(deps.storage)?;
    let mut unbonding = Uint128::zero();
    let allowances = ALLOWANCES.load(deps.storage, asset.clone())?;

    let full_asset = match ASSET.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Unrecognized Asset"));
        }
    };

    for allowance in allowances {
        if let Some(m) = MANAGER.may_load(deps.storage, allowance.spender)? {
            unbonding += manager::unbonding_query(deps.querier, &asset, self_address.clone(), m)?;
        }
    }

    Ok(adapter::QueryAnswer::Unbonding { amount: unbonding })
}

pub fn unbondable(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let mut unbondable = Uint128::zero();
    let self_address = SELF_ADDRESS.load(deps.storage)?;
    let allowances = ALLOWANCES.load(deps.storage, asset.clone())?;

    /*
    let full_asset = match ASSET.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Unrecognized Asset"));
        }
    };
    */

    for allowance in allowances {
        println!("ALLOWANCE");
        if let Some(m) = MANAGER.may_load(deps.storage, allowance.spender.clone())? {
            unbondable += manager::unbondable_query(deps.querier, &asset, self_address.clone(), m)?;
        }
    }
    Ok(adapter::QueryAnswer::Unbondable { amount: unbondable })
}

pub fn claimable(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let self_address = SELF_ADDRESS.load(deps.storage)?;
    let allowances = ALLOWANCES.load(deps.storage, asset.clone())?;

    let full_asset = match ASSET.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Unrecognized Asset"));
        }
    };

    let mut claimable = Uint128::zero();

    for allowance in allowances {
        if let Some(m) = MANAGER.may_load(deps.storage, allowance.spender)? {
            claimable += manager::claimable_query(deps.querier, &asset, self_address.clone(), m)?;
        }
    }

    Ok(adapter::QueryAnswer::Claimable { amount: claimable })
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
