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
pub struct Proposal {
    // Description
    // Address of the proposal proposer
    pub proposer: HumanAddr,
    // Description of proposal, can be in base64
    pub metadata: String,

    // Msg
    // Target smart contract ID
    pub target: Option<Uint128>,
    // Msg proposal template
    pub committeeMsg: Option<Uint128>,
    // Message to execute
    pub msg: Option<Binary>,

    // Committee
    // Committee that called the proposal
    pub committee: Uint128,

    // Status
    pub status: Status,

    //Status History
    pub status_history: Vec<Status>
}

#[cfg(feature = "governance-impl")]
impl Proposal {
    todo!();
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProposalDescription {
    pub proposer: HumanAddr,
    pub metadata: String
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for ProposalDescription {
    const NAMESPACE: &'static [u8] = b"proposal_description-";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProposalMsg {
    pub target: Option<Uint128>,
    pub committeeMsg: Option<Uint128>,
    pub msg: Option<Binary>,
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for ProposalMsg {
    const NAMESPACE: &'static [u8] = b"proposal_msg-";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProposalCommittee(pub Uint128);

#[cfg(feature = "governance-impl")]
impl BucketStorage for ProposalCommittee {
    const NAMESPACE: &'static [u8] = b"proposal_committee-";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    // Committee voting period
    CommitteeVote {votes: VoteTally, start: u64, end:u64},
    // In funding period
    Funding {amount: Uint128, start: u64, end:u64},
    // Voting in progress
    Voting {votes: VoteTally, start: u64, end:u64},
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

#[cfg(feature = "governance-impl")]
impl BucketStorage for Status {
    const NAMESPACE: &'static [u8] = b"proposal_status-";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StatusHistory (pub Vec<Status>);

#[cfg(feature = "governance-impl")]
impl BucketStorage for StatusHistory {
    const NAMESPACE: &'static [u8] = b"proposal_status_history-";
}