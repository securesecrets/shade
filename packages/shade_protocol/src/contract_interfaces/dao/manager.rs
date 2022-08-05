use cosmwasm_std::{
    Api,
    QuerierWrapper,
    DepsMut,
    Deps,
    Binary,
    CosmosMsg,
    Decimal,
    Delegation,
    Addr,
    Querier,
    StdError,
    StdResult,
    Storage,
    Uint128,
    Validator,
};

use crate::utils::{
    asset::Contract, 
    generic_response::ResponseStatus,
    ExecuteCallback, Query,
};
use cosmwasm_schema::{cw_serde};

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
    Manager(SubExecuteMsg),
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
    Balance { asset: Addr, holder: Addr, },
    Unbonding { asset: Addr, holder: Addr, },
    Claimable { asset: Addr, holder: Addr,  },
    Unbondable { asset: Addr, holder: Addr, },
    Reserves { asset: Addr, holder: Addr, },
}

#[cw_serde]
pub enum QueryMsg {
    Manager(SubQueryMsg),
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
    holder: Addr,
    manager: Contract,
) -> StdResult<Uint128> {
    match (QueryMsg::Manager(SubQueryMsg::Claimable {
        asset: asset.clone(),
        holder: holder.clone(),
    })
    .query(&querier, &manager)?)
    {
        QueryAnswer::Claimable { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query manager claimable from {}",
            manager.address
        ))),
    }
}

pub fn unbonding_query(
    querier: QuerierWrapper,
    asset: &Addr,
    holder: Addr,
    manager: Contract,
) -> StdResult<Uint128> {
    match (QueryMsg::Manager(SubQueryMsg::Unbonding {
        asset: asset.clone(),
        holder: holder.clone(),
    })
    .query(&querier, &manager)?)
    {
        QueryAnswer::Unbonding { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query manager unbonding from {}",
            manager.address
        ))),
    }
}

pub fn unbondable_query(
    querier: QuerierWrapper,
    asset: &Addr,
    holder: Addr,
    manager: Contract,
) -> StdResult<Uint128> {
    match (QueryMsg::Manager(SubQueryMsg::Unbondable {
        asset: asset.clone(),
        holder: holder.clone(),
    })
    .query(&querier, &manager)?)
    {
        QueryAnswer::Unbondable { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query manager unbondable from {}",
            manager.address
        ))),
    }
}

pub fn reserves_query(
    querier: QuerierWrapper,
    asset: &Addr,
    holder: Addr,
    manager: Contract,
) -> StdResult<Uint128> {

    match (QueryMsg::Manager(SubQueryMsg::Reserves {
        asset: asset.clone(),
        holder: holder.clone(),
    }).query(&querier, &manager)?) {
        QueryAnswer::Reserves { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            format!("Failed to query manager unbondable from {}", manager.address)
        ))
    }
}

pub fn balance_query(
    querier: QuerierWrapper,
    asset: &Addr,
    holder: Addr,
    manager: Contract,
) -> StdResult<Uint128> {
    match (QueryMsg::Manager(SubQueryMsg::Balance {
        asset: asset.clone(),
        holder: holder.clone(),
    })
    .query(&querier, &manager)?)
    {
        QueryAnswer::Balance { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query manager balance from {}",
            manager.address
        ))),
    }
}

pub fn claim_msg(asset: &Addr, manager: Contract) -> StdResult<CosmosMsg> {
    ExecuteMsg::Manager(SubExecuteMsg::Claim { asset: asset.clone().to_string() }).to_cosmos_msg(
        &manager,
        vec![],
    )
}

pub fn unbond_msg(asset: &Addr, amount: Uint128, manager: Contract) -> StdResult<CosmosMsg> {
    ExecuteMsg::Manager(SubExecuteMsg::Unbond { asset: asset.clone().to_string(), amount }).to_cosmos_msg(
        &manager,
        vec![],
    )
}

pub fn update_msg(asset: &Addr, manager: Contract) -> StdResult<CosmosMsg> {
    ExecuteMsg::Manager(SubExecuteMsg::Update { asset: asset.clone().to_string() }).to_cosmos_msg(
        &manager,
        vec![],
    )
}
