pub mod storage;

use crate::{utils::storage::plus::ItemStorage, Contract};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use secret_storage_plus::Item;
use storage::{Derivative, Profile, UserDerivativePool};

#[cw_serde]
pub struct Config {
    pub admin: Contract,
    pub auth: Contract,
    pub vote_token: Contract,
}
impl ItemStorage for Config {
    const ITEM: Item<'static, Self> = Item::new("config-");
}

#[cw_serde]
pub enum RunState {
    Active,
    Maintenance,
    Migrated { new: Contract },
}
impl ItemStorage for RunState {
    const ITEM: Item<'static, Self> = Item::new("runstate-");
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Contract,
    pub auth: Contract,

    // List of staking contracts to bootstrap
    // The code id and a list of profiles with name and unbonding times
    pub staking_profiles: Option<(u64, Vec<(String, u64)>)>,

    // When not migrating, it will bootstrap a list of staking profiles
    pub migration: Option<MigrationData>,

    pub vote_token: Contract,
    pub derivative: Option<Derivative>,
}
impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

pub struct MigrationData {
    // List of profiles to migrate
    profiles: Vec<(String, Profile)>,
}

#[cw_serde]
pub enum ExecuteMsg {
    // Admin facing
    UpdateConfig {},
    Profile {
        action: ProfileActions,
    },

    // User facing
    RequestUnbond {
        profile: String,
        amount: Uint128,
    },

    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint128,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>,
    },

    // Migration
    MigrateContract {
        id: u64,
    },
    MigrateUser {},
    ReceiveMigratedUser {
        user: Addr,
        data: Vec<(String, UserDerivativePool)>,
    },
}
impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ProfileActions {
    Remove { profile: String },
    Add { key: String, profile: Profile },
    Init { id: u64 },
    Update { key: String, profile: Profile },
}

#[cw_serde]
pub enum QueryMsg {}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}
