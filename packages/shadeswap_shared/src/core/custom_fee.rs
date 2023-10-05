use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Clone, Copy, Debug)]
pub struct Fee {
    pub nom: u64,
    pub denom: u64,
}

impl Fee {
    pub fn new(nom: u64, denom: u64) -> Self {
        Self { nom, denom }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CustomFee {
    pub shade_dao_fee: Fee,
    pub lp_fee: Fee,
}
