use crate::utils::generic_response::ResponseStatus;
use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::governance::vote::VoteTally;
use crate::utils::asset::Contract;

#[cfg(feature = "governance-impl")]
use crate::utils::storage::BucketStorage;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AllowedContract {
    pub name: String,
    pub contract: Contract
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for AllowedContract {
    const NAMESPACE: &'static [u8] = b"allowed_contract-";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Proposal {
    // Target smart contract ID
    pub target: Option<Uint128>,
    // Committee that called the proposal
    pub committee: Uint128,
    // Msg proposal template
    pub committeeMsg: Uint128,
    // Address of the proposal proposer
    pub proposer: HumanAddr,
    // Message to execute
    pub msg: Option<Binary>,
    // Description of proposal, can be in base64
    pub metadata: String,
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for Proposal {
    const NAMESPACE: &'static [u8] = b"proposal-";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CurrentStatus {
    // The current proposal status
    pub status: Status,
    // The deadline for this status
    pub deadline: u64
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for Proposal {
    const NAMESPACE: &'static [u8] = b"current_status-";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    // Committee voting period
    CommitteeVote,
    // In funding period
    Funding,
    // Voting in progress
    Voting,
    // Total votes did not reach minimum total votes
    Expired,
    // Proposal was rejected
    Rejected,
    // Proposal was vetoed
    Vetoed,
    // Proposal was approved
    Passed,
    // If proposal is a msg then it was executed and was successful
    Success,
    // Proposal never got executed after a cancel deadline,
    // assumed that tx failed everytime it got triggered
    Failed
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct QueriedProposal {
    proposal: Proposal,
    status: CurrentStatus,
    states: Vec<ProposalStates>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStates {
    TokenVoting { votes: VoteTally },
    CommitteeVoting { votes: VoteTally },
    Funding { amount: Uint128 }
}