use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128, Binary};
use secret_toolkit::utils::{InitCallback, HandleCallback, Query};
use secretcli::secretcli::{TestInit, TestHandle, TestQuery};
use crate::{
    asset::Contract,
    generic_response::ResponseStatus,
};

// This is used when calling itself
pub const GOVERNANCE_SELF: &str = "SELF";

// Admin command variable spot
pub const ADMIN_COMMAND_VARIABLE: &str = "{}";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: HumanAddr,
    // The amount of time given for each proposal
    pub proposal_deadline: u64,
    // The minimum total amount of votes needed to approve deadline
    pub minimum_votes: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AdminCommand {
    pub msg: String,
    pub total_arguments: u16,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Proposal {
    pub id: Uint128,
    pub target: String,
    pub msg: Binary,
    pub description: String,
    pub due_date: u64,
    // Used to determine if community voted for it
    pub is_admin_command: bool,
    pub vote_status: ProposalStatus,
    // This will be available after proposal is run
    pub run_status: Option<ResponseStatus>
}
//TODO: move vote status to its own store
//TODO: move run status to its own store

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    AdminRequested,
    InProgress,
    Expired,
    Rejected,
    Accepted,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Vote {
    Yes,
    No,
    Abstain,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub proposal_deadline: u64,
    pub quorum: Uint128,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

impl TestInit for InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    /// Generic proposal
    CreateProposal {
        // Contract that will be run
        target_contract: String,
        // This will be saved as binary
        proposal: String,
        description: String,
    },

    /// Admin Command
    /// These commands can be run by admins any time
    AddAdminCommand {
        name: String,
        proposal: String,
    },
    RemoveAdminCommand {
        name: String,
    },
    UpdateAdminCommand {
        name: String,
        proposal: String,
    },
    TriggerAdminCommand {
        target: String,
        command: String,
        variables: Vec<String>,
        description: String,
    },

    /// Config changes
    UpdateConfig {
        admin: Option<HumanAddr>,
        proposal_deadline: Option<u64>,
        minimum_votes: Option<Uint128>,
    },

    // RequestMigration {}

    /// Add a contract to send proposal msgs to
    AddSupportedContract {
        name: String,
        contract: Contract,
    },
    RemoveSupportedContract {
        name: String,
    },
    UpdateSupportedContract {
        name: String,
        contract: Contract,
    },



    /// Proposal voting
    MakeVote {
        proposal_id: Uint128,
        option: Vote,
    },

    /// Trigger proposal
    TriggerProposal {
        proposal_id: Uint128,
    }
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

impl TestHandle for HandleMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    CreateProposal { status: ResponseStatus, proposal_id: Uint128 },
    Vote { status: ResponseStatus },
    TriggerProposal { status: ResponseStatus },
    TriggerAdminCommand { status: ResponseStatus, proposal_id: Uint128 },
    UpdateConfig { status: ResponseStatus },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetProposals { total: Uint128, start: Uint128 },
    GetProposal { proposal_id: Uint128 },
    GetTotalProposals {},
    GetSupportedContracts {},
    GetSupportedContract { name: String },
    GetAdminCommands {},
    GetAdminCommand { name: String },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

impl TestQuery<QueryAnswer> for QueryMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Proposals { proposals: Vec<Proposal> },
    Proposal { proposal: Proposal },
    TotalProposals { total: Uint128 },
    SupportedContracts { contracts: Vec<String> },
    SupportedContract { contract: Contract },
    AdminCommands { commands: Vec<String> },
    AdminCommand { command: AdminCommand },
}