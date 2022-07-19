use cosmwasm_std::Deps;
use crate::utils::{asset::Contract, generic_response::ResponseStatus};
use crate::c_std::{
    Api,
    Binary,
    CosmosMsg,
    Decimal,
    Delegation,
    DepsMut,
    Addr,
    Querier,
    StdError,
    StdResult,
    Storage,
    Uint128,
    Validator,
};

use crate::utils::{ExecuteCallback, InstantiateCallback, Query};
use cosmwasm_schema::{cw_serde};

#[cw_serde]
pub enum SubHandleMsg {
    // Begin unbonding amount
    Unbond { asset: Addr, amount: Uint128 },
    Claim { asset: Addr },
    // Maintenance trigger e.g. claim rewards and restake
    Update { asset: Addr },
}

impl ExecuteCallback for SubHandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    Adapter(SubHandleMsg),
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
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

#[cw_serde]
pub enum SubQueryMsg {
    Balance { asset: Addr },
    Unbonding { asset: Addr },
    Claimable { asset: Addr },
    Unbondable { asset: Addr },
    Reserves { asset: Addr },
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
    deps: Deps,
    asset: &Addr,
    adapter: Contract,
) -> StdResult<Uint128> {
    match QueryMsg::Adapter(SubQueryMsg::Claimable {
        asset: asset.clone(),
    })
    .query(&deps.querier, &adapter)?
    {
        QueryAnswer::Claimable { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query adapter claimable from {}",
            adapter.address
        ))),
    }
}

pub fn unbonding_query(
    deps: Deps,
    asset: &Addr,
    adapter: Contract,
) -> StdResult<Uint128> {
    match QueryMsg::Adapter(SubQueryMsg::Unbonding {
        asset: asset.clone(),
    })
    .query(&deps.querier, &adapter)?
    {
        QueryAnswer::Unbonding { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query adapter unbonding from {}",
            adapter.address
        ))),
    }
}

pub fn unbondable_query(
    deps: Deps,
    asset: &Addr,
    adapter: Contract,
) -> StdResult<Uint128> {
    match QueryMsg::Adapter(SubQueryMsg::Unbondable {
        asset: asset.clone(),
    })
    .query(&deps.querier, &adapter)?
    {
        QueryAnswer::Unbondable { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query adapter unbondable from {}",
            adapter.address
        ))),
    }
}

pub fn reserves_query(
    deps: Deps,
    asset: &Addr,
    adapter: Contract,
) -> StdResult<Uint128> {

    match QueryMsg::Adapter(SubQueryMsg::Reserves {
        asset: asset.clone(),
    }).query(&deps.querier, &adapter)? {
        QueryAnswer::Reserves { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            format!("Failed to query adapter unbondable from {}", adapter.address)
        ))
    }
}

pub fn balance_query(
    deps: Deps,
    asset: &Addr,
    adapter: Contract,
) -> StdResult<Uint128> {
    match QueryMsg::Adapter(SubQueryMsg::Balance {
        asset: asset.clone(),
    })
    .query(&deps.querier, &adapter)?
    {
        QueryAnswer::Balance { amount } => Ok(amount),
        _ => Err(StdError::generic_err(format!(
            "Failed to query adapter balance from {}",
            adapter.address
        ))),
    }
}

pub fn claim_msg(asset: Addr, adapter: Contract) -> StdResult<CosmosMsg> {
    ExecuteMsg::Adapter(SubHandleMsg::Claim { asset }).to_cosmos_msg(
        &adapter,
        vec![],
    )
}

pub fn unbond_msg(asset: Addr, amount: Uint128, adapter: Contract) -> StdResult<CosmosMsg> {
    ExecuteMsg::Adapter(SubHandleMsg::Unbond { asset, amount }).to_cosmos_msg(
        &adapter,
        vec![],
    )
}

pub fn update_msg(asset: Addr, adapter: Contract) -> StdResult<CosmosMsg> {
    ExecuteMsg::Adapter(SubHandleMsg::Update { asset }).to_cosmos_msg(
        &adapter,
        vec![],
    )
}
