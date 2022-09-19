use crate::{
    c_std::{Addr, Binary, Coin, StdResult, Storage, Uint128},
    contract_interfaces::governance::{
        assembly::Assembly,
        profile::Profile,
        stored_id::{UserID, ID},
        vote::Vote,
    },
};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::Timestamp;
use secret_storage_plus::Map;

#[cfg(feature = "governance-impl")]
use crate::utils::storage::plus::{MapStorage, NaiveMapStorage};

#[cw_serde]
pub struct Proposal {
    // Description
    // Address of the proposal proposer
    pub proposer: Addr,
    // Proposal title
    pub title: String,
    // Description of proposal, can be in base64
    pub metadata: String,

    // Msg
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msgs: Option<Vec<ProposalMsg>>,

    // Assembly
    // Assembly that called the proposal
    pub assembly: u16,

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
    pub funders: Option<Vec<(Addr, Uint128)>>,
}

const ASSEMBLY_VOTE: Map<'static, (u32, Addr), Vote> = Map::new("user-assembly-vote-");
const ASSEMBLY_VOTES: Map<'static, u32, Vote> = Map::new("total-assembly-votes-");
const PUBLIC_VOTE: Map<'static, (u32, Addr), Vote> = Map::new("user-public-vote-");
const PUBLIC_VOTES: Map<'static, u32, Vote> = Map::new("total-public-votes-");

#[cfg(feature = "governance-impl")]
impl Proposal {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        // Create new ID
        let id = ID::add_proposal(storage)?;

        // Create proposers id
        UserID::add_proposal(storage, self.proposer.clone(), &id)?;

        if let Some(msgs) = self.msgs.clone() {
            Self::save_msg(storage, id, msgs)?;
        }

        Self::save_description(storage, id, ProposalDescription {
            proposer: self.proposer.clone(),
            title: self.title.clone(),
            metadata: self.metadata.clone(),
        })?;

        Self::save_assembly(storage, id, self.assembly)?;

        Self::save_status(storage, id, self.status.clone())?;

        Self::save_status_history(storage, id, self.status_history.clone())?;

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

    pub fn may_load(storage: &dyn Storage, id: u32) -> StdResult<Option<Self>> {
        if id > ID::proposal(storage)? {
            return Ok(None);
        }
        Ok(Some(Self::load(storage, id)?))
    }

    pub fn load(storage: &dyn Storage, id: u32) -> StdResult<Self> {
        let msgs = Self::msg(storage, id)?;
        let description = Self::description(storage, id)?;
        let assembly = Self::assembly(storage, id)?;
        let status = Self::status(storage, id)?;
        let status_history = Self::status_history(storage, id)?;

        let mut funders_arr = vec![];
        for funder in Self::funders(storage, id)?.iter() {
            funders_arr.push((funder.clone(), Self::funding(storage, id, &funder)?.amount))
        }

        let mut funders: Option<Vec<(Addr, Uint128)>> = None;
        if !funders_arr.is_empty() {
            if let Some(prof) =
                Profile::funding(storage, Assembly::data(storage, assembly)?.profile)?
            {
                if !prof.privacy {
                    funders = Some(funders_arr);
                }
            }
        }

        let assembly_data = Assembly::data(storage, assembly)?;

        Ok(Self {
            title: description.title,
            proposer: description.proposer,
            metadata: description.metadata,
            msgs,
            assembly,
            assembly_vote_tally: match Profile::assembly_voting(storage, assembly_data.profile)? {
                None => None,
                Some(_) => Some(Self::assembly_votes(storage, id)?),
            },
            public_vote_tally: match Profile::public_voting(storage, assembly_data.profile)? {
                None => None,
                Some(_) => Some(Self::public_votes(storage, id)?),
            },
            status,
            status_history,
            funders,
        })
    }

    pub fn msg(storage: &dyn Storage, id: u32) -> StdResult<Option<Vec<ProposalMsg>>> {
        match ProposalMsgs::may_load(storage, id)? {
            None => Ok(None),
            Some(i) => Ok(Some(i.0)),
        }
    }

    pub fn save_msg(storage: &mut dyn Storage, id: u32, data: Vec<ProposalMsg>) -> StdResult<()> {
        ProposalMsgs(data).save(storage, id)
    }

    pub fn description(storage: &dyn Storage, id: u32) -> StdResult<ProposalDescription> {
        ProposalDescription::load(storage, id)
    }

    pub fn save_description(
        storage: &mut dyn Storage,
        id: u32,
        data: ProposalDescription,
    ) -> StdResult<()> {
        data.save(storage, id)
    }

    pub fn assembly(storage: &dyn Storage, id: u32) -> StdResult<u16> {
        Ok(ProposalAssembly::load(storage, id)?.0)
    }

    pub fn save_assembly(storage: &mut dyn Storage, id: u32, data: u16) -> StdResult<()> {
        ProposalAssembly(data).save(storage, id)
    }

    pub fn status(storage: &dyn Storage, id: u32) -> StdResult<Status> {
        Status::load(storage, id)
    }

    pub fn save_status(storage: &mut dyn Storage, id: u32, data: Status) -> StdResult<()> {
        data.save(storage, id)
    }

    pub fn status_history(storage: &dyn Storage, id: u32) -> StdResult<Vec<Status>> {
        Ok(StatusHistory::load(storage, id)?.0)
    }

    pub fn save_status_history(
        storage: &mut dyn Storage,
        id: u32,
        data: Vec<Status>,
    ) -> StdResult<()> {
        StatusHistory(data).save(storage, id)
    }

