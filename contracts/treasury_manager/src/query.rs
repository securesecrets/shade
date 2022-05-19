use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage, Uint128};
use secret_toolkit::{
    snip20::{allowance_query, balance_query},
    utils::Query,
};
use shade_protocol::{
    contract_interfaces::{
        dao::{
            adapter,
            treasury_manager::{
                self,
                Status,
                Holder,
                Balance
            },
        },
        snip20,
    },
    utils::asset::Contract,
};

use crate::state::{
    allocations_r,
    asset_list_r,
    assets_r,
    config_r,
    self_address_r,
    viewing_key_r,
    holder_r,
    holders_r,
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
        amount: allowance.allowance,
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
    holder: Option<HumanAddr>,
) -> StdResult<adapter::QueryAnswer> {
    let allocations = match allocations_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Not an asset"));
        }
    };

    let config = config_r(&deps.storage).load()?;

    let full_asset = assets_r(&deps.storage).load(asset.to_string().as_bytes())?;

    let mut unbonding = Uint128::zero();

    let mut claimer = match holder {
        Some(h) => h,
        None => config.treasury,
    }

    match holder_r(&deps.storage).may_load(&claimer.as_str().as_bytes()) {
        Some(h) => {
            if let Some(u) = h.unbondings.iter().find(|u| u.token == asset) {
                unbonding += u.amount;
            }
        }
        None => {
            return Err(StdError::generic_err("Invalid holder"));
        }
    }

    // Complete amounts
    let mut claimable = balance_query(
        &deps.querier,
        self_address_r(&deps.storage).load()?,
        viewing_key_r(&deps.storage).load()?,
        1,
        full_asset.contract.code_hash.clone(),
        full_asset.contract.address.clone(),
    )?.amount;

    for alloc in allocations {
        if claimable >= unbonding {
            claimable = unbonding;
            break;
        }
        claimable += adapter::claimable_query(&deps, &asset, alloc.contract.clone())?;
    }
    Ok(adapter::QueryAnswer::Claimable { amount: claimable })
}

pub fn unbonding<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
    holder: Option<HumanAddr>,
) -> StdResult<adapter::QueryAnswer> {

    let allocations = match allocations_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Not an asset"));
        }
    };

    let mut unbonder = match holder {
        Some(h) => h,
        None => config.treasury,
    }

    match holder_r(&deps.storage).may_load(&unbonder.as_str().as_bytes()) {
        Some(h) => {
            if let Some(u) = h.unbondings.iter().find(|u| u.token == asset) {
                Ok(adapter::QueryAnswer::Unbonding { amount: u.amount })
            }
        }
        None => {
            return Err(StdError::generic_err("Invalid holder"));
        }
    }
}

/*NOTE Could be a situation where can_unbond returns true
 * but only partial balance available for unbond resulting
 * in stalled treasury trying to unbond more than is available
 */
pub fn unbondable<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
    holder: Option<HumanAddr>,
) -> StdResult<adapter::QueryAnswer> {

    if let Some(full_asset) = assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        let allocations = match allocations_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
            Some(a) => a,
            None => { return Err(StdError::generic_err("Not an asset")); }
        };

        let mut unbonder = match holder {
            Some(h) => h,
            None => config.treasury,
        }

        let mut balance = Uint128::zero();
        let mut unbonding = Uint128::zero();

        match holder_r(&deps.storage).may_load(&unbonder.as_str().as_bytes()) {
            Some(h) => {
                if let Some(u) = h.unbondings.iter().find(|u| u.token == asset) {
                    unbonding += u.amount;
                }
                if let Some(b) = h.balances.iter().find(|b| b.token == asset) {
                    balance += b.amount;
                }
            }
            None => {
                return Err(StdError::generic_err("Invalid holder"));
            }
        }

        let mut unbondable = balance_query(
            &deps.querier,
            self_address_r(&deps.storage).load()?,
            viewing_key_r(&deps.storage).load()?,
            1,
            full_asset.contract.code_hash.clone(),
            full_asset.contract.address.clone(),
        )?.amount;

        for alloc in allocations {
            if unbondable >= (balance - unbonding)? {
                unbondable = (balance - unbonding)?;
                break;
            }
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
    holder: Option<HumanAddr>,
) -> StdResult<adapter::QueryAnswer> {

    if let Some(full_asset) = assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        let allocations = match allocations_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
            Some(a) => a,
            None => { return Err(StdError::generic_err("Not an asset")); }
        };
        //TODO Holder

        let mut balance = balance_query(
            &deps.querier,
            self_address_r(&deps.storage).load()?,
            viewing_key_r(&deps.storage).load()?,
            1,
            full_asset.contract.code_hash.clone(),
            full_asset.contract.address.clone(),
        )?.amount;

        for alloc in allocations {
            balance += adapter::balance_query(&deps,
                                  &asset, alloc.contract.clone())?;
        }

        return Ok(adapter::QueryAnswer::Balance{
            amount: balance,
        });
    }
    Err(StdError::generic_err("Not a registered asset"))
}

pub fn holders<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<treasury_manager::QueryAnswer> {
    Ok(treasury_manager::QueryAnswer::Holders {
        holders: holders_r(&deps.storage).load()?,
    })
}

pub fn holder<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    holder: HumanAddr,
) -> StdResult<treasury_manager::QueryAnswer> {
    match holder_r(&deps.storage).may_load(holder.as_str().as_bytes())? {
        Some(h) => Ok(treasury_manager::QueryAnswer::Holder { holder: h }),
        None => Err(StdError::generic_err("Not a holder")),
    }
}
