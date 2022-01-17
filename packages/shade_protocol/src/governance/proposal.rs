use crate::generic_response::ResponseStatus;
use cosmwasm_std::{Binary, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Proposal {
    // Proposal ID
    pub id: Uint128,
    // Target smart contract
    pub target: String,
    // Message to execute
    pub msg: Binary,
    // Description of proposal
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct QueriedProposal {
    pub id: Uint128,
    pub target: String,
    pub msg: Binary,
    pub description: String,
    pub funding_deadline: u64,
    pub voting_deadline: Option<u64>,
    pub total_funding: Uint128,
    pub status: ProposalStatus,
    pub run_status: Option<ResponseStatus>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    // Admin command called
    AdminRequested,
    // In funding period
    Funding,
    // Voting in progress
    Voting,
    // Total votes did not reach minimum total votes
    Expired,
    // Majority voted No
    Rejected,
    // Majority votes yes
    Passed,
}
