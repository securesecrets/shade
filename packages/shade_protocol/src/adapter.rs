use crate::utils::{asset::Contract, generic_response::ResponseStatus};
use cosmwasm_std::{Binary, Decimal, Delegation, HumanAddr, Uint128, Validator, StdResult, StdError, Extern, Api, Querier, Storage, CosmosMsg};
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BondStatus {
    Active,
    Unbonding,
    UnbondComplete,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Bond {
    pub amount: Uint128,
    pub token: Contract,
    pub address: HumanAddr,
    pub status: BondStatus,
}

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
    Adapter(SubHandleMsg),
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
    Balance { asset: HumanAddr },
    Unbonding { asset: HumanAddr },
    Claimable { asset: HumanAddr },
    //CanUnbond { asset: HumanAddr },
    /* TODO
     * - LP pool assets
     * Ratio { asset0: HumanAddr, asset1: HumanAddr },
     * - things like unbond period
     * Metadata { asset: HumanAddr },
     * CanUnbond { asset: HumanAddr },
     */
}

/*
impl Query for SubQueryMsg {
    const BLOCK_SIZE: usize = 256;
}
*/

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Adapter(SubQueryMsg),
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
    //CanUnbond { unbondable: bool },
}

pub fn claimable_query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: &HumanAddr,
    adapter: Contract,
) -> StdResult<Uint128> {

    match (QueryMsg::Adapter(SubQueryMsg::Claimable {
        asset: asset.clone(),
    }).query(&deps.querier, adapter.code_hash, adapter.address.clone())?) {
        QueryAnswer::Claimable { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            format!("Failed to query adapter claimable from {}", adapter.address)
        ))
    }
}

pub fn unbonding_query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: &HumanAddr,
    adapter: Contract,
) -> StdResult<Uint128> {

    match (QueryMsg::Adapter(SubQueryMsg::Unbonding {
        asset: asset.clone(),
    }).query(&deps.querier, adapter.code_hash, adapter.address.clone())?) {
        QueryAnswer::Unbonding { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            format!("Failed to query adapter unbonding from {}", adapter.address)
        ))
    }
}

pub fn balance_query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: &HumanAddr,
    adapter: Contract,
) -> StdResult<Uint128> {

    match (QueryMsg::Adapter(
            SubQueryMsg::Balance {
                asset: asset.clone(),
            }
        ).query(&deps.querier, adapter.code_hash, adapter.address.clone())?) {
        QueryAnswer::Balance { amount } => Ok(amount),
        _ => Err(StdError::generic_err(
            format!("Failed to query adapter balance from {}", adapter.address)
        ))
    }
}

pub fn claim_msg(
    asset: HumanAddr,
    adapter: Contract,
) -> StdResult<CosmosMsg> {
    Ok(HandleMsg::Adapter(
            SubHandleMsg::Claim { 
                asset 
            }).to_cosmos_msg(
                adapter.code_hash,
                adapter.address,
                None
            )?
    )
}

pub fn unbond_msg(
    asset: HumanAddr,
    amount: Uint128,
    adapter: Contract,
) -> StdResult<CosmosMsg> {
    Ok(HandleMsg::Adapter(
            SubHandleMsg::Unbond{ 
                asset,
                amount
            }).to_cosmos_msg(
                adapter.code_hash,
                adapter.address,
                None
            )?
    )
}

pub fn update_msg(
    asset: HumanAddr,
    adapter: Contract,
) -> StdResult<CosmosMsg> {
    Ok(HandleMsg::Adapter(
            SubHandleMsg::Update {
                asset
            }).to_cosmos_msg(
                adapter.code_hash,
                adapter.address,
                None
            )?
    )
}
