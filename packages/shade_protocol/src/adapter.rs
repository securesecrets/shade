use crate::utils::{asset::Contract, generic_response::ResponseStatus};
use cosmwasm_std::{Binary, Decimal, Delegation, HumanAddr, Uint128, Validator};
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
pub enum HandleMsg {
    // Begin unbonding amount
    Unbond { asset: HumanAddr, amount: Uint128 },
    Claim { asset: HumanAddr },
    // Maintenance trigger e.g. claim rewards and restake
    Update { },
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
        validator: Validator,
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
pub enum QueryMsg {
    Balance { asset: HumanAddr },
    Unbonding { asset: HumanAddr },
    Claimable { asset: HumanAddr },
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
