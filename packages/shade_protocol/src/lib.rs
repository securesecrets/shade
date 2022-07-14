// TODO: make private later
pub mod contract_interfaces;
pub use contract_interfaces::*;

pub mod utils;

// Forward important libs to avoid constantly importing them in the cargo crates, could help reduce compile times
pub mod c_std {
    pub use cosmwasm_std::*;
}

pub mod storage {
    pub use cosmwasm_storage::*;
}

pub use serde;
pub use thiserror;
pub use cosmwasm_schema;
#[cfg(feature = "storage_plus")]
pub use secret_storage_plus;

#[cfg(feature = "query_auth_lib")]
pub use query_authentication;

#[cfg(feature = "ensemble")]
pub use fadroma;

#[cfg(feature = "multi-test")]
pub use secret_multi_test as multi_test;

// Expose contract in root since its so used
#[cfg(feature = "utils")]
pub use utils::asset::Contract;