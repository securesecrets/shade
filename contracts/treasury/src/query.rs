use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage, Uint128};
use secret_toolkit::{
    snip20::{allowance_query, balance_query},
};
use shade_protocol::contract_interfaces::{
    dao::{
        adapter, 
        treasury::{self, storage::*},
    },
};

/*
use crate::state::{
    allowances_r,
    asset_list_r,
    assets_r,
    config_r,
    managers_r,
    self_address_r,
    viewing_key_r,
};
*/

pub fn config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<treasury::QueryAnswer> {
    Ok(treasury::QueryAnswer::Config {
        config: CONFIG.load(&deps.storage)?,
    })
}

pub fn balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    //TODO: restrict to admin?

    let managers = MANAGERS.load(&deps.storage)?;

    match ASSETS.may_load(&deps.storage, asset.clone())? {
        Some(a) => {
            let mut balance = balance_query(
                &deps.querier,
                SELF_ADDRESS.load(&deps.storage)?,
                VIEWING_KEY.load(&deps.storage)?,
                1,
                a.contract.code_hash.clone(),
                a.contract.address.clone(),
            )?
            .amount;

            for allowance in ALLOWANCES.load(&deps.storage, asset.clone())? {
                match allowance {
                    treasury::Allowance::Portion { spender, .. } => {
                        let manager = managers
                            .clone()
                            .into_iter()
                            .find(|m| m.contract.address == spender)
                            .unwrap();
                        balance += adapter::balance_query(&deps, &asset.clone(), manager.contract)?;
                    }
                    _ => {}
                };
            }
            Ok(adapter::QueryAnswer::Balance { amount: balance })
        }
        None => Err(StdError::NotFound {
            kind: asset.to_string(),
            backtrace: None,
        }),
    }
}

pub fn reserves<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    //TODO: restrict to admin?

    let managers = MANAGERS.load(&deps.storage)?;

    match ASSETS.may_load(&deps.storage, asset.clone())? {
        Some(a) => {
            let mut reserves = balance_query(
                &deps.querier,
                SELF_ADDRESS.load(&deps.storage)?,
                VIEWING_KEY.load(&deps.storage)?,
                1,
                a.contract.code_hash.clone(),
                a.contract.address.clone(),
            )?.amount;

            for allowance in ALLOWANCES.load(&deps.storage, asset.clone())? {
                match allowance {
                    treasury::Allowance::Portion { spender, .. } => {
                        let manager = managers
                            .clone().into_iter()
                            .find(|m| m.contract.address == spender).unwrap();
                        reserves += adapter::reserves_query(
                            &deps,
                            &asset.clone(),
                            manager.contract
                        )?;
                    }
                    _ => {}
                };
            }
            Ok(adapter::QueryAnswer::Reserves { amount: reserves })
        }
        None => Err(StdError::NotFound {
            kind: asset.to_string(),
            backtrace: None,
        }),
    }
}

pub fn unbonding<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    let managers = MANAGERS.load(&deps.storage)?;
    let mut unbonding = Uint128::zero();

    for allowance in ALLOWANCES.load(&deps.storage, asset.clone())? {
        match allowance {
            treasury::Allowance::Portion { spender, .. } => {
                let manager = managers
                    .clone()
                    .into_iter()
                    .find(|m| m.contract.address == spender)
                    .unwrap();
                unbonding += adapter::unbonding_query(&deps, &asset, manager.contract)?;
            }
            _ => {}
        };
    }

    Ok(adapter::QueryAnswer::Unbonding { amount: unbonding })
}

pub fn unbondable<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    let managers = MANAGERS.load(&deps.storage)?;
    let mut unbondable = Uint128::zero();

    for manager in managers {
        unbondable += adapter::unbondable_query(&deps, &asset, manager.contract)?;
    }
    /*
    for allowance in ALLOWANCES.load(&deps.storage, asset.clone())? {
        match allowance {
            treasury::Allowance::Portion { spender, .. } => {
                let manager = managers
                    .clone()
                    .into_iter()
                    .find(|m| m.contract.address == spender)
                    .unwrap();
                unbondable += adapter::unbondable_query(&deps, &asset, manager.contract)?;
            }
            _ => {}
        };
    }
    */

    Ok(adapter::QueryAnswer::Unbondable { amount: unbondable })
}

pub fn claimable<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    let managers = MANAGERS.load(&deps.storage)?;
    let claimable = managers
        .into_iter()
        .map(|m| adapter::claimable_query(&deps, &asset, m.contract).ok().unwrap().u128())
        .sum();

    Ok(adapter::QueryAnswer::Claimable { amount: Uint128(claimable) })
}

pub fn allowance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
    spender: HumanAddr,
) -> StdResult<treasury::QueryAnswer> {
    let self_address = SELF_ADDRESS.load(&deps.storage)?;
    let key = VIEWING_KEY.load(&deps.storage)?;

    if let Some(full_asset) = ASSETS.may_load(&deps.storage, asset.clone())? {
        let cur_allowance = allowance_query(
            &deps.querier,
            self_address,
            spender.clone(),
            key,
            1,
            full_asset.contract.code_hash.clone(),
            full_asset.contract.address.clone(),
        )?;

        return Ok(treasury::QueryAnswer::Allowance {
            amount: cur_allowance.allowance,
        });
    }

    Err(StdError::generic_err(format!("Unknown Asset: {}", asset)))
}

pub fn assets<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<treasury::QueryAnswer> {
    Ok(treasury::QueryAnswer::Assets {
        assets: ASSET_LIST.load(&deps.storage)?,
    })
}

pub fn allowances<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<treasury::QueryAnswer> {
    Ok(treasury::QueryAnswer::Allowances {
        allowances: match ALLOWANCES.may_load(&deps.storage, asset)? {
            None => vec![],
            Some(a) => a,
        },
    })
}
