use crate::utils::{
    asset::Contract, 
    generic_response::ResponseStatus,
    unbonding::{UnbondStatus, Unbonding},
};
use cosmwasm_std::{Binary, Decimal, Delegation, HumanAddr, Uint128, Validator};
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    // unbond amount back to treasury
    Unbond {
        asset: HumanAddr,
        amount: Uint128, 
    },
    Rebalance { 
        asset: HumanAddr,
    },
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
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Balance { asset: HumanAddr },
    Unbondings {},
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Balance { amount: Uint128 },
    Unbondings { unbondings: Vec<Unbonding> },
}
