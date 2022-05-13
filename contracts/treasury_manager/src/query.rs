use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage, Uint128};
use secret_toolkit::{
    snip20::{allowance_query, balance_query},
    utils::Query,
};
use shade_protocol::{
    snip20,
    treasury_manager,
    adapter,
    utils::asset::Contract,
};

use crate::state::{
    allocations_r, asset_list_r, assets_r, config_r, self_address_r,
    viewing_key_r,
};

pub fn config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<treasury_manager::QueryAnswer> {
    Ok(treasury_manager::QueryAnswer::Config {
        config: config_r(&deps.storage).load()?,
    })
}



pub fn pending_allowance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<treasury_manager::QueryAnswer> {

    let config = config_r(&deps.storage).load()?;
    let full_asset = match assets_r(&deps.storage).may_load(asset.as_str().as_bytes())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err(""));
        }
    };

    let allowance = allowance_query(
        &deps.querier,
        config.treasury,
        self_address_r(&deps.storage).load()?,
        viewing_key_r(&deps.storage).load()?,
        1,
        full_asset.contract.code_hash.clone(),
        full_asset.contract.address.clone(),
    )?;

    Ok(treasury_manager::QueryAnswer::PendingAllowance { 
        amount: allowance.allowance
    })
}

pub fn reserves<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: &HumanAddr,
) -> StdResult<adapter::QueryAnswer> {

    if let Some(full_asset) = assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        let reserves = balance_query(
            &deps.querier,
            self_address_r(&deps.storage).load()?,
            viewing_key_r(&deps.storage).load()?,
            1,
            full_asset.contract.code_hash.clone(),
            full_asset.contract.address.clone(),
        )?.amount;

        return Ok(adapter::QueryAnswer::Reserves { 
            amount: reserves,
        });
    }

    Err(StdError::generic_err("Not a registered asset"))
}

pub fn assets<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<treasury_manager::QueryAnswer> {

    Ok(treasury_manager::QueryAnswer::Assets {
        assets: asset_list_r(&deps.storage).load()?,
    })
}

pub fn allocations<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<treasury_manager::QueryAnswer> {

    Ok(treasury_manager::QueryAnswer::Allocations {
        allocations: match allocations_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
            None => vec![],
            Some(a) => a,
        },
    })
}

pub fn claimable<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {

    let allocations = match allocations_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        Some(a) => a,
        None => { return Err(StdError::generic_err("Not an asset")); }
    };

    let mut claimable = Uint128::zero();

    for alloc in allocations {
        claimable += adapter::claimable_query(&deps,
                                  &asset,
                                  alloc.contract.clone(),
                                  )?;
    }

    Ok(adapter::QueryAnswer::Claimable {
        amount: claimable,
    })
}

pub fn unbonding<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {

    let allocations = match allocations_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        Some(a) => a,
        None => { return Err(StdError::generic_err("Not an asset")); }
    };

    let mut unbonding = Uint128::zero();

    for alloc in allocations {
        unbonding += adapter::unbonding_query(&deps,
                                  &asset,
                                  alloc.contract.clone(),
                                  )?;
    }

    Ok(adapter::QueryAnswer::Unbonding {
        amount: unbonding,
    })
}

/*NOTE Could be a situation where can_unbond returns true 
 * but only partial balance available for unbond resulting
 * in stalled treasury trying to unbond more than is available
 */
pub fn unbondable<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {

    if let Some(full_asset) = assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        let allocations = match allocations_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
            Some(a) => a,
            None => { return Err(StdError::generic_err("Not an asset")); }
        };

        let mut unbondable = balance_query(
            &deps.querier,
            self_address_r(&deps.storage).load()?,
            viewing_key_r(&deps.storage).load()?,
            1,
            full_asset.contract.code_hash.clone(),
            full_asset.contract.address.clone(),
        )?.amount;

        for alloc in allocations {
            // return true if any
            unbondable += adapter::unbondable_query(&deps,
                                  &asset, alloc.contract.clone())?;
        }

        return Ok(adapter::QueryAnswer::Unbondable {
            amount: unbondable,
        });
    }

    Err(StdError::generic_err("Not a registered asset"))
}

pub fn balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {

    if let Some(full_asset) = assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        let allocations = match allocations_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
            Some(a) => a,
            None => { return Err(StdError::generic_err("Not an asset")); }
        };

        let mut balance = balance_query(
            &deps.querier,
            self_address_r(&deps.storage).load()?,
            viewing_key_r(&deps.storage).load()?,
            1,
            full_asset.contract.code_hash.clone(),
            full_asset.contract.address.clone(),
        )?.amount;

        for alloc in allocations {
            // return true if any
            balance += adapter::balance_query(&deps,
                                  &asset, alloc.contract.clone())?;
        }

        return Ok(adapter::QueryAnswer::Balance{
            amount: balance,
        });
    }
    Err(StdError::generic_err("Not a registered asset"))
}
