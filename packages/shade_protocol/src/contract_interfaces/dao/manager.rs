use cosmwasm_std::{
    Addr,
    CosmosMsg,
    QuerierWrapper,
    StdError,
    StdResult,
    Uint128,
};

use crate::utils::{asset::Contract, generic_response::ResponseStatus, ExecuteCallback, Query};
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
    Manager(SubExecuteMsg),
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteAnswer {
    Init {
        status: ResponseStatus,
        address: String,
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
    BatchBalance { assets: Vec<String>, holder: String },
    Balance { asset: String, holder: String },
    Unbonding { asset: String, holder: String },
    Claimable { asset: String, holder: String },
    Unbondable { asset: String, holder: String },
    Reserves { asset: String, holder: String },
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
    BatchBalance { amounts: Vec<Uint128> },
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
    match QueryMsg::Manager(SubQueryMsg::Claimable {
        asset: asset.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .query(&querier, &manager)?
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
    match QueryMsg::Manager(SubQueryMsg::Unbonding {
        asset: asset.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .query(&querier, &manager)?
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
    match QueryMsg::Manager(SubQueryMsg::Unbondable {
        asset: asset.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .query(&querier, &manager)?
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
    match QueryMsg::Manager(SubQueryMsg::Reserves {
        asset: asset.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .query(&querier, &manager)?
    {
        QueryAnswer::Reserves { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query manager unbondable from {}",
            manager.address
        ))),
    }
}

pub fn balance_query(
    querier: QuerierWrapper,
    asset: &Addr,
    holder: Addr,
    manager: Contract,
) -> StdResult<Uint128> {
    match QueryMsg::Manager(SubQueryMsg::Balance {
        asset: asset.to_string().clone(),
        holder: holder.to_string().clone(),
    })
    .query(&querier, &manager)
    {
        Ok(resp) => match resp {
            QueryAnswer::Balance { amount } => Ok(amount),
            _ => Err(StdError::generic_err(format!(
                "Unexpected response from {} manager balance",
                manager.address
            ))),
        },
        Err(e) => {
            println!("HERERERER");
            return Err(StdError::generic_err(format!(
                "Failed to query manager balance: {}",
                e.to_string()
            )));
        }
    }
}

pub fn batch_balance_query(
    querier: QuerierWrapper,
    assets: &Vec<Addr>,
    holder: Addr,
    manager: Contract,
) -> StdResult<Vec<Uint128>> {
    match QueryMsg::Manager(SubQueryMsg::BatchBalance {
        assets: assets.iter().map(|a| a.to_string()).collect(),
        holder: holder.to_string().clone(),
    })
    .query(&querier, &manager)
    {
        Ok(resp) => match resp {
            QueryAnswer::BatchBalance { amounts } => Ok(amounts),
            _ => Err(StdError::generic_err(format!(
                "Unexpected response from {} manager batch balance",
                manager.address
            ))),
        },
        Err(e) => {
            return Err(StdError::generic_err(format!(
                "Failed to query manager batch balance: {}",
                e.to_string()
            )));
        }
    }
}

pub fn claim_msg(asset: &Addr, manager: Contract) -> StdResult<CosmosMsg> {
    ExecuteMsg::Manager(SubExecuteMsg::Claim {
        asset: asset.to_string().clone(),
    })
    .to_cosmos_msg(&manager, vec![])
}

pub fn unbond_msg(asset: &Addr, amount: Uint128, manager: Contract) -> StdResult<CosmosMsg> {
    ExecuteMsg::Manager(SubExecuteMsg::Unbond {
        asset: asset.to_string().clone(),
        amount,
    })
    .to_cosmos_msg(&manager, vec![])
}

pub fn update_msg(asset: &Addr, manager: Contract) -> StdResult<CosmosMsg> {
    ExecuteMsg::Manager(SubExecuteMsg::Update {
        asset: asset.to_string().clone(),
    })
    .to_cosmos_msg(&manager, vec![])
}
