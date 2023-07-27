#[cfg(feature = "query_auth_impl")]
pub mod auth;
pub mod helpers;

use crate::c_std::{Addr, Binary};

#[cfg(feature = "query_auth_impl")]
use crate::utils::storage::plus::ItemStorage;
use crate::{
    query_authentication::permit::Permit,
    utils::{
        asset::Contract,
        crypto::sha_256,
        generic_response::ResponseStatus,
        ExecuteCallback,
        InstantiateCallback,
        Query,
    },
};
use cosmwasm_schema::cw_serde;
#[cfg(feature = "query_auth_impl")]
use secret_storage_plus::Item;

#[cfg(feature = "query_auth_impl")]
#[cw_serde]
pub struct Admin(pub Contract);

#[cfg(feature = "query_auth_impl")]
impl ItemStorage for Admin {
    const ITEM: Item<'static, Self> = Item::new("admin-");
}

#[cfg(feature = "query_auth_impl")]
#[cw_serde]
pub struct RngSeed(pub Vec<u8>);

#[cfg(feature = "query_auth_impl")]
impl ItemStorage for RngSeed {
    const ITEM: Item<'static, Self> = Item::new("rng-seed-");
}

#[cfg(feature = "query_auth_impl")]
impl RngSeed {
    pub fn new(seed: Binary) -> Self {
        Self(sha_256(&seed.0).to_vec())
    }
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin_auth: Contract,
    pub prng_seed: Binary,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ContractStatus {
    Default,
    DisablePermit,
    DisableVK,
    DisableAll,
}

#[cfg(feature = "query_auth_impl")]
impl ItemStorage for ContractStatus {
    const ITEM: Item<'static, Self> = Item::new("contract-status-");
}

#[cw_serde]
pub enum ExecuteMsg {
    SetAdminAuth {
        admin: Contract,
        padding: Option<String>,
    },
    SetRunState {
        state: ContractStatus,
        padding: Option<String>,
    },

    SetViewingKey {
        key: String,
        padding: Option<String>,
    },
    CreateViewingKey {
        entropy: String,
        padding: Option<String>,
    },

    BlockPermitKey {
        key: String,
        padding: Option<String>,
    },
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteAnswer {
    SetAdminAuth { status: ResponseStatus },
    SetRunState { status: ResponseStatus },
    SetViewingKey { status: ResponseStatus },
    CreateViewingKey { key: String },
    BlockPermitKey { status: ResponseStatus },
}

pub type QueryPermit = Permit<PermitData>;

#[remain::sorted]
#[cw_serde]
pub struct PermitData {
    pub data: Binary,
    pub key: String,
}

#[cw_serde]
pub enum QueryMsg {
    Config {},

    ValidateViewingKey { user: Addr, key: String },
    ValidatePermit { permit: QueryPermit },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Config {
        admin: Contract,
        state: ContractStatus,
    },
    ValidateViewingKey {
        is_valid: bool,
    },
    ValidatePermit {
        user: Addr,
        is_revoked: bool,
    },
}
