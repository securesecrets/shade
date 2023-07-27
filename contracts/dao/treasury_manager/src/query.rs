use crate::storage::*;
use shade_protocol::{
    c_std::{Addr, Deps, Env, StdError, StdResult, Uint128},
    dao::{adapter, manager, treasury_manager},
    snip20::helpers::{allowance_query, balance_query},
    utils::{cycle::parse_utc_datetime, storage::plus::period_storage::Period},
};

pub fn config(deps: Deps) -> StdResult<treasury_manager::QueryAnswer> {
    Ok(treasury_manager::QueryAnswer::Config {
        config: CONFIG.load(deps.storage)?,
    })
}

pub fn metrics(
    deps: Deps,
    env: Env,
    date: Option<String>,
    epoch: Option<Uint128>,
    period: Period,
) -> StdResult<treasury_manager::QueryAnswer> {
    if date.is_some() && epoch.is_some() {
        return Err(StdError::generic_err("cannot pass both epoch and date"));
    }
    let key = {
        if let Some(d) = date {
            parse_utc_datetime(&d)?.timestamp() as u64
        } else if let Some(e) = epoch {
            e.u128() as u64
        } else {
            env.block.time.seconds()
        }
    };
    Ok(treasury_manager::QueryAnswer::Metrics {
        metrics: METRICS.load_period(deps.storage, key, period)?,
    })
}

pub fn pending_allowance(
    deps: Deps,
    env: Env,
    asset: Addr,
) -> StdResult<treasury_manager::QueryAnswer> {
    let config = CONFIG.load(deps.storage)?;
    let full_asset = match ASSETS.may_load(deps.storage, asset)? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Not a registered asset"));
        }
    };

    let allowance = allowance_query(
        &deps.querier,
        config.treasury,
        env.contract.address,
        VIEWING_KEY.load(deps.storage)?,
        1,
        &full_asset.contract.clone(),
    )?
    .allowance;

    Ok(treasury_manager::QueryAnswer::PendingAllowance { amount: allowance })
}

pub fn reserves(
    deps: Deps,
    env: Env,
    asset: Addr,
    _holder: Addr,
) -> StdResult<manager::QueryAnswer> {
    if let Some(full_asset) = ASSETS.may_load(deps.storage, asset)? {
        let reserves = balance_query(
            &deps.querier,
            env.contract.address,
            VIEWING_KEY.load(deps.storage)?,
            &full_asset.contract.clone(),
        )?;

        return Ok(manager::QueryAnswer::Reserves { amount: reserves });
    }

    Err(StdError::generic_err("Not a registered asset"))
}

pub fn assets(deps: Deps) -> StdResult<treasury_manager::QueryAnswer> {
    Ok(treasury_manager::QueryAnswer::Assets {
        assets: ASSET_LIST.load(deps.storage)?,
    })
}

pub fn allocations(deps: Deps, asset: Addr) -> StdResult<treasury_manager::QueryAnswer> {
    Ok(treasury_manager::QueryAnswer::Allocations {
        allocations: match ALLOCATIONS.may_load(deps.storage, asset)? {
            None => vec![],
            Some(a) => a,
        },
    })
}

pub fn unbonding(deps: Deps, asset: Addr, holder: Addr) -> StdResult<manager::QueryAnswer> {
    if ASSETS.may_load(deps.storage, asset.clone())?.is_none() {
        return Err(StdError::generic_err("Not a registered asset"));
    }

    let _config = CONFIG.load(deps.storage)?;

    match HOLDING.may_load(deps.storage, holder)? {
        Some(holder) => Ok(manager::QueryAnswer::Unbonding {
            amount: match holder.unbondings.iter().find(|u| u.token == asset.clone()) {
                Some(u) => u.amount,
                None => Uint128::zero(),
            },
        }),
        None => {
            return Err(StdError::generic_err("Invalid holder"));
        }
    }
}

