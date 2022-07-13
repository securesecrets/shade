use crate::utils::{asset::Contract, cycle::Cycle, generic_response::ResponseStatus};

use crate::contract_interfaces::dao::adapter;
use crate::c_std::{Binary, Addr, StdResult, Uint128};

use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use crate::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
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
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Manager {
    pub contract: Contract,
    pub balance: Uint128,
    pub desired: Uint128,
}

/*
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Balance {
    pub token: Addr,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Active,
    Disabled,
    Closed,
    Transferred,
}

//TODO: move accounts to treasury manager
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Account {
    pub balances: Vec<Balance>,
    pub unbondings: Vec<Balance>,
    pub claimable: Vec<Balance>,
    pub status: Status,
}
*/

// Flag to be sent with funds
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Flag {
    pub flag: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InitMsg {
    pub admin: Option<Addr>,
    pub viewing_key: String,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
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

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: Config },
    Assets { assets: Vec<Addr> },
    Allowances { allowances: Vec<Allowance> },
    CurrentAllowances { allowances: Vec<Allowance> },
    Allowance { allowance: Uint128 },
    //Accounts { accounts: Vec<Addr> },
    //Account { account: Account },
}
