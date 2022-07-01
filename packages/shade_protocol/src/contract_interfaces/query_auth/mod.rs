#[cfg(feature = "query_auth_impl")]
pub mod auth;

use cosmwasm_std::{Binary, HumanAddr};
use schemars::JsonSchema;
use query_authentication::permit::Permit;
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use serde::{Deserialize, Serialize};
use crate::utils::generic_response::ResponseStatus;
#[cfg(feature = "query_auth_impl")]
use crate::utils::storage::plus::ItemStorage;
#[cfg(feature = "query_auth_impl")]
use secret_storage_plus::Item;
use secret_toolkit::crypto::sha_256;
use crate::utils::asset::Contract;

#[cfg(feature = "query_auth_impl")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Admin(pub Contract);

#[cfg(feature = "query_auth_impl")]
impl ItemStorage for Admin {
    const ITEM: Item<'static, Self> = Item::new("admin-");
}

#[cfg(feature = "query_auth_impl")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub admin_auth: Contract,
    pub prng_seed: Binary
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatus {
    Default,
    DisablePermit,
    DisableVK,
    DisableAll
}

#[cfg(feature = "query_auth_impl")]
impl ItemStorage for ContractStatus {
    const ITEM: Item<'static, Self> = Item::new("contract-status-");
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
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
    }
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    SetAdminAuth {
        status: ResponseStatus
    },
    SetRunState {
        status: ResponseStatus
    },
    SetViewingKey {
        status: ResponseStatus
    },
    CreateViewingKey {
        key: String
    },
    BlockPermitKey {
        status: ResponseStatus
    },
}

pub type QueryPermit = Permit<PermitData>;

#[remain::sorted]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PermitData {
    pub data: Binary,
    pub key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},

    ValidateViewingKey {
        user: HumanAddr,
        key: String,
    },
    ValidatePermit {
        permit: QueryPermit
    }
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config {
        admin: Contract,
        state: ContractStatus
    },
    ValidateViewingKey {
        is_valid: bool
    },
    ValidatePermit {
        user: HumanAddr,
        is_revoked: bool
    }
}


