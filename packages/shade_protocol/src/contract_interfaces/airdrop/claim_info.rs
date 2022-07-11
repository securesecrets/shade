use crate::math_compat::Uint128;
use crate::c_std::HumanAddr;
use crate::schemars::JsonSchema;
use crate::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct RequiredTask {
    pub address: HumanAddr,
    pub percent: Uint128,
}
