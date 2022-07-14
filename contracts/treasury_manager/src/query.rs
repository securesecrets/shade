use shade_protocol::c_std::{Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage, Uint128};
use shade_protocol::secret_toolkit::{
    snip20::{allowance_query, balance_query},
};
use shade_protocol::{
    contract_interfaces::{
        dao::{
            adapter,
            manager,
            treasury_manager::{
                self
            },
        },
    },
};

use crate::state::{
    allocations_r,
    asset_list_r,
    assets_r,
    config_r,
    self_address_r,
    viewing_key_r,
    holding_r,
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
    holder: HumanAddr,
) -> StdResult<manager::QueryAnswer> {
    if let Some(full_asset) = assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        let reserves = balance_query(
            &deps.querier,
            self_address_r(&deps.storage).load()?,
            viewing_key_r(&deps.storage).load()?,
            1,
            full_asset.contract.code_hash.clone(),
            full_asset.contract.address.clone(),
        )?.amount;

        return Ok(manager::QueryAnswer::Reserves { 
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

pub fn unbonding<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
    holder: HumanAddr,
) -> StdResult<manager::QueryAnswer> {

    if assets_r(&deps.storage).may_load(asset.to_string().as_bytes())?.is_none() {
        return Err(StdError::generic_err("Not an asset"));
    }

    //let allocations = allocations_r(&deps.storage).load(asset.to_string().as_bytes())?;

    let _config = config_r(&deps.storage).load()?;

    match holding_r(&deps.storage).may_load(&holder.as_str().as_bytes())? {
        Some(holder) => {
            Ok(manager::QueryAnswer::Unbonding {
                amount: match holder.unbondings.iter().find(|u| u.token == asset) {
                    Some(u) => u.amount,
                    None => Uint128::zero(),
                }
            })
        },
        None => {
            return Err(StdError::generic_err("Invalid holder"));
        }
    }
}

pub fn claimable<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
    holder: HumanAddr,
) -> StdResult<manager::QueryAnswer> {

    let full_asset = match assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Not an asset"));
        }
    };
    let allocations = match allocations_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        Some(a) => a,
        None => { return Err(StdError::generic_err("Not an asset")); }
    };
    //TODO claiming needs ordered unbondings so other holders don't get bumped

    let mut claimable = balance_query(
        &deps.querier,
        self_address_r(&deps.storage).load()?,
        viewing_key_r(&deps.storage).load()?,
        1,
        full_asset.contract.code_hash.clone(),
        full_asset.contract.address.clone(),
    )?.amount;

    /*
    let _config = config_r(&deps.storage).load()?;
    let _other_unbondings = Uint128::zero();
    */

    for alloc in allocations {
        claimable += adapter::claimable_query(&deps,
                              &asset, alloc.contract.clone())?;
    }

    //TODO other unbondings
    match holding_r(&deps.storage).may_load(&holder.as_str().as_bytes())? {
        Some(holder) => {
            let unbonding = match holder.unbondings.iter().find(|u| u.token == asset) {
                Some(u) => u.amount,
                None => Uint128::zero(),
            };

            if claimable > unbonding {
                Ok(manager::QueryAnswer::Claimable {
                    amount: unbonding,
                })
            }
            else {
                Ok(manager::QueryAnswer::Claimable {
                    amount: claimable,
                })
            }
        }
        None => Err(StdError::generic_err("Invalid holder")),
    }
}

/*NOTE Could be a situation where can_unbond returns true
 * but only partial balance available for unbond resulting
 * in stalled treasury trying to unbond more than is available
 */
pub fn unbondable<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
    holder: HumanAddr,
) -> StdResult<manager::QueryAnswer> {

    if let Some(full_asset) = assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        let config = config_r(&deps.storage).load()?;
        let allocations = match allocations_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
            Some(a) => a,
            None => { return Err(StdError::generic_err("Not an asset")); }
        };

        /*
        let unbonder = match holder {
            Some(h) => h,
            None => config.treasury,
        };
        */

        let mut balance = Uint128::zero();
        let mut unbonding = Uint128::zero();

        match holding_r(&deps.storage).may_load(&holder.as_str().as_bytes())? {
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
            /*
            if unbondable >= (balance - unbonding)? {
                unbondable = (balance - unbonding)?;
                break;
            }
            */
            unbondable += adapter::unbondable_query(&deps,
                                  &asset, alloc.contract.clone())?;
        }

        return Ok(manager::QueryAnswer::Unbondable {
            amount: unbondable,
        });
    }

    Err(StdError::generic_err("Not a registered asset"))
}

pub fn balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
    holder: HumanAddr,
) -> StdResult<manager::QueryAnswer> {

    match assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        Some(asset) => {
            /*
            let allocations = match allocations_r(&deps.storage).may_load(asset.contract.address.to_string().as_bytes())? {
                Some(a) => a,
                None => { return Err(StdError::generic_err("Not an asset")); }
            };
            */

            let holding = holding_r(&deps.storage).load(&holder.as_str().as_bytes())?;

            Ok(manager::QueryAnswer::Balance {
                amount: match holding.balances.iter().find(|u| u.token == asset.contract.address) {
                    Some(b) => b.amount,
                    None => Uint128::zero(),
                }
            })

        },
        None => Err(StdError::generic_err("Not a registered asset"))
    }
}

pub fn holders<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<treasury_manager::QueryAnswer> {
    Ok(treasury_manager::QueryAnswer::Holders {
        holders: holders_r(&deps.storage).load()?,
    })
}

pub fn holding<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    holder: HumanAddr,
) -> StdResult<treasury_manager::QueryAnswer> {
    match holding_r(&deps.storage).may_load(holder.as_str().as_bytes())? {
        Some(h) => Ok(treasury_manager::QueryAnswer::Holding { holding: h }),
        None => Err(StdError::generic_err("Not a holder")),
    }
}
