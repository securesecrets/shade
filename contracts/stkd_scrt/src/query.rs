use shade_protocol::{
    c_std::{
        Addr,
        Api,
        BankQuery,
        Delegation,
        Deps,
        DistributionMsg,
        FullDelegation,
        Querier,
        StdError,
        StdResult,
        Storage,
        Uint128,
    },
    dao::{adapter, stkd_scrt::{QueryAnswer, staking_derivatives},
    snip20::helpers::balance_query,
    utils::asset::scrt_balance,
};

use crate::storage::*;

pub fn config(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: CONFIG.load(deps.storage)?,
    })
}

pub fn balance(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
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
        &config.staking_derivatives,
    )?;

    Ok(adapter::QueryAnswer::Balance {
        amount: holdings.claimable_scrt
            + holdings.unbonding_scrt
            + holdings.token_balance_value_in_scrt,
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

    let holdings = staking_derivatives::holdings_query(
        &deps.querier,
        env.contract.address,
        &config.staking_derivatives,
    )?;

    Ok(adapter::QueryAnswer::Claimable {
        amount: holdings.claimable_scrt,
    })
}

pub fn unbonding(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
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
        &config.staking_derivatives,
    )?;

    Ok(adapter::QueryAnswer::Unbonding {
        amount: holdings.unbonding_scrt,
    })
}

pub fn unbondable(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
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
        &config.staking_derivatives,
    )?;

    Ok(adapter::QueryAnswer::Unbondable {
        amount: holdings.token_balance_value_in_scrt,
    })
}

pub fn reserves(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
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
