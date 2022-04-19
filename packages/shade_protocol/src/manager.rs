use crate::utils::{
    asset::Contract, 
    generic_response::ResponseStatus,
    unbonding::{UnbondStatus, Unbonding},
};
use cosmwasm_std::{Binary, Decimal, Delegation, HumanAddr, Uint128, Validator, StdResult, CosmosMsg, StdError, Storage, Api, Querier, Extern};
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SubHandleMsg {
    // unbond amount back to treasury
    Unbond {
        asset: HumanAddr,
        amount: Uint128, 
    },
    Claim {
        asset: HumanAddr,
    },
    Rebalance { 
        asset: HumanAddr,
    },
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
        //address: HumanAddr,
    },
    Response { 
        status: ResponseStatus,
    },

    Rebalance {
        status: ResponseStatus,
        unbond: Uint128,
        input: Uint128,
    },
    Claim {
        status: ResponseStatus,
        amount: Uint128,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SubQueryMsg {
    Balance { asset: HumanAddr },
    Unbonding { asset: HumanAddr },
    Claimable { asset: HumanAddr },
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
}

pub fn claimable_query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: &HumanAddr,
    manager: Contract,
) -> StdResult<Uint128> {

    match (QueryMsg::Manager(SubQueryMsg::Claimable {
        asset: asset.clone(),
    }).query(&deps.querier, manager.code_hash, manager.address.clone())?) {
        QueryAnswer::Claimable { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            format!("Failed to query manager claimable from {}", manager.address)
        ))
    }
}

pub fn unbonding_query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: &HumanAddr,
    manager: Contract,
) -> StdResult<Uint128> {

    match (QueryMsg::Manager(SubQueryMsg::Unbonding {
        asset: asset.clone(),
    }).query(&deps.querier, manager.code_hash, manager.address.clone())?) {
        QueryAnswer::Unbonding { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            format!("Failed to query manager claimable from {}", manager.address)
        ))
    }
}

pub fn balance_query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: &HumanAddr,
    manager: Contract,
) -> StdResult<Uint128> {

    match (QueryMsg::Manager(
            SubQueryMsg::Balance {
                asset: asset.clone(),
            }
        ).query(&deps.querier, manager.code_hash, manager.address.clone())?) {
        QueryAnswer::Balance { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            format!("Failed to query manager balance from {}", manager.address)
        ))
    }
}

pub fn claim_msg(
    asset: HumanAddr,
    manager: Contract,
) -> StdResult<CosmosMsg> {
    Ok(HandleMsg::Manager(
            SubHandleMsg::Claim { 
                asset 
            }).to_cosmos_msg(
                manager.code_hash,
                manager.address,
                None
            )?
    )
}

pub fn unbond_msg(
    asset: HumanAddr,
    amount: Uint128,
    manager: Contract,
) -> StdResult<CosmosMsg> {
    Ok(HandleMsg::Manager(
            SubHandleMsg::Unbond{ 
                asset,
                amount
            }).to_cosmos_msg(
                manager.code_hash,
                manager.address,
                None
            )?
    )
}
