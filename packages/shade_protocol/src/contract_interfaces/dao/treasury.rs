use crate::utils::asset::RawContract;
use crate::utils::{asset::Contract, cycle::Cycle, generic_response::ResponseStatus};

use crate::contract_interfaces::dao::adapter;
use crate::c_std::{Binary, Addr, StdResult, Uint128};

use crate::utils::{ExecuteCallback, InstantiateCallback, Query};
use cosmwasm_schema::{cw_serde};

/// The permission referenced in the Admin Auth contract to give a user
/// admin permissions for the Shade Treasury
pub const SHADE_TREASURY_ADMIN: &str = "SHADE_TREASURY_ADMIN";

#[cw_serde]
pub struct Config {
    pub admin_auth: Contract,
}

/* Examples:
 * Constant-Portion -> Finance manager
 * Constant-Amount -> Rewards, pre-set manually adjusted
 * Monthly-Portion -> Rewards, self-scaling
 * Monthly-Amount -> Governance grant or Committee funding
 *
 * Once-Portion -> Disallowed
 */
#[cw_serde]
pub enum Allowance {
    // Monthly refresh, not counted in rebalance
    Amount {
        //nick: Option<String>,
        spender: Addr,
        // Unlike others, this is a direct number of uTKN to allow monthly
        cycle: Cycle,
        amount: Uint128,
        last_refresh: String,
    },
    Portion {
        //nick: Option<String>,
        spender: Addr,
        portion: Uint128,
        //TODO: This needs to be omitted from the handle msg
        last_refresh: String,
        tolerance: Uint128,
    },
}

//TODO rename to Adapter
#[cw_serde]
pub struct Manager {
    pub contract: Contract,
    pub balance: Uint128,
    pub desired: Uint128,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin_auth: RawContract,
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
    /* TODO: Maybe?
    TransferAccount {
    },
    */
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
    Unbond {
        status: ResponseStatus,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Assets {},
    // List of recurring allowances configured
    Allowances {
        asset: String,
    },
    // List of actual current amounts
    Allowance {
        asset: String,
        spender: String,
    },
    /*
    AccountHolders { },
    Account { 
        holder: Addr,
    },
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
    Allowances { allowances: Vec<Allowance> },
    CurrentAllowances { allowances: Vec<Allowance> },
    Allowance { amount: Uint128 },
}
