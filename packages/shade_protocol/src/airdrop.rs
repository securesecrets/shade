use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use secret_toolkit::utils::{InitCallback, HandleCallback, Query};
use cosmwasm_std::{HumanAddr, Uint128};
use crate::asset::Contract;
use crate::generic_response::ResponseStatus;
use crate::signature::Permit;

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

type AddressProofPermit = Permit<AddressProofMsg>;

#[remain::sorted]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AddressProofMsg {
    pub address: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: HumanAddr,
    // Where the decayed tokens will be dumped, if none then nothing happens
    pub dump_address: Option<HumanAddr>,
    // The snip20 to be minted
    pub airdrop_snip20: Contract,
    // Total claimable amount
    pub airdrop_total: Uint128,
    // Required tasks
    pub task_claim: Vec<RequiredTask>,
    // Checks if airdrop has started / ended
    pub start_date: u64,
    pub end_date: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    // Where the decayed tokens will be dumped, if none then nothing happens
    pub dump_address: Option<HumanAddr>,
    pub airdrop_token: Contract,
    // The airdrop time limit
    pub start_time: Option<u64>,
    // Can be set to never end
    pub end_time: Option<u64>,
    // Delegators snapshot
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
        dump_address: Option<HumanAddr>,
        start_date: Option<u64>,
        end_date: Option<u64>,
    },
    AddTasks {
        tasks: Vec<RequiredTask>
    },
    CompleteTask {
        address: HumanAddr
    },
    Claim {},
    Decay {},
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
    Claim { status: ResponseStatus },
    Decay { status: ResponseStatus },
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
    Config { config: Config, total_claimed: Uint128 },
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