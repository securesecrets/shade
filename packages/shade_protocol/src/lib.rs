// TODO: make private later
pub mod contract_interfaces;
pub use contract_interfaces::*;

pub const BLOCK_SIZE: usize = 256;
pub mod utils;

// Forward important libs to avoid constantly importing them in the cargo crates, could help reduce compile times
pub mod c_std {
    pub use contract_derive::shd_entry_point;
    pub use cosmwasm_std::*;
}

pub mod storage {
    pub use cosmwasm_storage::*;
}

pub use cosmwasm_schema;
pub use schemars;
#[cfg(feature = "storage_plus")]
pub use secret_storage_plus;
pub use serde;
pub use thiserror;

#[cfg(feature = "query_auth_lib")]
pub use query_authentication;

#[cfg(feature = "ensemble")]
pub use fadroma;

#[cfg(not(target_arch = "wasm32"))]
#[cfg(feature = "multi-test")]
pub use secret_multi_test as multi_test;

#[cfg(feature = "multi-test")]
pub use anyhow::Result as AnyResult;

// Expose contract in root since its so used
#[cfg(feature = "utils")]
pub use utils::asset::Contract;

#[cfg(feature = "chrono")]
pub use chrono;
