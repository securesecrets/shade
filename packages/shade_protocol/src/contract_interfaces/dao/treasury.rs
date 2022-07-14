use crate::utils::{asset::Contract, cycle::Cycle, generic_response::ResponseStatus};

use crate::contract_interfaces::dao::adapter;
use crate::c_std::{Binary, Addr, StdResult, Uint128};

use crate::utils::{HandleCallback, InitCallback, Query};
use cosmwasm_schema::{cw_serde};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
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

/*
#[cw_serde]
pub struct Balance {
    pub token: Addr,
    pub amount: Uint128,
}

#[cw_serde]
pub enum Status {
    Active,
    Disabled,
    Closed,
    Transferred,
}

//TODO: move accounts to treasury manager
#[cw_serde]
pub struct Account {
    pub balances: Vec<Balance>,
    pub unbondings: Vec<Balance>,
    pub claimable: Vec<Balance>,
    pub status: Status,
}
*/

// Flag to be sent with funds
#[cw_serde]
pub struct Flag {
    pub flag: String,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<Addr>,
    pub viewing_key: String,
}

impl InitCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive {
        sender: Addr,
        from: Addr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
    UpdateConfig {
        config: Config,
    },
    RegisterAsset {
        contract: Contract,
        reserves: Option<Uint128>,
    },
    RegisterManager {
        contract: Contract,
    },
    // Setup a new allowance
    Allowance {
        asset: Addr,
        allowance: Allowance,
    },
    /* TODO: Maybe?
    TransferAccount {
    },
    */
    Adapter(adapter::SubHandleMsg),
}

impl HandleCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum HandleAnswer {
    Init {
        status: ResponseStatus,
        address: Addr,
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
        asset: Addr,
    },
    // List of actual current amounts
    Allowance {
        asset: Addr,
        spender: Addr,
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
    Allowance { allowance: Uint128 },
    //Accounts { accounts: Vec<Addr> },
    //Account { account: Account },
}
