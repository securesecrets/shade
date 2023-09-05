use shade_protocol::c_std::{
    Addr,
    Deps,
    StdError,
    StdResult,
    Uint128,
};

use shade_protocol::{
    contract_interfaces::dao::{
        adapter,
        lp_shdswap::{get_supported_asset, is_supported_asset, QueryAnswer},
    },
};

use shade_protocol::snip20::helpers::balance_query;

use crate::storage::*;

pub fn config(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: CONFIG.load(deps.storage)?,
    })
}

pub fn rewards(_deps: Deps) -> StdResult<Uint128> {
    //TODO: query pending rewards from rewards contract
    Ok(Uint128::zero())
}

pub fn balance(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let config = CONFIG.load(deps.storage)?;

    if !is_supported_asset(&config, &asset) {
        return Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        )));
    }

    let balance = Uint128::zero();

    if vec![config.token_a.address, config.token_b.address].contains(&asset) {
        // Determine balance of LP, determine redemption value
    } else if config.liquidity_token.address == asset {
        // Check LP tokens in rewards contract + balance
    }

    Ok(adapter::QueryAnswer::Balance { amount: balance })
}

pub fn claimable(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let config = CONFIG.load(deps.storage)?;

    if !is_supported_asset(&config, &asset) {
        return Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        )));
    }

    let asset_contract = get_supported_asset(&config, &asset);

    let balance = balance_query(
        &deps.querier,
        SELF_ADDRESS.load(deps.storage)?,
        VIEWING_KEY.load(deps.storage)?,
        &asset_contract,
    )?;

    let mut claimable = UNBONDING.load(deps.storage, asset.clone())?;

    if balance < claimable {
        claimable = balance;
    }

    Ok(adapter::QueryAnswer::Claimable { amount: claimable })
}

pub fn unbonding(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let config = CONFIG.load(deps.storage)?;

    if !is_supported_asset(&config, &asset) {
        return Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        )));
    }

    Ok(adapter::QueryAnswer::Unbonding {
        amount: UNBONDING.load(deps.storage, asset.clone())?,
    })
}

pub fn unbondable(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let config = CONFIG.load(deps.storage)?;

    if !is_supported_asset(&config, &asset) {
        return Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        )));
    }

    let unbonding = UNBONDING.load(deps.storage, asset.clone())?;

    /* Need to check LP token redemption value
     */
    let unbondable = match balance(deps, asset)? {
        adapter::QueryAnswer::Balance { amount } => {
            if amount < unbonding {
                Uint128::zero()
            } else {
                amount - unbonding
            }
        }
        _ => {
            return Err(StdError::generic_err("Failed to query balance"));
        }
    };

    Ok(adapter::QueryAnswer::Unbondable { amount: unbondable })
}

pub fn reserves(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let config = CONFIG.load(deps.storage)?;

    if !is_supported_asset(&config, &asset) {
        return Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        )));
    }

    let asset_contract = get_supported_asset(&config, &asset);

    let unbonding = UNBONDING.load(deps.storage, asset.clone())?;

    let balance = balance_query(
        &deps.querier,
        SELF_ADDRESS.load(deps.storage)?,
        VIEWING_KEY.load(deps.storage)?,
        &asset_contract,
    )?;

    if unbonding >= balance {
        return Ok(adapter::QueryAnswer::Reserves {
            amount: Uint128::zero(),
        });
    } else {
        return Ok(adapter::QueryAnswer::Reserves {
            amount: balance - unbonding,
        });
    }
}
