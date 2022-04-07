use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage, Uint128};
use secret_toolkit::{
    snip20::allowance_query,
    utils::Query,
};
use shade_protocol::{
    snip20,
    finance_manager,
    adapter,
    manager,
    utils::asset::Contract,
};

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

pub fn adapter_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    adapter: Contract,
    asset: &HumanAddr,
) -> StdResult<Uint128> {

    match (adapter::QueryMsg::Balance {
        asset: asset.clone(),
    }.query(&deps.querier, adapter.code_hash, adapter.address.clone())?) {
        adapter::QueryAnswer::Balance { amount } => Ok(amount),
        _ => Err(
            StdError::generic_err(
                format!("Failed to query adapter balance from {}", adapter.address)
            )
        )
    }
}

pub fn outstanding_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: &HumanAddr,
) -> StdResult<manager::QueryAnswer> {

    if let Some(full_asset) = assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {

        let allocs = allocations_r(&deps.storage).load(asset.as_str().as_bytes())?;

        let balance = allocs.iter().map(|alloc| {
            adapter_balance(&deps, alloc.contract.clone(), &full_asset.contract.address).ok().unwrap().u128()
        }).sum::<u128>();

        return Ok(manager::QueryAnswer::Balance { 
            amount: Uint128(balance)
        });
    }

    Err(StdError::generic_err("Not a registered asset"))

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

pub fn claimable<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<manager::QueryAnswer> {

    let allocations = match allocations_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        Some(a) => a,
        None => { return Err(StdError::generic_err("Not an asset")); }
    };

    let mut failed_queries = 0u128;
    let mut claimable = 0u128;

    for alloc in allocations {
        claimable += match (adapter::QueryMsg::Claimable {
            asset: asset.clone(),
        }.query(&deps.querier, alloc.contract.code_hash, alloc.contract.address.clone())?) {
            adapter::QueryAnswer::Claimable { amount } => amount.u128(),
            _ => {
                return Err( StdError::generic_err(
                    format!("Failed to query adapter claimable from {}", alloc.contract.address)
                ))
            }
        }
    }

    /*
    let claimable = allocations.into_iter().map(|a| {
        match (adapter::QueryMsg::Claimable {
            asset: asset.clone(),
        }.query(&deps.querier, a.contract.code_hash, a.contract.address.clone()).ok()) {
            //adapter::QueryAnswer::Claimable { amount } => amount.u128(),
            Some(resp) => resp.amount.u128(),
            _ => {
                failed_queries += 1;
                0u128
            }
            /*
            {
                return Err( StdError::generic_err(
                    format!("Failed to query adapter claimable from {}", adapter.address)
                ))
            }
            */
        }
    }).sum::<u128>();
    */

    Ok(manager::QueryAnswer::Claimable {
        amount: Uint128(claimable),
    })
}
