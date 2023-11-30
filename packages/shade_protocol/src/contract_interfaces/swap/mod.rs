use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod core;
pub mod amm_pair;
pub mod factory;
pub mod router;
pub mod staking;

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Pagination {
    pub start: u64,
    pub limit: u8,
}
