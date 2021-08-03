use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, JsonSchema)]
pub struct ReferenceData {
    pub rate: Uint128,
    pub last_updated_base: u64,
    pub last_updated_quote: u64,
}
