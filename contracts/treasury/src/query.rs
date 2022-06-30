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
        config: config_r(&deps.storage).load()?,
    })
}

pub fn balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    //TODO: restrict to admin?

    let managers = MANAGERS.load(&deps.storage)?;

    match ASSETS.may_load(&deps.storage, asset)? {
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

            for allowance in ALLOWANCES.load(&deps.storage, asset)? {
                match allowance {
                    treasury::Allowance::Portion { spender, .. } => {
                        let manager = managers
                            .clone()
                            .into_iter()
                            .find(|m| m.contract.address == spender)
                            .unwrap();
                        balance += adapter::balance_query(&deps, &asset, manager.contract)?;
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

    match ASSETS.may_load(&deps.storage, asset)? {
        Some(a) => {
            let mut reserves = balance_query(
                &deps.querier,
                SELF_ADDRESS.load(&deps.storage)?,
                VIEWING_KEY.load(&deps.storage)?,
                1,
                a.contract.code_hash.clone(),
                a.contract.address.clone(),
            )?.amount;

            for allowance in ALLOWANCES.load(&deps.storage, asset)? {
                match allowance {
                    treasury::Allowance::Portion { spender, .. } => {
                        let manager = managers
                            .clone().into_iter()
                            .find(|m| m.contract.address == spender).unwrap();
                        reserves += adapter::reserves_query(
                            &deps,
                            &asset,
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

    for allowance in ALLOWANCES.load(&deps.storage, asset)? {
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

pub fn claimable<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    let managers = MANAGERS.load(&deps.storage)?;
    let mut claimable = Uint128::zero();

    for allowance in ALLOWANCES.load(&deps.storage, asset)? {
        match allowance {
            treasury::Allowance::Portion { spender, .. } => {
                let manager = managers
                    .clone()
                    .into_iter()
                    .find(|m| m.contract.address == spender)
                    .unwrap();
                claimable += adapter::claimable_query(&deps, &asset, manager.contract)?;
            }
            _ => {}
        };
    }

    Ok(adapter::QueryAnswer::Claimable { amount: claimable })
}

pub fn allowance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
    spender: HumanAddr,
) -> StdResult<treasury::QueryAnswer> {
    let self_address = SELF_ADDRESS.load(&deps.storage)?;
    let key = VIEWING_KEY.load(&deps.storage)?;

    if let Some(full_asset) = ASSETS.may_load(&deps.storage, asset)? {
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
            allowance: cur_allowance.allowance,
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
