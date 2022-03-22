use crate::utils::{asset::Contract, generic_response::ResponseStatus};
use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: HumanAddr,
    //pub account_holders: Vec<HumanAddr>,
    pub sscrt: Contract,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RefreshTracker {
    pub amount: Uint128,
    pub limit: Uint128,
    // RFC3339 datetime
    pub last_refresh: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Cycle {
    Once,
    Constant,
    Daily {
        days: Uint128,
    },
    Monthly {
        months: Uint128,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Allocation {
    // To remain liquid at all times
    Reserves {
        portion: Uint128,
    },
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
        // Unlike others, this is a direct number of uTKN to allow monthly
        cycle: Cycle,
        portion: Uint128,
        last_refresh: String,
    },
}

// Flag to be sent with funds
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Flag {
    pub flag: String,
}

/*
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllocationData {
    pub spender: HumanAddr,
    pub amount: Uint128,
}
*/

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub viewing_key: String,
    pub sscrt: Contract,
    //pub account_holders: Option<Vec<HumanAddr>>,
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
    RegisterAllocation {
        asset: HumanAddr,
        allowance: Allocation,
    },
    Rebalance {
        asset: Option<HumanAddr>,
    },
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
    RegisterAllocation { status: ResponseStatus },
    Rebalance { status: ResponseStatus },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Assets {},
    Balance { asset: HumanAddr },
    Allocations { asset: HumanAddr },
    Allowance {
        asset: HumanAddr,
        spender: HumanAddr,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: Config },
    Assets { assets: Vec<HumanAddr> },
    Allocations { allocations: Vec<Allocation> },
    Balance { amount: Uint128 },
    Allowance { allowance: Uint128 },
}
