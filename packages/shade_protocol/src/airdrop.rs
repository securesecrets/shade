use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use secret_toolkit::utils::{InitCallback, HandleCallback, Query};
use cosmwasm_std::{HumanAddr, Uint128};
use crate::asset::Contract;
use crate::generic_response::ResponseStatus;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RequiredTask {
    pub address: HumanAddr,
    pub percent: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Reward {
    pub address: HumanAddr,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: HumanAddr,
    // The snip20 to be minted
    pub airdrop_snip20: Contract,
    // Required tasks
    pub task_claim: Vec<RequiredTask>,
    // Checks if airdrop has started / ended
    pub start_date: u64,
    pub end_date: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub airdrop_token: Contract,
    // The airdrop time limit
    pub start_time: Option<u64>,
    // Can be set to never end
    pub end_time: Option<u64>,
    // Secret network delegators snapshot
    pub rewards: Vec<Reward>,
    // Default gifted amount
    pub default_claim: Uint128,
    // The task related claims
    pub task_claim: Vec<RequiredTask>
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        admin: Option<HumanAddr>,
        start_date: Option<u64>,
        end_date: Option<u64>,
    },
    AddTasks {
        tasks: Vec<RequiredTask>
    },
    CompleteTask {
        address: HumanAddr
    },
    Claim {}
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Init { status: ResponseStatus },
    UpdateConfig { status: ResponseStatus },
    AddTask { status: ResponseStatus },
    CompleteTask { status: ResponseStatus },
    Claim { status: ResponseStatus }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig { },
    GetDates { },
    GetEligibility { address: HumanAddr }
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    // TODO: add total claimed in config
    Config { config: Config },
    Dates { start: u64, end: Option<u64> },
    Eligibility {
        // Total eligible
        total: Uint128,
        // Total claimed
        claimed: Uint128,
        // Total unclaimed but available
        unclaimed: Uint128,
        finished_tasks: Vec<RequiredTask>
    }
}