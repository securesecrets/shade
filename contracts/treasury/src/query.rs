use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage};
use secret_toolkit::utils::Query;
use shade_protocol::{snip20, treasury};

use crate::state::{allocations_r, asset_list_r, assets_r, config_r, self_address_r, viewing_key_r, last_allowance_refresh_r};

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
    //TODO: restrict to admin

    match assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        Some(a) => {
            let resp = snip20::QueryMsg::Balance {
                address: self_address_r(&deps.storage).load()?,
                key: viewing_key_r(&deps.storage).load()?,
            }
            .query(&deps.querier, a.contract.code_hash, a.contract.address)?;

            match resp {
                snip20::QueryAnswer::Balance { amount } => {
                    Ok(treasury::QueryAnswer::Balance { amount })
                }
                _ => Err(StdError::GenericErr {
                    msg: "Unexpected Response".to_string(),
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

pub fn assets<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<treasury::QueryAnswer> {
    Ok(treasury::QueryAnswer::Assets {
        assets: asset_list_r(&deps.storage).load()?,
    })
}

pub fn allocations<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<treasury::QueryAnswer> {
    Ok(treasury::QueryAnswer::Allocations {
        allocations: match allocations_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
            None => {
                vec![]
            }
            Some(a) => a,
        },
    })
}

pub fn last_allowance_refresh<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<treasury::QueryAnswer> {

    Ok(treasury::QueryAnswer::LastAllowanceRefresh {
        datetime: last_allowance_refresh_r(&deps.storage).load()?,
    })
}

/*
pub fn can_rebalance<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::CanRebalance { possible: false })
}
*/
