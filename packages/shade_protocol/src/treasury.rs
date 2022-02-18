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
#[serde(rename_all = "snake_case")]
pub enum Allocation {
    // To remain liquid at all times
    Reserves {
        allocation: Uint128,
    },
    // Won't be counted in rebalancing
    Rewards {
        contract: Contract,
        allocation: Uint128,
    },
    // Monthly refresh, not counted in rebalance
    Allowance {
        address: HumanAddr,
        // Unlike others, this is a direct number of uTKN to allow monthly
        amount: Uint128,
    },
    // SCRT/ATOM/OSMO staking
    Staking {
        contract: Contract,
        allocation: Uint128,
    },
    // SKY / Derivative Staking
    Application {
        contract: Contract,
        allocation: Uint128,
        token: HumanAddr,
    },
    // Liquidity Providing
    Pool {
        contract: Contract,
        allocation: Uint128,
        secondary_asset: HumanAddr,
        token: HumanAddr,
    },
}

// Flag to be sent with funds
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Flag {
    pub flag: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllowanceData {
    pub spender: HumanAddr,
    pub amount: Uint128,
}

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
    OneTimeAllowance {
        asset: HumanAddr,
        spender: HumanAddr,
        amount: Uint128,
        expiration: Option<u64>,
    },
    UpdateConfig {
        config: Config,
    },
    RegisterAsset {
        contract: Contract,
        reserves: Option<Uint128>,
    },
    /* List of contracts/users given an allowance based on a percentage of the asset balance
     * e.g. governance, LP, SKY
     */
    RegisterAllocation {
        asset: HumanAddr,
        allocation: Allocation,
    },
    RefreshAllowance {},
    // Trigger to re-allocate asset (all if none)
    //Rebalance { asset: Option<HumanAddr> },
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
    UpdateConfig {
        status: ResponseStatus,
    },
    Receive {
        status: ResponseStatus,
    },
    RegisterAsset {
        status: ResponseStatus,
    },
    RegisterApp {
        status: ResponseStatus,
    },
    RefreshAllowance {
        status: ResponseStatus,
    },
    OneTimeAllowance {
        status: ResponseStatus,
    },
    //Rebalance { status: ResponseStatus },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Assets {},
    Balance {
        asset: HumanAddr,
    },
    Allocations {
        asset: HumanAddr,
    },
    Allowances {
        asset: HumanAddr,
        spender: HumanAddr,
    },
    LastAllowanceRefresh {},
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
    Allowances { allowances: Vec<AllowanceData> },
    LastAllowanceRefresh { datetime: String },
}
