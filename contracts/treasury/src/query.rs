use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage, Uint128};
use secret_toolkit::{snip20::{allowance_query, balance_query}, utils::Query};
use shade_protocol::{snip20, treasury, adapter};

use crate::state::{
    allowances_r, asset_list_r, assets_r, config_r, self_address_r,
    viewing_key_r, managers_r,
    account_list_r, account_r,
};

pub fn config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<treasury::QueryAnswer> {
    Ok(treasury::QueryAnswer::Config {
        config: config_r(&deps.storage).load()?,
    })
}

pub fn balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: &HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    //TODO: restrict to admin?

    let managers = managers_r(&deps.storage).load()?;

    match assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        Some(a) => {
            let mut balance = balance_query(
                &deps.querier,
                self_address_r(&deps.storage).load()?,
                viewing_key_r(&deps.storage).load()?,
                1,
                a.contract.code_hash.clone(),
                a.contract.address.clone(),
            )?.amount;


            for allowance in allowances_r(&deps.storage).load(&asset.as_str().as_bytes())? {
                match allowance {
                    treasury::Allowance::Portion { spender, .. } => {
                        let manager = managers
                            .clone().into_iter()
                            .find(|m| m.contract.address == spender).unwrap();
                        balance += adapter::balance_query(
                            &deps,
                            asset,
                            manager.contract
                        )?;
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
    asset: &HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    //TODO: restrict to admin?

    let managers = managers_r(&deps.storage).load()?;

    match assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        Some(a) => {
            let mut reserves = balance_query(
                &deps.querier,
                self_address_r(&deps.storage).load()?,
                viewing_key_r(&deps.storage).load()?,
                1,
                a.contract.code_hash.clone(),
                a.contract.address.clone(),
            )?.amount;

            for allowance in allowances_r(&deps.storage).load(&asset.as_str().as_bytes())? {
                match allowance {
                    treasury::Allowance::Portion { spender, .. } => {
                        let manager = managers
                            .clone().into_iter()
                            .find(|m| m.contract.address == spender).unwrap();
                        reserves += adapter::reserves_query(
                            &deps,
                            asset,
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
    asset: &HumanAddr,
) -> StdResult<adapter::QueryAnswer> {

    let managers = managers_r(&deps.storage).load()?;
    let mut unbonding = Uint128::zero();

    for allowance in allowances_r(&deps.storage).load(&asset.as_str().as_bytes())? {
        match allowance {
            treasury::Allowance::Portion { spender, .. } => {
                let manager = managers
                    .clone().into_iter()
                    .find(|m| m.contract.address == spender).unwrap();
                unbonding += adapter::unbonding_query(
                    &deps,
                    asset,
                    manager.contract
                )?;
            }
            _ => {}
        };
    }

    Ok(adapter::QueryAnswer::Unbonding {
        amount: unbonding
    })
}

pub fn claimable<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: &HumanAddr,
) -> StdResult<adapter::QueryAnswer> {

    let managers = managers_r(&deps.storage).load()?;
    let mut claimable = Uint128::zero();

    for allowance in allowances_r(&deps.storage).load(&asset.as_str().as_bytes())? {
        match allowance {
            treasury::Allowance::Portion { spender, .. } => {
                let manager = managers
                    .clone().into_iter()
                    .find(|m| m.contract.address == spender).unwrap();
                claimable += adapter::claimable_query(
                    &deps,
                    asset,
                    manager.contract
                )?;
            }
            _ => {}
        };
    }

    Ok(adapter::QueryAnswer::Claimable {
        amount: claimable
    })
}

pub fn allowance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: &HumanAddr,
    spender: &HumanAddr,
) -> StdResult<treasury::QueryAnswer> {

    let self_address = self_address_r(&deps.storage).load()?;
    let key = viewing_key_r(&deps.storage).load()?;

    if let Some(full_asset) = assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
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
        assets: asset_list_r(&deps.storage).load()?,
    })
}

pub fn allowances<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<treasury::QueryAnswer> {

    Ok(treasury::QueryAnswer::Allowances {
        allowances: match allowances_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
            None => vec![],
            Some(a) => a,
        },
    })
}

pub fn account_holders<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<treasury::QueryAnswer> {
    Ok(treasury::QueryAnswer::Accounts {
        accounts: account_list_r(&deps.storage).load()?,
    })
}

pub fn account<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    holder: HumanAddr,
) -> StdResult<treasury::QueryAnswer> {
    match account_r(&deps.storage).may_load(holder.as_str().as_bytes())? {
        Some(a) => Ok(
            treasury::QueryAnswer::Account {
                account: a,
            }
        ),
        None => Err(StdError::generic_err("Not an account holder"))
    }
}