pub fn claimable(
    deps: Deps,
    env: Env,
    asset: Addr,
    holder: Addr,
) -> StdResult<manager::QueryAnswer> {
    let full_asset = match ASSETS.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Not a registered asset"));
        }
    };
    let allocations = match ALLOCATIONS.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => vec![],
    };
    //TODO claiming needs ordered unbondings so other holders don't get bumped

    let mut claimable = balance_query(
        &deps.querier,
        env.contract.address,
        VIEWING_KEY.load(deps.storage)?,
        &full_asset.contract.clone(),
    )?;

    for alloc in allocations {
        claimable += adapter::claimable_query(deps.querier, &asset, alloc.contract.clone())?;
    }

    match HOLDING.may_load(deps.storage, holder)? {
        Some(holder) => {
            let unbonding = match holder.unbondings.iter().find(|u| u.token == asset) {
                Some(u) => u.amount,
                None => Uint128::zero(),
            };

            if claimable > unbonding {
                Ok(manager::QueryAnswer::Claimable { amount: unbonding })
            } else {
                Ok(manager::QueryAnswer::Claimable { amount: claimable })
            }
        }
        None => Err(StdError::generic_err("Invalid holder")),
    }
}

pub fn unbondable(
    deps: Deps,
    env: Env,
    asset: Addr,
    holder: Addr,
) -> StdResult<manager::QueryAnswer> {
    let full_asset = match ASSETS.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Not a registered asset"));
        }
    };
    let mut holder_balance = Uint128::zero();

    match HOLDING.may_load(deps.storage, holder.clone())? {
        Some(h) => {
            if let Some(b) = h.balances.iter().find(|b| b.token == asset.clone()) {
                holder_balance += b.amount;
            }
        }
        None => {
            return Err(StdError::generic_err("Invalid holder"));
        }
    }

    if holder_balance.is_zero() {
        return Ok(manager::QueryAnswer::Unbondable {
            amount: holder_balance,
        });
    }

    let mut unbondable = balance_query(
        &deps.querier,
        env.contract.address,
        VIEWING_KEY.load(deps.storage)?,
        &full_asset.contract.clone(),
    )?;

    let allocations = ALLOCATIONS
        .may_load(deps.storage, asset.clone())?
        .unwrap_or(vec![]);

    for alloc in allocations {
        unbondable += adapter::unbondable_query(deps.querier, &asset, alloc.contract)?;
        if unbondable > holder_balance {
            break;
        }
    }

    if unbondable > holder_balance {
        unbondable = holder_balance;
    }

    return Ok(manager::QueryAnswer::Unbondable { amount: unbondable });
}

pub fn batch_balance(
    deps: Deps,
    assets: Vec<Addr>,
    holder: Addr,
) -> StdResult<manager::QueryAnswer> {
    let holding = match HOLDING.may_load(deps.storage, holder.clone())? {
        Some(h) => h,
        None => {
            return Err(StdError::generic_err("Invalid Holder"));
        }
    };

    let mut balances = vec![];

    for asset in assets {
        if let Some(asset) = ASSETS.may_load(deps.storage, asset)? {
            balances.push(
                match holding
                    .balances
                    .iter()
                    .find(|b| b.token == asset.contract.address)
                {
                    Some(b) => b.amount,
                    None => Uint128::zero(),
                },
            );
        } else {
            balances.push(Uint128::zero());
        }
    }

    Ok(manager::QueryAnswer::BatchBalance { amounts: balances })
}

pub fn balance(deps: Deps, asset: Addr, holder: Addr) -> StdResult<manager::QueryAnswer> {
    if let Some(asset) = ASSETS.may_load(deps.storage, asset)? {
        let holding = match HOLDING.may_load(deps.storage, holder.clone())? {
            Some(h) => h,
            None => {
                return Err(StdError::generic_err("Invalid Holder"));
            }
        };
        // TODO include unbonding so balance is more 'stable'
        //      likely requires treasury rebalance changes
        let balance = match holding
            .balances
            .iter()
            .find(|b| b.token == asset.contract.address)
        {
            Some(b) => b.amount,
            None => Uint128::zero(),
        };

        Ok(manager::QueryAnswer::Balance { amount: balance })
    } else {
        Err(StdError::generic_err("Not a registered asset"))
    }
}

pub fn holders(deps: Deps) -> StdResult<treasury_manager::QueryAnswer> {
    Ok(treasury_manager::QueryAnswer::Holders {
        holders: HOLDERS.load(deps.storage)?,
    })
}

pub fn holding(deps: Deps, holder: Addr) -> StdResult<treasury_manager::QueryAnswer> {
    match HOLDING.may_load(deps.storage, holder)? {
        Some(h) => Ok(treasury_manager::QueryAnswer::Holding { holding: h }),
        None => Err(StdError::generic_err("Not a holder")),
    }
}
