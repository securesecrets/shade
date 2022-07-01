use cosmwasm_math_compat::Uint128;
use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct RequiredTask {
    pub address: HumanAddr,
    pub percent: Uint128,
}
