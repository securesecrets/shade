mod helpers;
pub mod msg;
pub use msg::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
pub mod core;
// Forward important libs to avoid constantly importing them in the cargo crates, could help reduce compile times
pub mod c_std {
    pub use cosmwasm_std::*;
}
pub use shade_protocol::{admin, airdrop, query_auth, snip20, utils};
pub const BLOCK_SIZE: usize = 256;
pub use helpers::*;
pub use serde;

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct Pagination {
    pub start: u64,
    pub limit: u8,
}
