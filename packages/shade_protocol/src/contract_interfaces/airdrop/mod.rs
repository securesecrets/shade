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
use crate::c_std::{Uint128, Binary, Addr};

use crate::utils::{ExecuteCallback, InstantiateCallback, Query};
use cosmwasm_schema::{cw_serde};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    // Used for permit validation when querying
    pub contract: Addr,
    // Where the decayed tokens will be dumped, if none then nothing happens
    pub dump_address: Option<Addr>,
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

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<Addr>,
    // Where the decayed tokens will be dumped, if none then nothing happens
    pub dump_address: Option<Addr>,
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

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        admin: Option<Addr>,
        dump_address: Option<Addr>,
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
        address: Addr,
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

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
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
        addresses: Vec<Addr>,
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
        addresses: Vec<Addr>,
    },
    ClaimDecay {
        status: ResponseStatus,
    },
}

#[cw_serde]
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
        account: Addr,
        key: String,
        current_date: Option<u64>,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
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
        addresses: Vec<Addr>,
    },
}

#[cw_serde]
pub struct AccountVerification {
    pub account: Addr,
    pub claimed: bool,
}