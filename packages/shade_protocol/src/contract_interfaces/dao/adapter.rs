use crate::utils::{asset::Contract, generic_response::ResponseStatus};
use crate::c_std::{
    Api,
    Binary,
    CosmosMsg,
    Decimal,
    Delegation,
    Extern,
    Addr,
    Querier,
    StdError,
    StdResult,
    Storage,
    Uint128,
    Validator,
};

use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use crate::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SubHandleMsg {
    // Begin unbonding amount
    Unbond { asset: Addr, amount: Uint128 },
    Claim { asset: Addr },
    // Maintenance trigger e.g. claim rewards and restake
    Update { asset: Addr },
}

impl HandleCallback for SubHandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Adapter(SubHandleMsg),
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Init {
        status: ResponseStatus,
        address: Addr,
    },
    Unbond {
        status: ResponseStatus,
        amount: Uint128,
    },
    Claim {
        status: ResponseStatus,
        amount: Uint128,
    },
    Update {
        status: ResponseStatus,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SubQueryMsg {
    Balance { asset: Addr },
    Unbonding { asset: Addr },
    Claimable { asset: Addr },
    Unbondable { asset: Addr },
    Reserves { asset: Addr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Adapter(SubQueryMsg),
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Balance { amount: Uint128 },
    Unbonding { amount: Uint128 },
    Claimable { amount: Uint128 },
    Unbondable { amount: Uint128 },
    Reserves { amount: Uint128 },
}

pub fn claimable_query(
    deps: &Deps,
    asset: &Addr,
    adapter: Contract,
) -> StdResult<Uint128> {
    match (QueryMsg::Adapter(SubQueryMsg::Claimable {
        asset: asset.clone(),
    })
    .query(&deps.querier, adapter.code_hash, adapter.address.clone())?)
    {
        QueryAnswer::Claimable { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query adapter claimable from {}",
            adapter.address
        ))),
    }
}

pub fn unbonding_query(
    deps: &Deps,
    asset: &Addr,
    adapter: Contract,
) -> StdResult<Uint128> {
    match (QueryMsg::Adapter(SubQueryMsg::Unbonding {
        asset: asset.clone(),
    })
    .query(&deps.querier, adapter.code_hash, adapter.address.clone())?)
    {
        QueryAnswer::Unbonding { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query adapter unbonding from {}",
            adapter.address
        ))),
    }
}

pub fn unbondable_query(
    deps: &Deps,
    asset: &Addr,
    adapter: Contract,
) -> StdResult<Uint128> {
    match (QueryMsg::Adapter(SubQueryMsg::Unbondable {
        asset: asset.clone(),
    })
    .query(&deps.querier, adapter.code_hash, adapter.address.clone())?)
    {
        QueryAnswer::Unbondable { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query adapter unbondable from {}",
            adapter.address
        ))),
    }
}

pub fn reserves_query(
    deps: &Deps,
    asset: &Addr,
    adapter: Contract,
) -> StdResult<Uint128> {

    match (QueryMsg::Adapter(SubQueryMsg::Reserves {
        asset: asset.clone(),
    }).query(&deps.querier, adapter.code_hash, adapter.address.clone())?) {
        QueryAnswer::Reserves { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            format!("Failed to query adapter unbondable from {}", adapter.address)
        ))
    }
}

pub fn balance_query(
    deps: &Deps,
    asset: &Addr,
    adapter: Contract,
) -> StdResult<Uint128> {
    match (QueryMsg::Adapter(SubQueryMsg::Balance {
        asset: asset.clone(),
    })
    .query(&deps.querier, adapter.code_hash, adapter.address.clone())?)
    {
        QueryAnswer::Balance { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query adapter balance from {}",
            adapter.address
        ))),
    }
}

pub fn claim_msg(asset: Addr, adapter: Contract) -> StdResult<CosmosMsg> {
    Ok(
        HandleMsg::Adapter(SubHandleMsg::Claim { asset }).to_cosmos_msg(
            adapter.code_hash,
            adapter.address,
            None,
        )?,
    )
}

pub fn unbond_msg(asset: Addr, amount: Uint128, adapter: Contract) -> StdResult<CosmosMsg> {
    Ok(
        HandleMsg::Adapter(SubHandleMsg::Unbond { asset, amount }).to_cosmos_msg(
            adapter.code_hash,
            adapter.address,
            None,
        )?,
    )
}

pub fn update_msg(asset: Addr, adapter: Contract) -> StdResult<CosmosMsg> {
    Ok(
        HandleMsg::Adapter(SubHandleMsg::Update { asset }).to_cosmos_msg(
            adapter.code_hash,
            adapter.address,
            None,
        )?,
    )
}
