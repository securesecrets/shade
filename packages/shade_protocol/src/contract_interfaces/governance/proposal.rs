use crate::{
    contract_interfaces::governance::{
        assembly::Assembly,
        profile::Profile,
        stored_id::ID,
        vote::Vote,
    },
    utils::{asset::Contract, generic_response::ResponseStatus},
};
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{Binary, Coin, HumanAddr, StdResult, Storage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[cfg(feature = "governance-impl")]
use crate::utils::storage::default::BucketStorage;
use crate::utils::storage::default::NaiveBucketStorage;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Proposal {
    // Description
    // Address of the proposal proposer
    pub proposer: HumanAddr,
    // Proposal title
    pub title: String,
    // Description of proposal, can be in base64
    pub metadata: String,

    // Msg
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msgs: Option<Vec<ProposalMsg>>,

    // Assembly
    // Assembly that called the proposal
    pub assembly: Uint128,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub assembly_vote_tally: Option<Vote>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_vote_tally: Option<Vote>,

    // Status
    pub status: Status,

    // Status History
    pub status_history: Vec<Status>,

    // Funders
    // Leave as an option so we can hide the data if None
    #[serde(skip_serializing_if = "Option::is_none")]
    pub funders: Option<Vec<(HumanAddr, Uint128)>>,
}

const ASSEMBLY_VOTE: &'static [u8] = b"user-assembly-vote-";
const ASSEMBLY_VOTES: &'static [u8] = b"total-assembly-votes-";
const PUBLIC_VOTE: &'static [u8] = b"user-public-vote-";
const PUBLIC_VOTES: &'static [u8] = b"total-public-votes-";

#[cfg(feature = "governance-impl")]
impl Proposal {
    pub fn save<S: Storage>(&self, storage: &mut S, id: &Uint128) -> StdResult<()> {
        if let Some(msgs) = self.msgs.clone() {
            Self::save_msg(storage, &id, msgs)?;
        }

        Self::save_description(storage, &id, ProposalDescription {
            proposer: self.proposer.clone(),
            title: self.title.clone(),
            metadata: self.metadata.clone(),
        })?;

        Self::save_assembly(storage, &id, self.assembly)?;

        Self::save_status(storage, &id, self.status.clone())?;

        Self::save_status_history(storage, &id, self.status_history.clone())?;

        if let Some(funder_list) = self.funders.clone() {
            let mut funders = vec![];
            for (funder, funding) in funder_list.iter() {
                funders.push(funder.clone());
                Self::save_funding(storage, id, &funder, Funding {
                    amount: *funding,
                    claimed: false,
                })?
            }
            Self::save_funders(storage, id, funders)?;
        }

        Ok(())
    }

    pub fn may_load<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Option<Self>> {
        if id > &ID::proposal(storage)? {
            return Ok(None);
        }
        Ok(Some(Self::load(storage, id)?))
    }

    pub fn load<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Self> {
        let msgs = Self::msg(storage, id)?;
        let description = Self::description(storage, &id)?;
        let assembly = Self::assembly(storage, &id)?;
        let status = Self::status(storage, &id)?;
        let status_history = Self::status_history(storage, &id)?;

        let mut funders_arr = vec![];
        for funder in Self::funders(storage, &id)?.iter() {
            funders_arr.push((funder.clone(), Self::funding(storage, &id, &funder)?.amount))
        }

        let mut funders: Option<Vec<(HumanAddr, Uint128)>> = None;
        if !funders_arr.is_empty() {
            if let Some(prof) =
                Profile::funding(storage, &Assembly::data(storage, &assembly)?.profile)?
            {
                if !prof.privacy {
                    funders = Some(funders_arr);
                }
            }
        }

        let assembly_data = Assembly::data(storage, &assembly)?;

        Ok(Self {
            title: description.title,
            proposer: description.proposer,
            metadata: description.metadata,
            msgs,
            assembly,
            assembly_vote_tally: match Profile::assembly_voting(storage, &assembly_data.profile)? {
                None => None,
                Some(_) => Some(Self::assembly_votes(storage, &id)?),
            },
            public_vote_tally: match Profile::public_voting(storage, &assembly_data.profile)? {
                None => None,
                Some(_) => Some(Self::public_votes(storage, &id)?),
            },
            status,
            status_history,
            funders,
        })
    }

    pub fn msg<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Option<Vec<ProposalMsg>>> {
        match ProposalMsgs::may_load(storage, &id.to_be_bytes())? {
            None => Ok(None),
            Some(i) => Ok(Some(i.0)),
        }
    }

    pub fn save_msg<S: Storage>(
        storage: &mut S,
        id: &Uint128,
        data: Vec<ProposalMsg>,
    ) -> StdResult<()> {
        ProposalMsgs(data).save(storage, &id.to_be_bytes())
    }

    pub fn description<S: Storage>(storage: &S, id: &Uint128) -> StdResult<ProposalDescription> {
        ProposalDescription::load(storage, &id.to_be_bytes())
    }

    pub fn save_description<S: Storage>(
        storage: &mut S,
        id: &Uint128,
        data: ProposalDescription,
    ) -> StdResult<()> {
        data.save(storage, &id.to_be_bytes())
    }

    pub fn assembly<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Uint128> {
        Ok(ProposalAssembly::load(storage, &id.to_be_bytes())?.0)
    }

    pub fn save_assembly<S: Storage>(
        storage: &mut S,
        id: &Uint128,
        data: Uint128,
    ) -> StdResult<()> {
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

    pub fn save_status_history<S: Storage>(
        storage: &mut S,
        id: &Uint128,
        data: Vec<Status>,
    ) -> StdResult<()> {
        StatusHistory(data).save(storage, &id.to_be_bytes())
    }

    pub fn funders<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Vec<HumanAddr>> {
        let funders = match Funders::may_load(storage, &id.to_be_bytes())? {
            None => vec![],
            Some(item) => item.0,
        };
        Ok(funders)
    }

    pub fn save_funders<S: Storage>(
        storage: &mut S,
        id: &Uint128,
        data: Vec<HumanAddr>,
    ) -> StdResult<()> {
        Funders(data).save(storage, &id.to_be_bytes())
    }

    pub fn funding<S: Storage>(storage: &S, id: &Uint128, user: &HumanAddr) -> StdResult<Funding> {
        let key = id.to_string() + "-" + user.as_str();
        Funding::load(storage, key.as_bytes())
    }

    pub fn save_funding<S: Storage>(
        storage: &mut S,
        id: &Uint128,
        user: &HumanAddr,
        data: Funding,
    ) -> StdResult<()> {
        let key = id.to_string() + "-" + user.as_str();
        data.save(storage, key.as_bytes())
    }

    // User assembly votes
    pub fn assembly_vote<S: Storage>(
        storage: &S,
        id: &Uint128,
        user: &HumanAddr,
    ) -> StdResult<Option<Vote>> {
        let key = id.to_string() + "-" + user.as_str();
        Ok(Vote::may_load(storage, ASSEMBLY_VOTE, key.as_bytes())?)
    }

    pub fn save_assembly_vote<S: Storage>(
        storage: &mut S,
        id: &Uint128,
        user: &HumanAddr,
        data: &Vote,
    ) -> StdResult<()> {
        let key = id.to_string() + "-" + user.as_str();
        Vote::write(storage, ASSEMBLY_VOTE).save(key.as_bytes(), data)
    }

    // Total assembly votes
    pub fn assembly_votes<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Vote> {
        match Vote::may_load(storage, ASSEMBLY_VOTES, &id.to_be_bytes())? {
            None => Ok(Vote::default()),
            Some(vote) => Ok(vote),
        }
    }

    pub fn save_assembly_votes<S: Storage>(
        storage: &mut S,
        id: &Uint128,
        data: &Vote,
    ) -> StdResult<()> {
        Vote::write(storage, ASSEMBLY_VOTES).save(&id.to_be_bytes(), data)
    }

    // User public votes
    pub fn public_vote<S: Storage>(
        storage: &S,
        id: &Uint128,
        user: &HumanAddr,
    ) -> StdResult<Option<Vote>> {
        let key = id.to_string() + "-" + user.as_str();
        Ok(Vote::may_load(storage, PUBLIC_VOTE, key.as_bytes())?)
    }

    pub fn save_public_vote<S: Storage>(
        storage: &mut S,
        id: &Uint128,
        user: &HumanAddr,
        data: &Vote,
    ) -> StdResult<()> {
        let key = id.to_string() + "-" + user.as_str();
        Vote::write(storage, PUBLIC_VOTE).save(key.as_bytes(), data)
    }

    // Total public votes
    pub fn public_votes<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Vote> {
        match Vote::may_load(storage, PUBLIC_VOTES, &id.to_be_bytes())? {
            None => Ok(Vote::default()),
            Some(vote) => Ok(vote),
        }
    }

    pub fn save_public_votes<S: Storage>(
        storage: &mut S,
        id: &Uint128,
        data: &Vote,
    ) -> StdResult<()> {
        Vote::write(storage, PUBLIC_VOTES).save(&id.to_be_bytes(), data)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProposalDescription {
    pub proposer: HumanAddr,
    pub title: String,
    pub metadata: String,
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for ProposalDescription {
    const NAMESPACE: &'static [u8] = b"proposal_description-";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProposalMsg {
    pub target: Uint128,
    pub assembly_msg: Uint128,
    // Used as both Vec<String> when calling a handleMsg and Vec<Binary> when saving the msg
    pub msg: Binary,
    pub send: Vec<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
struct ProposalMsgs(pub Vec<ProposalMsg>);

#[cfg(feature = "governance-impl")]
impl BucketStorage for ProposalMsgs {
    const NAMESPACE: &'static [u8] = b"proposal_msgs-";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
struct ProposalAssembly(pub Uint128);

#[cfg(feature = "governance-impl")]
impl BucketStorage for ProposalAssembly {
    const NAMESPACE: &'static [u8] = b"proposal_assembly-";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    // Assembly voting period
    AssemblyVote {
        start: u64,
        end: u64,
    },
    // In funding period
    Funding {
        amount: Uint128,
        start: u64,
        end: u64,
    },
    // Voting in progress
    Voting {
        start: u64,
        end: u64,
    },
    // Total votes did not reach minimum total votes
    Expired,
    // Proposal was rejected
    Rejected,
    // Proposal was vetoed
    // NOTE: percent it stored because proposal settings can change before claiming
    Vetoed {
        slash_percent: Uint128,
    },
    // Proposal was approved, has a set timeline before it can be canceled
    Passed {
        start: u64,
        end: u64,
    },
    // If proposal is a msg then it was executed and was successful
    Success,
    // Proposal never got executed after a cancel deadline,
    // assumed that tx failed everytime it got triggered
    Canceled,
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for Status {
    const NAMESPACE: &'static [u8] = b"proposal_status-";
}

#[cfg(feature = "governance-impl")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
struct StatusHistory(pub Vec<Status>);

#[cfg(feature = "governance-impl")]
impl BucketStorage for StatusHistory {
    const NAMESPACE: &'static [u8] = b"proposal_status_history-";
}

#[cfg(feature = "governance-impl")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
struct Funders(pub Vec<HumanAddr>);

#[cfg(feature = "governance-impl")]
impl BucketStorage for Funders {
    const NAMESPACE: &'static [u8] = b"proposal_funders-";
}

#[cfg(feature = "governance-impl")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Funding {
    pub amount: Uint128,
    pub claimed: bool,
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for Funding {
    const NAMESPACE: &'static [u8] = b"proposal_funding-";
}
