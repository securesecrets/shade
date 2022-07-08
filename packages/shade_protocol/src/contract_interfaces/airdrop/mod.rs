pub mod account;
pub mod claim_info;
pub mod errors;

use crate::{
    contract_interfaces::airdrop::{
        account::{AccountPermit, AddressProofPermit},
        claim_info::RequiredTask,
    },
    utils::{asset::Contract, generic_response::ResponseStatus},
};
use crate::math_compat::Uint128;
use crate::c_std::{Binary, HumanAddr};
use crate::schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use crate::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: HumanAddr,
    // Used for permit validation when querying
    pub contract: HumanAddr,
    // Where the decayed tokens will be dumped, if none then nothing happens
    pub dump_address: Option<HumanAddr>,
    // The snip20 to be minted
    pub airdrop_snip20: Contract,
    // Airdrop amount
    pub airdrop_amount: Uint128,
    // Required tasks
    pub task_claim: Vec<RequiredTask>,
    // Checks if airdrop has started / ended
    pub start_date: u64,
    // Airdrop stops at end date if there is one
    pub end_date: Option<u64>,
    // Starts to decay at this date
    pub decay_start: Option<u64>,
    // This is necessary to validate the airdrop information
    // tree root
    pub merkle_root: Binary,
    // tree height
    pub total_accounts: u32,
    // max possible reward amount; used to prevent collision possibility
    pub max_amount: Uint128,
    // Protects from leaking user information by limiting amount detail
    pub query_rounding: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    // Where the decayed tokens will be dumped, if none then nothing happens
    pub dump_address: Option<HumanAddr>,
    pub airdrop_token: Contract,
    // Airdrop amount
    pub airdrop_amount: Uint128,
    // The airdrop time limit
    pub start_date: Option<u64>,
    // Can be set to never end
    pub end_date: Option<u64>,
    // Starts to decay at this date
    pub decay_start: Option<u64>,
    // Base64 encoded version of the tree root
    pub merkle_root: Binary,
    // Root height
    pub total_accounts: u32,
    // Max possible reward amount
    pub max_amount: Uint128,
    // Default gifted amount
    pub default_claim: Uint128,
    // The task related claims
    pub task_claim: Vec<RequiredTask>,
    // Protects from leaking user information by limiting amount detail
    pub query_rounding: Uint128,
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
        query_rounding: Option<Uint128>,
        start_date: Option<u64>,
        end_date: Option<u64>,
        decay_start: Option<u64>,
        padding: Option<String>,
    },
    AddTasks {
        tasks: Vec<RequiredTask>,
        padding: Option<String>,
    },
    CompleteTask {
        address: HumanAddr,
        padding: Option<String>,
    },
    Account {
        addresses: Vec<AddressProofPermit>,
        partial_tree: Vec<Binary>,
        padding: Option<String>,
    },
    DisablePermitKey {
        key: String,
        padding: Option<String>,
    },
    SetViewingKey {
        key: String,
        padding: Option<String>,
    },
    Claim {
        padding: Option<String>,
    },
    ClaimDecay {
        padding: Option<String>,
    },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    UpdateConfig {
        status: ResponseStatus,
    },
    AddTask {
        status: ResponseStatus,
    },
    CompleteTask {
        status: ResponseStatus,
    },
    Account {
        status: ResponseStatus,
        // Total eligible
        total: Uint128,
        // Total claimed
        claimed: Uint128,
        finished_tasks: Vec<RequiredTask>,
        // Addresses claimed
        addresses: Vec<HumanAddr>,
    },
    DisablePermitKey {
        status: ResponseStatus,
    },
    SetViewingKey {
        status: ResponseStatus,
    },
    Claim {
        status: ResponseStatus,
        // Total eligible
        total: Uint128,
        // Total claimed
        claimed: Uint128,
        finished_tasks: Vec<RequiredTask>,
        // Addresses claimed
        addresses: Vec<HumanAddr>,
    },
    ClaimDecay {
        status: ResponseStatus,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Dates {
        current_date: Option<u64>,
    },
    TotalClaimed {},
    Account {
        permit: AccountPermit,
        current_date: Option<u64>,
    },
    AccountWithKey {
        account: HumanAddr,
        key: String,
        current_date: Option<u64>,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config {
        config: Config,
    },
    Dates {
        start: u64,
        end: Option<u64>,
        decay_start: Option<u64>,
        decay_factor: Option<Uint128>,
    },
    TotalClaimed {
        claimed: Uint128,
    },
    Account {
        // Total eligible
        total: Uint128,
        // Total claimed
        claimed: Uint128,
        // Total unclaimed but available
        unclaimed: Uint128,
        finished_tasks: Vec<RequiredTask>,
        // Addresses claimed
        addresses: Vec<HumanAddr>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AccountVerification {
    pub account: HumanAddr,
    pub claimed: bool,
}