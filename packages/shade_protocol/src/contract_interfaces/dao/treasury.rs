use crate::utils::{
    asset::{Contract, RawContract},
    cycle::Cycle,
    generic_response::ResponseStatus,
};

use crate::{
    c_std::{Addr, Binary, StdResult, Uint128},
    contract_interfaces::dao::adapter,
};

use crate::utils::{ExecuteCallback, InstantiateCallback, Query};
use cosmwasm_schema::cw_serde;

/// The permission referenced in the Admin Auth contract to give a user
/// admin permissions for the Shade Treasury
//pub const SHADE_TREASURY_ADMIN: &str = "SHADE_TREASURY_ADMIN";

#[cw_serde]
pub struct Config {
    pub admin_auth: Contract,
    pub multisig: Addr,
}

#[cw_serde]
pub enum RunLevel {
    Normal,
    Deactivated,
    Migrating,
}

#[cw_serde]
pub enum Context {
    Receive,
    Rebalance,
    Migration,
}

#[cw_serde]
pub enum Action {
    IncreaseAllowance,
    DecreaseAllowance,
    ManagerUnbond,
    ManagerClaim,
    FundsReceived,
    SendFunds,
}

#[cw_serde]
pub struct Metric {
    pub action: Action,
    pub context: Context,
    pub timestamp: u64,
    pub token: Addr,
    pub amount: Uint128,
    pub user: Addr,
}

#[cw_serde]
pub enum AllowanceType {
    Amount,
    Portion,
}

#[cw_serde]
pub struct Allowance {
    pub spender: Addr,
    pub allowance_type: AllowanceType,
    pub cycle: Cycle,
    pub amount: Uint128,
    pub tolerance: Uint128,
}

#[cw_serde]
pub struct AllowanceMeta {
    pub spender: Addr,
    pub allowance_type: AllowanceType,
    pub cycle: Cycle,
    pub amount: Uint128,
    pub tolerance: Uint128,
    pub last_refresh: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin_auth: RawContract,
    pub multisig: String,
    pub viewing_key: String,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive {
        sender: String,
        from: String,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
    UpdateConfig {
        config: Config,
    },
    RegisterAsset {
        contract: RawContract,
    },
    RegisterManager {
        contract: RawContract,
    },
    // Setup a new allowance
    Allowance {
        asset: String,
        allowance: Allowance,
    },
    Update {
        asset: String,
    },
    SetRunLevel {
        run_level: RunLevel,
    },

    //TODO remove, change to treasury only interface
    Adapter(adapter::SubExecuteMsg),
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteAnswer {
    Init {
        status: ResponseStatus,
        address: String,
    },
    UpdateConfig {
        status: ResponseStatus,
    },
    Receive {
        status: ResponseStatus,
    },
    RegisterAsset {
        status: ResponseStatus,
    },
    Allowance {
        status: ResponseStatus,
    },
    Rebalance {
        status: ResponseStatus,
    },
    Migration {
        status: ResponseStatus,
    },
    Unbond {
        status: ResponseStatus,
    },
    RunLevel {
        run_level: RunLevel,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Assets {},
    // List of recurring allowances configured
    Allowances { asset: String },
    // Current allowance to spender
    Allowance { asset: String, spender: String },
    RunLevel,
    Metrics { date: String },
    /*
    Balance { asset: String },
    Reserves { asset: String },
    */
    Adapter(adapter::SubQueryMsg),
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Config { config: Config },
    Assets { assets: Vec<Addr> },
    Allowances { allowances: Vec<AllowanceMeta> },
    Allowance { amount: Uint128 },
    RunLevel { run_level: RunLevel },
    Metrics { metrics: Vec<Metric> },
}
