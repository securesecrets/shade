use crate::utils::{asset::Contract, generic_response::ResponseStatus};
use cosmwasm_std::{
    Api,
    Binary,
    CosmosMsg,
    Decimal,
    Delegation,
    Extern,
    HumanAddr,
    Querier,
    StdError,
    StdResult,
    Storage,
    Uint128,
    Validator,
};
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SubHandleMsg {
    // Begin unbonding amount
    Unbond { asset: HumanAddr, amount: Uint128 },
    Claim { asset: HumanAddr },
    // Maintenance trigger e.g. claim rewards and restake
    Update { asset: HumanAddr },
}

impl HandleCallback for SubHandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Manager(SubHandleMsg),
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Init {
        status: ResponseStatus,
        address: HumanAddr,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SubQueryMsg {
    Balance { asset: HumanAddr, holder: HumanAddr, },
    Unbonding { asset: HumanAddr, holder: HumanAddr, },
    Claimable { asset: HumanAddr, holder: HumanAddr,  },
    Unbondable { asset: HumanAddr, holder: HumanAddr, },
    Reserves { asset: HumanAddr, holder: HumanAddr, },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Manager(SubQueryMsg),
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Balance { amount: Uint128 },
    Unbonding { amount: Uint128 },
    Claimable { amount: Uint128 },
    Unbondable { amount: Uint128 },
    Reserves { amount: Uint128 },
}

pub fn claimable_query(
    deps: DepsMut,
    asset: &HumanAddr,
    holder: HumanAddr,
    manager: Contract,
) -> StdResult<Uint128> {
    match (QueryMsg::Manager(SubQueryMsg::Claimable {
        asset: asset.clone(),
        holder: holder.clone(),
    })
    .query(&deps.querier, manager.code_hash, manager.address.clone())?)
    {
        QueryAnswer::Claimable { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query manager claimable from {}",
            manager.address
        ))),
    }
}

pub fn unbonding_query(
    deps: DepsMut,
    asset: &HumanAddr,
    holder: HumanAddr,
    manager: Contract,
) -> StdResult<Uint128> {
    match (QueryMsg::Manager(SubQueryMsg::Unbonding {
        asset: asset.clone(),
        holder: holder.clone(),
    })
    .query(&deps.querier, manager.code_hash, manager.address.clone())?)
    {
        QueryAnswer::Unbonding { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query manager unbonding from {}",
            manager.address
        ))),
    }
}

pub fn unbondable_query(
    deps: DepsMut,
    asset: &HumanAddr,
    holder: HumanAddr,
    manager: Contract,
) -> StdResult<Uint128> {
    match (QueryMsg::Manager(SubQueryMsg::Unbondable {
        asset: asset.clone(),
        holder: holder.clone(),
    })
    .query(&deps.querier, manager.code_hash, manager.address.clone())?)
    {
        QueryAnswer::Unbondable { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query manager unbondable from {}",
            manager.address
        ))),
    }
}

pub fn reserves_query(
    deps: DepsMut,
    asset: &HumanAddr,
    holder: HumanAddr,
    manager: Contract,
) -> StdResult<Uint128> {

    match (QueryMsg::Manager(SubQueryMsg::Reserves {
        asset: asset.clone(),
        holder: holder.clone(),
    }).query(&deps.querier, manager.code_hash, manager.address.clone())?) {
        QueryAnswer::Reserves { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            format!("Failed to query manager unbondable from {}", manager.address)
        ))
    }
}

pub fn balance_query(
    deps: DepsMut,
    asset: &HumanAddr,
    holder: HumanAddr,
    manager: Contract,
) -> StdResult<Uint128> {
    match (QueryMsg::Manager(SubQueryMsg::Balance {
        asset: asset.clone(),
        holder: holder.clone(),
    })
    .query(&deps.querier, manager.code_hash, manager.address.clone())?)
    {
        QueryAnswer::Balance { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query manager balance from {}",
            manager.address
        ))),
    }
}

pub fn claim_msg(asset: HumanAddr, manager: Contract) -> StdResult<CosmosMsg> {
    HandleMsg::Manager(SubHandleMsg::Claim { asset }).to_cosmos_msg(
        manager.code_hash,
        manager.address,
        None,
    )
}

pub fn unbond_msg(asset: HumanAddr, amount: Uint128, manager: Contract) -> StdResult<CosmosMsg> {
    HandleMsg::Manager(SubHandleMsg::Unbond { asset, amount }).to_cosmos_msg(
        manager.code_hash,
        manager.address,
        None,
    )
}

pub fn update_msg(asset: HumanAddr, manager: Contract) -> StdResult<CosmosMsg> {
    HandleMsg::Manager(SubHandleMsg::Update { asset }).to_cosmos_msg(
        manager.code_hash,
        manager.address,
        None,
    )
}
