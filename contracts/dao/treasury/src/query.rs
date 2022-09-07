use shade_protocol::{
    c_std::{Api, Deps, Addr, Querier, StdError, StdResult, Storage, Uint128},
    snip20::helpers::{allowance_query, balance_query},
    contract_interfaces::{
        dao::{
            manager, 
            treasury,
        },
    },
};

use crate::storage::*;

pub fn config(
    deps: Deps,
) -> StdResult<treasury::QueryAnswer> {
    Ok(treasury::QueryAnswer::Config {
        config: CONFIG.load(deps.storage)?,
    })
}

pub fn balance(
    deps: Deps,
    asset: Addr,
) -> StdResult<manager::QueryAnswer> {
    //TODO: restrict to admin?

    let managers = MANAGERS.load(deps.storage)?;

    match ASSETS.may_load(deps.storage, asset.clone())? {
        Some(a) => {
            let mut balance = balance_query(
                &deps.querier,
                SELF_ADDRESS.load(deps.storage)?,
                VIEWING_KEY.load(deps.storage)?,
                &a.contract.clone(),
            )?;

            //panic!("BALANCE {}", balance);

            let self_address = SELF_ADDRESS.load(deps.storage)?;

            for allowance in ALLOWANCES.load(deps.storage, asset.clone())? {
                match allowance {
                    treasury::Allowance::Portion { spender, .. } => {
                        let manager = managers
                            .clone()
                            .into_iter()
                            .find(|m| m.contract.address == spender)
                            .unwrap();
                        balance += manager::balance_query(
                            deps.querier,
                            &asset.clone(),
                            self_address.clone(),
                            manager.contract,
                        )?;
                    }
                    _ => {}
                };
            }
            Ok(manager::QueryAnswer::Balance { amount: balance })
        }
        None => Err(StdError::generic_err(format!("Asset not found: {}", asset.to_string()))),
    }
}

pub fn reserves(
    deps: Deps,
    asset: Addr,
) -> StdResult<manager::QueryAnswer> {
    //TODO: restrict to admin?

    let managers = MANAGERS.load(deps.storage)?;
    let self_address = SELF_ADDRESS.load(deps.storage)?;

    match ASSETS.may_load(deps.storage, asset.clone())? {
        Some(a) => {
            let mut reserves = balance_query(
                &deps.querier,
                self_address,
                VIEWING_KEY.load(deps.storage)?,
                &a.contract.clone(),
            )?;

            /*
            for allowance in ALLOWANCES.load(deps.storage, asset.clone())? {
                match allowance {
                    treasury::Allowance::Portion { spender, .. } => {
                        let manager = managers
                            .clone().into_iter()
                            .find(|m| m.contract.address == spender).unwrap();
                        reserves += manager::reserves_query(
                            &deps,
                            &asset.clone(),
                            self_address.clone(),
                            manager.contract
                        )?;
                    }
                    _ => {}
                };
            }
            */
            Ok(manager::QueryAnswer::Reserves { amount: reserves })
        }
        None => Err(StdError::generic_err(format!("Asset not found {}", asset))),
    }
}

pub fn unbonding(
    deps: Deps,
    asset: Addr,
) -> StdResult<manager::QueryAnswer> {
    let managers = MANAGERS.load(deps.storage)?;
    let self_address = SELF_ADDRESS.load(deps.storage)?;
    let mut unbonding = Uint128::zero();

    for allowance in ALLOWANCES.load(deps.storage, asset.clone())? {
        match allowance {
            treasury::Allowance::Portion { spender, .. } => {
                let manager = managers
                    .clone()
                    .into_iter()
                    .find(|m| m.contract.address == spender)
                    .unwrap();
                unbonding += manager::unbonding_query(deps.querier, &asset, self_address.clone(), manager.contract)?;
            }
            _ => {}
        };
    }

    Ok(manager::QueryAnswer::Unbonding { amount: unbonding })
}

pub fn unbondable(
    deps: Deps,
    asset: Addr,
) -> StdResult<manager::QueryAnswer> {
    let managers = MANAGERS.load(deps.storage)?;
    let mut unbondable = Uint128::zero();
    let self_address = SELF_ADDRESS.load(deps.storage)?;

    for manager in managers {
        unbondable += manager::unbondable_query(deps.querier, &asset, self_address.clone(), manager.contract)?;
    }
    /*
    for allowance in ALLOWANCES.load(deps.storage, asset.clone())? {
        match allowance {
            treasury::Allowance::Portion { spender, .. } => {
                let manager = managers
                    .clone()
                    .into_iter()
                    .find(|m| m.contract.address == spender)
                    .unwrap();
                unbondable += manager::unbondable_query(&deps, &asset, manager.contract)?;
            }
            _ => {}
        };
    }
    */

    Ok(manager::QueryAnswer::Unbondable { amount: unbondable })
}

pub fn claimable(
    deps: Deps,
    asset: Addr,
) -> StdResult<manager::QueryAnswer> {
    let managers = MANAGERS.load(deps.storage)?;
    let self_address = SELF_ADDRESS.load(deps.storage)?;
    let claimable = managers
        .into_iter()
        .map(|m| manager::claimable_query(
                deps.querier, 
                &asset, 
                self_address.clone(),
                m.contract
            ).ok().unwrap().u128())
        .sum();

    Ok(manager::QueryAnswer::Claimable { amount: Uint128::new(claimable) })
}

pub fn allowance(
    deps: Deps,
    asset: Addr,
    spender: Addr,
) -> StdResult<treasury::QueryAnswer> {
    let self_address = SELF_ADDRESS.load(deps.storage)?;
    let key = VIEWING_KEY.load(deps.storage)?;

    if let Some(full_asset) = ASSETS.may_load(deps.storage, asset.clone())? {
        let cur_allowance = allowance_query(
            &deps.querier,
            self_address,
            spender.clone(),
            key,
            1,
            &full_asset.contract.clone(),
        )?;

        return Ok(treasury::QueryAnswer::Allowance {
            amount: cur_allowance.amount,
        });
    }

    Err(StdError::generic_err(format!("Unknown Asset: {}", asset)))
}

pub fn assets(
    deps: Deps,
) -> StdResult<treasury::QueryAnswer> {
    Ok(treasury::QueryAnswer::Assets {
        assets: ASSET_LIST.load(deps.storage)?,
    })
}

pub fn allowances(
    deps: Deps,
    asset: Addr,
) -> StdResult<treasury::QueryAnswer> {
    Ok(treasury::QueryAnswer::Allowances {
        allowances: match ALLOWANCES.may_load(deps.storage, asset)? {
            None => vec![],
            Some(a) => a,
        },
    })
}
