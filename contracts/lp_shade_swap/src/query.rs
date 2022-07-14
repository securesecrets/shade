use shade_protocol::c_std::{
    Api, BalanceResponse, BankQuery, Delegation, DistQuery, DepsMut, FullDelegation, Addr,
    Querier, RewardsResponse, StdError, StdResult, Storage, Uint128,
};

use shade_protocol::{
    contract_interfaces::dao::{
        adapter, 
        lp_shade_swap::{is_supported_asset, get_supported_asset, QueryAnswer},
    },
    utils::asset::scrt_balance,
};

use shade_protocol::snip20::helpers::balance_query;

use crate::{
    state::{config_r, self_address_r, unbonding_r, viewing_key_r},
};

pub fn config<S: Storage, A: Api, Q: Querier>(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: config_r(&deps.storage).load()?,
    })
}

pub fn rewards<S: Storage, A: Api, Q: Querier>(deps: Deps) -> StdResult<Uint128> {
    //TODO: query pending rewards from rewards contract
    Ok(Uint128::zero())
}

pub fn balance<S: Storage, A: Api, Q: Querier>(
    deps: Deps,
    asset: Addr,
) -> StdResult<adapter::QueryAnswer> {

    let config = config_r(&deps.storage).load()?;

    if !is_supported_asset(&config, &asset) {
        return Err(StdError::generic_err(format!("Unrecognized Asset {}", asset)));
    }

    let mut balance = Uint128::zero();

    if vec![config.token_a.address, config.token_b.address].contains(&asset) {
        // Determine balance of LP, determine redemption value
    }
    else if config.liquidity_token.address == asset {
        // Check LP tokens in rewards contract + balance
    }

    Ok(adapter::QueryAnswer::Balance {
        amount: balance,
    })
}

pub fn claimable<S: Storage, A: Api, Q: Querier>(
    deps: Deps,
    asset: Addr,
) -> StdResult<adapter::QueryAnswer> {

    let config = config_r(&deps.storage).load()?;

    if !is_supported_asset(&config, &asset) {
        return Err(StdError::generic_err(format!("Unrecognized Asset {}", asset)));
    }

    let asset_contract = get_supported_asset(&config, &asset);

    let balance = balance_query(
        &deps.querier,
        self_address_r(&deps.storage).load()?,
        viewing_key_r(&deps.storage).load()?,
        1,
        asset_contract.code_hash.clone(),
        asset_contract.address.clone(),
    )?.amount;

    let mut claimable = unbonding_r(&deps.storage).load(asset.as_str().as_bytes())?;

    if balance < claimable {
        claimable = balance;
    }

    Ok(adapter::QueryAnswer::Claimable {
        amount: claimable,
    })
}

pub fn unbonding<S: Storage, A: Api, Q: Querier>(
    deps: Deps,
    asset: Addr,
) -> StdResult<adapter::QueryAnswer> {

    let config = config_r(&deps.storage).load()?;

    if !is_supported_asset(&config, &asset) {
        return Err(StdError::generic_err(format!("Unrecognized Asset {}", asset)));
    }

    Ok(adapter::QueryAnswer::Unbonding {
        amount: unbonding_r(&deps.storage).load(asset.as_str().as_bytes())?
    })
}

pub fn unbondable<S: Storage, A: Api, Q: Querier>(
    deps: Deps,
    asset: Addr,
) -> StdResult<adapter::QueryAnswer> {

    let config = config_r(&deps.storage).load()?;

    if !is_supported_asset(&config, &asset) {
        return Err(StdError::generic_err(format!("Unrecognized Asset {}", asset)));
    }

    let unbonding = unbonding_r(&deps.storage).load(asset.as_str().as_bytes())?;

    /* Need to check LP token redemption value
     */
    let unbondable = match balance(deps, asset)? {
        adapter::QueryAnswer::Balance { amount } => {
            if amount < unbonding {
                Uint128::zero()
            }
            else {
                (amount - unbonding)?
            }
        }
        _ => {
            return Err(StdError::generic_err("Failed to query balance"));
        }
    };

    Ok(adapter::QueryAnswer::Unbondable {
        amount: unbondable,
    })
}

pub fn reserves<S: Storage, A: Api, Q: Querier>(
    deps: Deps,
    asset: Addr,
) -> StdResult<adapter::QueryAnswer> {

    let config = config_r(&deps.storage).load()?;

    if !is_supported_asset(&config, &asset) {
        return Err(StdError::generic_err(format!("Unrecognized Asset {}", asset)));
    }

    let asset_contract = get_supported_asset(&config, &asset);

    let unbonding = unbonding_r(&deps.storage).load(asset.as_str().as_bytes())?;

    let balance = balance_query(
        &deps.querier,
        self_address_r(&deps.storage).load()?,
        viewing_key_r(&deps.storage).load()?,
        1,
        asset_contract.code_hash.clone(),
        asset_contract.address.clone(),
    )?.amount;

    if unbonding >= balance {
        return Ok(adapter::QueryAnswer::Reserves {
            amount: Uint128::zero(),
        });
    }
    else {
        return Ok(adapter::QueryAnswer::Reserves {
            amount: (balance - unbonding)?,
        });
    }

}
