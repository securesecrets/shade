use crate::{
    adapter,
    utils::{
        asset::Contract,
        generic_response::ResponseStatus,
        cycle::Cycle,
    },
};

use cosmwasm_std::{Binary, HumanAddr, Uint128, StdResult};
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub admin: HumanAddr,
    pub sscrt: Contract,
}

/* Examples:
 * Constant-Portion -> Finance manager
 * Constant-Amount -> Rewards, pre-set manually adjusted
 * Monthly-Portion -> Rewards, self-scaling
 * Monthly-Amount -> Governance grant or Committee funding
 *
 * Once-Portion -> Disallowed
 */
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Allowance {
    // Monthly refresh, not counted in rebalance
    Amount {
        //nick: Option<String>,
        spender: HumanAddr,
        // Unlike others, this is a direct number of uTKN to allow monthly
        cycle: Cycle,
        amount: Uint128,
        last_refresh: String,
    },
    Portion {
        //nick: Option<String>,
        spender: HumanAddr,
        portion: Uint128,
        //TODO: This needs to be omitted from the handle msg
        last_refresh: String,
        tolerance: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Manager {
    pub contract: Contract,
    pub balance: Uint128,
    pub desired: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Balance {
    pub token: HumanAddr,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Active,
    Disabled,
    Closed,
    Transferred,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Account {
    pub balances: Vec<Balance>,
    pub unbondings: Vec<Balance>,
    pub claimable: Vec<Balance>,
    pub status: Status,
}

// Flag to be sent with funds
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Flag {
    pub flag: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub viewing_key: String,
    pub sscrt: Contract,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
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
        asset: HumanAddr,
        allowance: Allowance,
    },
    AddAccount {
        holder: HumanAddr,
    },
    CloseAccount {
        holder: HumanAddr,
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

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Init {
        status: ResponseStatus,
        address: HumanAddr,
    },
    UpdateConfig { status: ResponseStatus },
    Receive { status: ResponseStatus },
    RegisterAsset { status: ResponseStatus },
    Allowance { status: ResponseStatus },
    AddAccount { status: ResponseStatus },
    RemoveAccount { status: ResponseStatus },
    Rebalance { status: ResponseStatus },
    Unbond { status: ResponseStatus },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Assets {},
    // List of recurring allowances configured
    Allowances { asset: HumanAddr },
    // List of actual current amounts
    Allowance {
        asset: HumanAddr,
        spender: HumanAddr,
    },
    Accounts { },
    Account { 
        holder: HumanAddr,
    },
    Adapter(adapter::SubQueryMsg),
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: Config },
    Assets { assets: Vec<HumanAddr> },
    Allowances { allowances: Vec<Allowance> },
    CurrentAllowances { allowances: Vec<Allowance> },
    Allowance { allowance: Uint128 },
    Accounts { accounts: Vec<HumanAddr> },
    Account { account: Account, },
}
