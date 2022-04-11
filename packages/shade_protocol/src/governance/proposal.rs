use crate::utils::generic_response::ResponseStatus;
use cosmwasm_std::{Binary, HumanAddr, StdResult, Storage};
use secret_cosmwasm_math_compat::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::governance::stored_id::ID;
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
    pub assemblyMsg: Option<Uint128>,
    // Message to execute
    pub msg: Option<Binary>,

    // Assembly
    // Assembly that called the proposal
    pub assembly: Uint128,

    // Status
    pub status: Status,

    //Status History
    pub status_history: Vec<Status>

    // TODO: add an optional funders list so they can be redeemed later
}

#[cfg(feature = "governance-impl")]
impl Proposal {
    pub fn save<S: Storage>(&self, storage: &mut S, id: &Uint128) -> StdResult<()> {
        Self::save_msg(storage, &id, ProposalMsg{
            target: self.target,
            assemblyMsg: self.assemblyMsg,
            msg: self.msg.clone()
        })?;

        Self::save_description(storage, &id, ProposalDescription {
            proposer: self.proposer.clone(),
            metadata: self.metadata.clone()
        })?;

        Self::save_assembly(storage, &id, self.assembly)?;

        Self::save_status(storage, &id, self.status.clone())?;

        Self::save_status_history(storage, &id, self.status_history.clone())?;

        Ok(())
    }

    pub fn may_load<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Option<Self>> {
        if id > &ID::proposal(storage)? {
            return Ok(None)
        }
        Ok(Some(Self::load(storage, id)?))
    }

    pub fn load<S: Storage>(storage: & S, id: &Uint128) -> StdResult<Self> {
        let msg = Self::msg(storage, id)?;
        let description = Self::description(storage, &id)?;
        let assembly = Self::assembly(storage, &id)?;
        let status = Self::status(storage, &id)?;
        let status_history = Self::status_history(storage, &id)?;

        Ok(Self {
            proposer: description.proposer,
            metadata: description.metadata,
            target: msg.target,
            assemblyMsg: msg.assemblyMsg,
            msg: msg.msg,
            assembly,
            status,
            status_history
        })
    }

    pub fn msg<S: Storage>(storage: &S, id: &Uint128) -> StdResult<ProposalMsg> {
        ProposalMsg::load(storage, &id.to_be_bytes())
    }

    pub fn save_msg<S: Storage>(storage: &mut S, id: &Uint128, data: ProposalMsg) -> StdResult<()> {
        data.save(storage, &id.to_be_bytes())
    }

    pub fn description<S: Storage>(storage: &S, id: &Uint128) -> StdResult<ProposalDescription> {
        ProposalDescription::load(storage, &id.to_be_bytes())
    }

    pub fn save_description<S: Storage>(storage: &mut S, id: &Uint128, data: ProposalDescription) -> StdResult<()> {
        data.save(storage, &id.to_be_bytes())
    }

    pub fn assembly<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Uint128> {
        Ok(ProposalAssembly::load(storage, &id.to_be_bytes())?.0)
    }

    pub fn save_assembly<S: Storage>(storage: &mut S, id: &Uint128, data: Uint128) -> StdResult<()> {
        ProposalAssembly(data).save(storage, &id.to_be_bytes())
    }

    pub fn status<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Status> {
        Status::load(storage, &id.to_be_bytes())
    }

    pub fn save_status<S: Storage>(storage: &mut S, id: &Uint128, data: Status) -> StdResult<()> {
        data.save(storage, &id.to_be_bytes())
    }

    pub fn status_history<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Vec<Status>> {
        Ok(StatusHistory::load(storage, &id.to_be_bytes())?.0)
    }

    pub fn save_status_history<S: Storage>(storage: &mut S, id: &Uint128, data: Vec<Status>) -> StdResult<()> {
        StatusHistory(data).save(storage, &id.to_be_bytes())
    }
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
    pub assemblyMsg: Option<Uint128>,
    pub msg: Option<Binary>,
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for ProposalMsg {
    const NAMESPACE: &'static [u8] = b"proposal_msg-";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProposalAssembly(pub Uint128);

#[cfg(feature = "governance-impl")]
impl BucketStorage for ProposalAssembly {
    const NAMESPACE: &'static [u8] = b"proposal_assembly-";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    // Assembly voting period
    AssemblyVote {votes: VoteTally, start: u64, end:u64},
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
    // Proposal was approved, has a set timeline before it can be canceled
    Passed {start: u64, end: u64},
    // If proposal is a msg then it was executed and was successful
    Success,
    // Proposal never got executed after a cancel deadline,
    // assumed that tx failed everytime it got triggered
    Canceled
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for Status {
    const NAMESPACE: &'static [u8] = b"proposal_status-";
}

#[cfg(feature = "governance-impl")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StatusHistory (pub Vec<Status>);

#[cfg(feature = "governance-impl")]
impl BucketStorage for StatusHistory {
    const NAMESPACE: &'static [u8] = b"proposal_status_history-";
}