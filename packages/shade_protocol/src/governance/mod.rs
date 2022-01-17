pub mod proposal;
pub mod vote;

use crate::{asset::Contract, generic_response::ResponseStatus};
use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use serde::{Deserialize, Serialize};

// This is used when calling itself
pub const GOVERNANCE_SELF: &str = "SELF";

// Admin command variable spot
pub const ADMIN_COMMAND_VARIABLE: &str = "{}";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: HumanAddr,
    // Staking contract - optional to support admin only
    pub staker: Option<Contract>,
    // The token allowed for funding
    pub funding_token: Contract,
    // The amount required to fund a proposal
    pub funding_amount: Uint128,
    // Proposal funding period deadline
    pub funding_deadline: u64,
    // Proposal voting period deadline
    pub voting_deadline: u64,
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
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub staker: Option<Contract>,
    pub funding_token: Contract,
    pub funding_amount: Uint128,
    pub funding_deadline: u64,
    pub voting_deadline: u64,
    pub quorum: Uint128,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

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

    /// Proposal funding
    Receive {
        sender: HumanAddr,
        amount: Uint128,
        // Proposal ID
        msg: Option<Binary>,
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
        staker: Option<Contract>,
        proposal_deadline: Option<u64>,
        funding_amount: Option<Uint128>,
        funding_deadline: Option<u64>,
        minimum_votes: Option<Uint128>,
    },

    DisableStaker {},

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

    /// Proposal voting - can only be done by staking contract
    MakeVote {
        voter: HumanAddr,
        proposal_id: Uint128,
        votes: vote::VoteTally,
    },

    /// Trigger proposal
    TriggerProposal {
        proposal_id: Uint128,
    },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    CreateProposal {
        status: ResponseStatus,
        proposal_id: Uint128,
    },
    FundProposal {
        status: ResponseStatus,
        total_funding: Uint128,
    },
    AddAdminCommand {
        status: ResponseStatus,
    },
    RemoveAdminCommand {
        status: ResponseStatus,
    },
    UpdateAdminCommand {
        status: ResponseStatus,
    },
    TriggerAdminCommand {
        status: ResponseStatus,
        proposal_id: Uint128,
    },
    UpdateConfig {
        status: ResponseStatus,
    },
    DisableStaker {
        status: ResponseStatus,
    },
    AddSupportedContract {
        status: ResponseStatus,
    },
    RemoveSupportedContract {
        status: ResponseStatus,
    },
    UpdateSupportedContract {
        status: ResponseStatus,
    },
    MakeVote {
        status: ResponseStatus,
    },
    TriggerProposal {
        status: ResponseStatus,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetProposalVotes {
        proposal_id: Uint128,
    },
    GetProposals {
        start: Uint128,
        end: Uint128,
        status: Option<proposal::ProposalStatus>,
    },
    GetProposal {
        proposal_id: Uint128,
    },
    GetTotalProposals {},
    GetSupportedContracts {},
    GetSupportedContract {
        name: String,
    },
    GetAdminCommands {},
    GetAdminCommand {
        name: String,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    ProposalVotes {
        status: vote::VoteTally,
    },
    Proposals {
        proposals: Vec<proposal::QueriedProposal>,
    },
    Proposal {
        proposal: proposal::QueriedProposal,
    },
    TotalProposals {
        total: Uint128,
    },
    SupportedContracts {
        contracts: Vec<String>,
    },
    SupportedContract {
        contract: Contract,
    },
    AdminCommands {
        commands: Vec<String>,
    },
    AdminCommand {
        command: AdminCommand,
    },
}
