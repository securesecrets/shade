use shade_protocol::{
    c_std::{Addr, Deps, Env, StdError, StdResult, Uint128},
    dao::{
        adapter,
        stkd_scrt::{staking_derivatives, QueryAnswer},
    },
};

use crate::storage::*;

pub fn config(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: CONFIG.load(deps.storage)?,
    })
}

pub fn balance(deps: Deps, env: Env, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let config = CONFIG.load(deps.storage)?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        )));
    }

    let holdings = staking_derivatives::holdings_query(
        &deps.querier,
        env.contract.address,
        VIEWING_KEY.load(deps.storage)?,
        env.block.time.seconds(),
        &config.staking_derivatives,
    )?;

    Ok(adapter::QueryAnswer::Balance {
        amount: holdings.claimable_scrt
            + holdings.unbonding_scrt
            + holdings.token_balance_value_in_scrt,
    })
}

pub fn claimable(deps: Deps, env: Env, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let config = CONFIG.load(deps.storage)?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        )));
    }

    let holdings = staking_derivatives::holdings_query(
        &deps.querier,
        env.contract.address,
        VIEWING_KEY.load(deps.storage)?,
        env.block.time.seconds(),
        &config.staking_derivatives,
    )?;

    Ok(adapter::QueryAnswer::Claimable {
        amount: holdings.claimable_scrt,
    })
}

pub fn unbonding(deps: Deps, env: Env, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let config = CONFIG.load(deps.storage)?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        )));
    }

    let holdings = staking_derivatives::holdings_query(
        &deps.querier,
        env.contract.address,
        VIEWING_KEY.load(deps.storage)?,
        env.block.time.seconds(),
        &config.staking_derivatives,
    )?;

    Ok(adapter::QueryAnswer::Unbonding {
        amount: holdings.unbonding_scrt,
    })
}

pub fn unbondable(deps: Deps, env: Env, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let config = CONFIG.load(deps.storage)?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        )));
    }

    let holdings = staking_derivatives::holdings_query(
        &deps.querier,
        env.contract.address,
        VIEWING_KEY.load(deps.storage)?,
        env.block.time.seconds(),
        &config.staking_derivatives,
    )?;

    Ok(adapter::QueryAnswer::Unbondable {
        amount: holdings.token_balance_value_in_scrt,
    })
}

pub fn reserves(deps: Deps, _env: Env, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let config = CONFIG.load(deps.storage)?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        )));
    }

    Ok(adapter::QueryAnswer::Reserves {
        amount: Uint128::zero(),
    })
}
