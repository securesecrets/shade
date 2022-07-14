use shade_protocol::c_std::{
    Api,
    BalanceResponse,
    BankQuery,
    Delegation,
    DistQuery,
    DepsMut,
    FullDelegation,
    Addr,
    Querier,
    RewardsResponse,
    StdError,
    StdResult,
    Storage,
    Uint128,
};

use shade_protocol::{
    contract_interfaces::dao::rewards_emission::QueryAnswer,
    utils::asset::scrt_balance,
};

use shade_protocol::snip20::helpers::{allowance_query, balance_query};
use shade_protocol::contract_interfaces::dao::adapter;

use crate::state::{asset_r, assets_r, config_r, self_address_r, viewing_key_r};

pub fn config(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: config_r(&deps.storage).load()?,
    })
}

pub fn pending_allowance(
    deps: Deps,
    asset: Addr,
) -> StdResult<QueryAnswer> {
    let full_asset = match asset_r(&deps.storage).may_load(asset.as_str().as_bytes())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err(format!(
                "Unrecognized Asset {}",
                asset
            )));
        }
    };

    let config = config_r(&deps.storage).load()?;

    let allowance = allowance_query(
        &deps.querier,
        config.treasury,
        self_address_r(&deps.storage).load()?,
        viewing_key_r(&deps.storage).load()?,
        1,
        full_asset.contract.code_hash.clone(),
        full_asset.contract.address.clone(),
    )?
    .allowance;

    Ok(QueryAnswer::PendingAllowance { amount: allowance })
}

pub fn balance(
    deps: Deps,
    asset: Addr,
) -> StdResult<adapter::QueryAnswer> {
    let full_asset = match asset_r(&deps.storage).may_load(asset.as_str().as_bytes())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err(format!(
                "Unrecognized Asset {}",
                asset
            )));
        }
    };

    let balance = balance_query(
        &deps.querier,
        self_address_r(&deps.storage).load()?,
        viewing_key_r(&deps.storage).load()?,
        1,
        full_asset.contract.code_hash.clone(),
        full_asset.contract.address.clone(),
    )?
    .amount;

    Ok(adapter::QueryAnswer::Balance { amount: balance })
}
