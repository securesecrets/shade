use crate::{
    c_std::{Addr, CosmosMsg, QuerierWrapper, StdError, StdResult, Uint128},
    utils::{asset::Contract, generic_response::ResponseStatus},
};

use crate::utils::{ExecuteCallback, Query};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub enum SubExecuteMsg {
    // Begin unbonding amount
    Unbond { asset: String, amount: Uint128 },
    Claim { asset: String },
    // Maintenance trigger e.g. claim rewards and restake
    Update { asset: String },
}

impl ExecuteCallback for SubExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    Adapter(SubExecuteMsg),
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteAnswer {
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

#[cw_serde]
pub enum SubQueryMsg {
    Balance { asset: String },
    Unbonding { asset: String },
    Claimable { asset: String },
    Unbondable { asset: String },
    Reserves { asset: String },
}

#[cw_serde]
pub enum QueryMsg {
    Adapter(SubQueryMsg),
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Balance { amount: Uint128 },
    Unbonding { amount: Uint128 },
    Claimable { amount: Uint128 },
    Unbondable { amount: Uint128 },
    Reserves { amount: Uint128 },
}

pub fn claimable_query(
    querier: QuerierWrapper,
    asset: &Addr,
    adapter: Contract,
) -> StdResult<Uint128> {
    match QueryMsg::Adapter(SubQueryMsg::Claimable {
        asset: asset.to_string().clone(),
    })
    .query(&querier, &adapter)?
    {
        QueryAnswer::Claimable { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query adapter claimable from {}",
            adapter.address
        ))),
    }
}

pub fn unbonding_query(
    querier: QuerierWrapper,
    asset: &Addr,
    adapter: Contract,
) -> StdResult<Uint128> {
    match QueryMsg::Adapter(SubQueryMsg::Unbonding {
        asset: asset.to_string().clone(),
    })
    .query(&querier, &adapter)?
    {
        QueryAnswer::Unbonding { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query adapter unbonding from {}",
            adapter.address
        ))),
    }
}

pub fn unbondable_query(
    querier: QuerierWrapper,
    asset: &Addr,
    adapter: Contract,
) -> StdResult<Uint128> {
    match QueryMsg::Adapter(SubQueryMsg::Unbondable {
        asset: asset.to_string().clone(),
    })
    .query(&querier, &adapter)?
    {
        QueryAnswer::Unbondable { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query adapter unbondable from {}",
            adapter.address
        ))),
    }
}

pub fn reserves_query(
    querier: QuerierWrapper,
    asset: &Addr,
    adapter: Contract,
) -> StdResult<Uint128> {
    match QueryMsg::Adapter(SubQueryMsg::Reserves {
        asset: asset.to_string().clone(),
    })
    .query(&querier, &adapter)?
    {
        QueryAnswer::Reserves { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query adapter unbondable from {}",
            adapter.address
        ))),
    }
}

pub fn balance_query(
    querier: QuerierWrapper,
    asset: &Addr,
    adapter: Contract,
) -> StdResult<Uint128> {
    match QueryMsg::Adapter(SubQueryMsg::Balance {
        asset: asset.to_string().clone(),
    })
    .query(&querier, &adapter)?
    {
        QueryAnswer::Balance { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query adapter balance from {}",
            adapter.address
        ))),
    }
}

pub fn claim_msg(asset: &Addr, adapter: Contract) -> StdResult<CosmosMsg> {
    ExecuteMsg::Adapter(SubExecuteMsg::Claim {
        asset: asset.to_string().clone(),
    })
    .to_cosmos_msg(&adapter, vec![])
}

pub fn unbond_msg(asset: &Addr, amount: Uint128, adapter: Contract) -> StdResult<CosmosMsg> {
    ExecuteMsg::Adapter(SubExecuteMsg::Unbond {
        asset: asset.to_string().clone(),
        amount,
    })
    .to_cosmos_msg(&adapter, vec![])
}

pub fn update_msg(asset: &Addr, adapter: Contract) -> StdResult<CosmosMsg> {
    ExecuteMsg::Adapter(SubExecuteMsg::Update {
        asset: asset.to_string().clone(),
    })
    .to_cosmos_msg(&adapter, vec![])
}
