use cosmwasm_std::{HumanAddr, Uint128};
use crate::utils::asset::Contract;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum UnbondStatus {
    Active,
    Unbonding,
    UnbondComplete,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Unbonding {
    pub amount: Uint128,
    pub token: Contract,
    // Token unbonding to e.g. treasury
    pub address: HumanAddr,
    pub status: UnbondStatus,
}