    pub fn funders(storage: &dyn Storage, id: u32) -> StdResult<Vec<Addr>> {
        let funders = match Funders::may_load(storage, id)? {
            None => vec![],
            Some(item) => item.0,
        };
        Ok(funders)
    }

    pub fn save_funders(storage: &mut dyn Storage, id: u32, data: Vec<Addr>) -> StdResult<()> {
        Funders(data).save(storage, id)
    }

    pub fn funding(storage: &dyn Storage, id: u32, user: &Addr) -> StdResult<Funding> {
        Funding::load(storage, (id, user.clone()))
    }

    pub fn save_funding(
        storage: &mut dyn Storage,
        id: u32,
        user: &Addr,
        data: Funding,
    ) -> StdResult<()> {
        data.save(storage, (id, user.clone()))
    }

    // User assembly votes
    pub fn assembly_vote(storage: &dyn Storage, id: u32, user: &Addr) -> StdResult<Option<Vote>> {
        Ok(Vote::may_load(storage, ASSEMBLY_VOTE, (id, user.clone()))?)
    }

    pub fn save_assembly_vote(
        storage: &mut dyn Storage,
        id: u32,
        user: &Addr,
        data: &Vote,
    ) -> StdResult<()> {
        data.save(storage, ASSEMBLY_VOTE, (id, user.clone()))
    }

    // Total assembly votes
    pub fn assembly_votes(storage: &dyn Storage, id: u32) -> StdResult<Vote> {
        match Vote::may_load(storage, ASSEMBLY_VOTES, id)? {
            None => Ok(Vote::default()),
            Some(vote) => Ok(vote),
        }
    }

    pub fn save_assembly_votes(storage: &mut dyn Storage, id: u32, data: &Vote) -> StdResult<()> {
        data.save(storage, ASSEMBLY_VOTES, id)
    }

    // User public votes
    pub fn public_vote(storage: &dyn Storage, id: u32, user: &Addr) -> StdResult<Option<Vote>> {
        Ok(Vote::may_load(storage, PUBLIC_VOTE, (id, user.clone()))?)
    }

    pub fn save_public_vote(
        storage: &mut dyn Storage,
        id: u32,
        user: &Addr,
        data: &Vote,
    ) -> StdResult<()> {
        data.save(storage, PUBLIC_VOTE, (id, user.clone()))
    }

    // Total public votes
    pub fn public_votes(storage: &dyn Storage, id: u32) -> StdResult<Vote> {
        match Vote::may_load(storage, PUBLIC_VOTES, id)? {
            None => Ok(Vote::default()),
            Some(vote) => Ok(vote),
        }
    }

    pub fn save_public_votes(storage: &mut dyn Storage, id: u32, data: &Vote) -> StdResult<()> {
        data.save(storage, PUBLIC_VOTES, id)
    }
}

#[cw_serde]
pub struct ProposalDescription {
    pub proposer: Addr,
    pub title: String,
    pub metadata: String,
}

#[cfg(feature = "governance-impl")]
impl MapStorage<'static, u32> for ProposalDescription {
    const MAP: Map<'static, u32, Self> = Map::new("proposal_description-");
}

#[cw_serde]
pub struct ProposalMsg {
    pub target: u16,
    pub assembly_msg: u16,
    // Used as both Vec<String> when calling a handleMsg and Vec<Binary> when saving the msg
    pub msg: Binary,
    pub send: Vec<Coin>,
}

#[cw_serde]
struct ProposalMsgs(pub Vec<ProposalMsg>);

#[cfg(feature = "governance-impl")]
impl MapStorage<'static, u32> for ProposalMsgs {
    const MAP: Map<'static, u32, Self> = Map::new("proposal_msgs-");
}

#[cw_serde]
struct ProposalAssembly(pub u16);

#[cfg(feature = "governance-impl")]
impl MapStorage<'static, u32> for ProposalAssembly {
    const MAP: Map<'static, u32, Self> = Map::new("proposal_assembly-");
}

#[cw_serde]
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

impl Status {
    pub fn passed(storage: &dyn Storage, profile: u16, time: &Timestamp) -> StdResult<Status> {
        let seconds = time.seconds();
        Ok(Self::Passed {
            start: seconds,
            end: seconds + Profile::data(storage, profile)?.cancel_deadline,
        })
    }
}

#[cfg(feature = "governance-impl")]
impl MapStorage<'static, u32> for Status {
    const MAP: Map<'static, u32, Self> = Map::new("proposal_status-");
}

#[cfg(feature = "governance-impl")]
#[cw_serde]
struct StatusHistory(pub Vec<Status>);

#[cfg(feature = "governance-impl")]
impl MapStorage<'static, u32> for StatusHistory {
    const MAP: Map<'static, u32, Self> = Map::new("proposal_status_history-");
}

#[cfg(feature = "governance-impl")]
#[cw_serde]
struct Funders(pub Vec<Addr>);

#[cfg(feature = "governance-impl")]
impl MapStorage<'static, u32> for Funders {
    const MAP: Map<'static, u32, Self> = Map::new("proposal_funders-");
}

#[cfg(feature = "governance-impl")]
#[cw_serde]
pub struct Funding {
    pub amount: Uint128,
    pub claimed: bool,
}

#[cfg(feature = "governance-impl")]
impl MapStorage<'static, (u32, Addr)> for Funding {
    const MAP: Map<'static, (u32, Addr), Self> = Map::new("proposal_funding-");
}
