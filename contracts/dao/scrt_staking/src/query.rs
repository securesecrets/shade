use shade_protocol::{
    c_std::{Addr, Delegation, Deps, StdError, StdResult, Uint128},
    dao::{adapter, scrt_staking::QueryAnswer},
    utils::asset::scrt_balance,
};

use crate::storage::*;

pub fn config(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: CONFIG.load(deps.storage)?,
    })
}

pub fn delegations(deps: Deps) -> StdResult<Vec<Delegation>> {
    deps.querier
        .query_all_delegations(SELF_ADDRESS.load(deps.storage)?)
}

pub fn rewards(deps: Deps) -> StdResult<Uint128> {
    let self_address = SELF_ADDRESS.load(deps.storage)?;

    let mut rewards = Uint128::zero();

    // TODO change to stargate query
    for d in deps.querier.query_all_delegations(self_address.clone())? {
        if let Some(delegation) = deps
            .querier
            .query_delegation(self_address.clone(), d.validator.clone())?
        {
            for coin in delegation.accumulated_rewards {
                if coin.denom != "uscrt" {
                    // TODO send to treasury
                    return Err(StdError::generic_err("Non-scrt coin in rewards!"));
                }
                rewards += coin.amount;
            }
        } else {
            return Err(StdError::generic_err(format!(
                "No delegation to {} but it was in storage",
                d.validator
            )));
        }
    }

    Ok(rewards)
}

pub fn balance(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let config = CONFIG.load(deps.storage)?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        )));
    }

    let delegated = Uint128::new(
        delegations(deps)?
            .into_iter()
            .map(|d| d.amount.amount.u128())
            .sum::<u128>(),
    );

    let scrt_balance = scrt_balance(deps, SELF_ADDRESS.load(deps.storage)?)?;

    let rewards = rewards(deps)?;

    Ok(adapter::QueryAnswer::Balance {
        amount: delegated + rewards + scrt_balance,
    })
}

pub fn claimable(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let config = CONFIG.load(deps.storage)?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        )));
    }

    let scrt_balance = scrt_balance(deps, SELF_ADDRESS.load(deps.storage)?)?;
    let rewards = rewards(deps)?;
    //assert!(false, "balance {}", scrt_balance);
    let unbonding = UNBONDING.load(deps.storage)?;
    //assert!(false, "unbonding {}", unbonding);

    let mut amount = scrt_balance + rewards;
    if amount > unbonding {
        amount = unbonding;
    }

    Ok(adapter::QueryAnswer::Claimable { amount })
}

pub fn unbonding(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let config = CONFIG.load(deps.storage)?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        )));
    }

    let scrt_balance = scrt_balance(deps, SELF_ADDRESS.load(deps.storage)?)?;

    let rewards = rewards(deps)?;
    let mut unbonding = UNBONDING.load(deps.storage)?;

    if unbonding >= (scrt_balance + rewards) {
        unbonding -= scrt_balance + rewards;
    } else {
        unbonding = Uint128::zero();
    }
    Ok(adapter::QueryAnswer::Unbonding { amount: unbonding })
}

pub fn unbondable(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let config = CONFIG.load(deps.storage)?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        )));
    }

    /* TODO: issues since we cant query unbondings
     *    While assets are unbonding they don't reflect anywhere in balance
     *    Once the unbonding funds are here they will show, making it difficult to present
     *    unbondable funds that arent being currently unbonded
     */
    let delegated = Uint128::new(
        delegations(deps)?
            .into_iter()
            .map(|d| d.amount.amount.u128())
            .sum::<u128>(),
    );

    let scrt_balance = scrt_balance(deps, SELF_ADDRESS.load(deps.storage)?)?;
    let rewards = rewards(deps)?;

    let unbonding = UNBONDING.load(deps.storage)?;

    let mut unbondable = delegated;

    if unbonding < scrt_balance + rewards {
        unbondable += scrt_balance + rewards - unbonding;
    }

    /*TODO: Query current unbondings
     * u >= 7 = 0
     * u <  7 = unbondable
     */
    Ok(adapter::QueryAnswer::Unbondable { amount: unbondable })
}

pub fn reserves(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let config = CONFIG.load(deps.storage)?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        )));
    }

    let scrt_balance = scrt_balance(deps, SELF_ADDRESS.load(deps.storage)?)?;

    let rewards = rewards(deps)?;

    if !scrt_balance.is_zero() {
        assert!(false, "scrt bal {}", scrt_balance);
    }
    Ok(adapter::QueryAnswer::Reserves {
        amount: scrt_balance + rewards,
    })
}
