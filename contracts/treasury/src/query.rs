use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage, Uint128};
use secret_toolkit::{snip20::allowance_query, utils::Query};
use shade_protocol::{snip20, treasury, manager};

use crate::state::{
    allowances_r, asset_list_r, assets_r, config_r, self_address_r,
    viewing_key_r, managers_r,
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
) -> StdResult<treasury::QueryAnswer> {
    //TODO: restrict to admin?

    match assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        Some(a) => {
            match (snip20::QueryMsg::Balance {
                address: self_address_r(&deps.storage).load()?,
                key: viewing_key_r(&deps.storage).load()?,
            }.query(&deps.querier, a.contract.code_hash, a.contract.address)?) {

                snip20::QueryAnswer::Balance { amount } => {
                    Ok(treasury::QueryAnswer::Balance { amount })
                }
                _ => Err(StdError::GenericErr {
                    msg: "Unexpected Snip20 Response".to_string(),
                    backtrace: None,
                }),
            }
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
) -> StdResult<treasury::QueryAnswer> {
    //TODO: restrict to admin?

    let mut unbonding = Uint128::zero();
    for manager in managers_r(&deps.storage).load()? {
        unbonding += manager::unbonding_query(&deps, asset, manager.contract)?;
    }

    Ok(treasury::QueryAnswer::Unbonding {
        amount: unbonding
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

pub fn current_allowances<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<treasury::QueryAnswer> {

    Err(StdError::generic_err("Not Implemented"))
    /*
    Ok(treasury::QueryAnswer::Allowances {
        allowance: match allowances_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
            None => {
                vec![]
            }
            Some(a) => a,
        },
    })
    */
}
