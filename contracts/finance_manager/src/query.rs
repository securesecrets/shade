use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage};
use secret_toolkit::{snip20::allowance_query, utils::Query};
use shade_protocol::{snip20, finance_manager};

use crate::state::{
    allocations_r, asset_list_r, assets_r, config_r, self_address_r,
    viewing_key_r,
};

pub fn config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<finance_manager::QueryAnswer> {
    Ok(finance_manager::QueryAnswer::Config {
        config: config_r(&deps.storage).load()?,
    })
}

pub fn balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: &HumanAddr,
) -> StdResult<finance_manager::QueryAnswer> {
    //TODO: restrict to admin?

    match assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        Some(a) => {
            Err(StdError::generic_err("Not Implemented"))
            // TODO: This should be outstanding, not counting allowance
            /*
            match (snip20::QueryMsg::Balance {
                address: self_address_r(&deps.storage).load()?,
                key: viewing_key_r(&deps.storage).load()?,
            }.query(&deps.querier, a.contract.code_hash, a.contract.address)?) {

                snip20::QueryAnswer::Balance { amount } => {
                    Ok(finance_manager::QueryAnswer::Balance { amount })
                }
                _ => Err(StdError::GenericErr {
                    msg: "Unexpected Snip20 Response".to_string(),
                    backtrace: None,
                }),
            }
            */
        }
        None => Err(StdError::NotFound {
            kind: asset.to_string(),
            backtrace: None,
        }),
    }
}

pub fn assets<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<finance_manager::QueryAnswer> {
    Ok(finance_manager::QueryAnswer::Assets {
        assets: asset_list_r(&deps.storage).load()?,
    })
}

pub fn allocations<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<finance_manager::QueryAnswer> {
    Ok(finance_manager::QueryAnswer::Allocations {
        allocations: match allocations_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
            None => vec![],
            Some(a) => a,
        },
    })
}
