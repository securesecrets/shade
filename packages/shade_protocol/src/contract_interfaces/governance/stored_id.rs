use crate::utils::storage::default::NaiveSingletonStorage;
use crate::c_std::Uint128;
use crate::c_std::{StdResult, Storage};
use cosmwasm_schema::{cw_serde};

#[cw_serde]// Used to get total IDs
pub struct ID(Uint128);

impl NaiveSingletonStorage for ID {}

const PROP_KEY: &'static [u8] = b"proposal_id-";
const COMMITTEE_KEY: &'static [u8] = b"assembly_id-";
const COMMITTEE_MSG_KEY: &'static [u8] = b"assembly_msg_id-";
const PROFILE_KEY: &'static [u8] = b"profile_id-";
const CONTRACT_KEY: &'static [u8] = b"allowed_contract_id-";

impl ID {
    // Load current ID related proposals
    pub fn set_proposal(storage: &mut dyn Storage, id: Uint128) -> StdResult<()> {
        ID(id).save(storage, PROP_KEY)
    }

    pub fn proposal(storage: &dyn Storage) -> StdResult<Uint128> {
        Ok(ID::load(storage, PROP_KEY)?.0)
    }

    pub fn add_proposal(storage: &mut dyn Storage) -> StdResult<Uint128> {
        let item = match ID::may_load(storage, PROP_KEY)? {
            None => ID(Uint128::zero()),
            Some(i) => {
                ID(i.0.checked_add(Uint128::new(1))?)
            }
        };
        item.save(storage, PROP_KEY)?;
        Ok(item.0)
    }

    // Assembly
    pub fn set_assembly(storage: &mut dyn Storage, id: Uint128) -> StdResult<()> {
        ID(id).save(storage, COMMITTEE_KEY)
    }

    pub fn assembly(storage: &dyn Storage) -> StdResult<Uint128> {
        Ok(ID::load(storage, COMMITTEE_KEY)?.0)
    }

    pub fn add_assembly(storage: &mut dyn Storage) -> StdResult<Uint128> {
        let mut item = ID::load(storage, COMMITTEE_KEY)?;
        item.0 += Uint128::new(1);
        item.save(storage, COMMITTEE_KEY)?;
        Ok(item.0)
    }

    // Assembly Msg
    pub fn set_assembly_msg(storage: &mut dyn Storage, id: Uint128) -> StdResult<()> {
        ID(id).save(storage, COMMITTEE_MSG_KEY)
    }

    pub fn assembly_msg(storage: &dyn Storage) -> StdResult<Uint128> {
        Ok(ID::load(storage, COMMITTEE_MSG_KEY)?.0)
    }

    pub fn add_assembly_msg(storage: &mut dyn Storage) -> StdResult<Uint128> {
        let mut item = ID::load(storage, COMMITTEE_MSG_KEY)?;
        item.0 += Uint128::new(1);
        item.save(storage, COMMITTEE_MSG_KEY)?;
        Ok(item.0)
    }

    // Profile
    pub fn set_profile(storage: &mut dyn Storage, id: Uint128) -> StdResult<()> {
        ID(id).save(storage, PROFILE_KEY)
    }

    pub fn profile(storage: &dyn Storage) -> StdResult<Uint128> {
        Ok(ID::load(storage, PROFILE_KEY)?.0)
    }

    pub fn add_profile(storage: &mut dyn Storage) -> StdResult<Uint128> {
        let mut item = ID::load(storage, PROFILE_KEY)?;
        item.0 += Uint128::new(1);
        item.save(storage, PROFILE_KEY)?;
        Ok(item.0)
    }

    // Contract
    // Profile
    pub fn set_contract(storage: &mut dyn Storage, id: Uint128) -> StdResult<()> {
        ID(id).save(storage, CONTRACT_KEY)
    }

    pub fn contract(storage: &dyn Storage) -> StdResult<Uint128> {
        Ok(ID::load(storage, CONTRACT_KEY)?.0)
    }

    pub fn add_contract(storage: &mut dyn Storage) -> StdResult<Uint128> {
        let mut item = ID::load(storage, CONTRACT_KEY)?;
        item.0 += Uint128::new(1);
        item.save(storage, CONTRACT_KEY)?;
        Ok(item.0)
    }
}

#[cw_serde]
// Used for ease of querying
// TODO: use u64 instead for faster storage
pub struct UserID(Uint128);

impl NaiveMapStorage<'static> for UserID {}

// Using user ID cause its practically the same type
const USER_PROP_ID: Map<'static, Addr, UserID> = Map::new("user_proposal_id-");
const USER_PROP: Map<'static, (Addr, u128), UserID> = Map::new("user_proposal_list-");

const USER_ASSEMBLY_VOTE_ID: Map<'static, Addr, UserID> = Map::new("user_assembly_votes_id-");
const USER_ASSEMBLY_VOTE: Map<'static, (Addr, u128), UserID> = Map::new("user_assembly_votes_list-");

const USER_FUNDING_ID: Map<'static, Addr, UserID> = Map::new("user_funding_id-");
const USER_FUNDING: Map<'static, (Addr, u128), UserID> = Map::new("user_funding_list-");

const USER_VOTES_ID: Map<'static, Addr, UserID> = Map::new("user_votes_id-");
const USER_VOTES: Map<'static, (Addr, u128), UserID> = Map::new("user_votes_list-");

impl UserID {
    // Stores the proposal's id
    pub fn total_proposals<S: Storage>(storage: & S, user: Addr) -> StdResult<Uint128> {
        Ok(UserID::may_load(storage, USER_PROP_ID, user)?.unwrap_or(UserID(Uint128::zero())).0)
    }

    pub fn proposal<S: Storage>(storage: & S, user: Addr, id: Uint128) -> StdResult<Uint128> {
        Ok(UserID::load(storage, USER_PROP, (user, id.u128()))?.0)
    }

    pub fn add_proposal<S: Storage>(storage: &mut S, user: Addr, prop_id: Uint128) -> StdResult<Uint128> {
        let item = match UserID::may_load(storage, USER_PROP_ID, user.clone())? {
            None => Uint128::zero(),
            Some(i) => i.0.checked_add(Uint128::new(1))?
        };
        UserID(item).save(storage, USER_PROP_ID, user.clone())?;
        UserID(prop_id).save(storage, USER_PROP, (user, item.u128()))?;
        Ok(item)
    }

    // Stores the proposal's ID so it can be cross searched
    pub fn total_assembly_votes<S: Storage>(storage: & S, user: Addr) -> StdResult<Uint128> {
        Ok(UserID::may_load(storage, USER_ASSEMBLY_VOTE_ID, user)?.unwrap_or(UserID(Uint128::zero())).0)
    }

    pub fn assembly_vote<S: Storage>(storage: & S, user: Addr, id: Uint128) -> StdResult<Uint128> {
        Ok(UserID::load(storage, USER_ASSEMBLY_VOTE, (user, id.u128()))?.0)
    }

    pub fn add_assembly_vote<S: Storage>(storage: &mut S, user: Addr, prop_id: Uint128) -> StdResult<Uint128> {
        let item = match UserID::may_load(storage, USER_ASSEMBLY_VOTE_ID, user.clone())? {
            None => Uint128::zero(),
            Some(i) => i.0.checked_add(Uint128::new(1))?
        };
        UserID(item).save(storage, USER_ASSEMBLY_VOTE_ID, user.clone())?;
        UserID(prop_id).save(storage, USER_ASSEMBLY_VOTE, (user, item.u128()))?;
        Ok(item)
    }

    // Stores the proposal's ID so it can be cross searched
    pub fn total_funding<S: Storage>(storage: & S, user: Addr) -> StdResult<Uint128> {
        Ok(UserID::may_load(storage, USER_FUNDING_ID, user)?.unwrap_or(UserID(Uint128::zero())).0)
    }

    pub fn funding<S: Storage>(storage: & S, user: Addr, id: Uint128) -> StdResult<Uint128> {
        Ok(UserID::load(storage, USER_FUNDING, (user, id.u128()))?.0)
    }

    pub fn add_funding<S: Storage>(storage: &mut S, user: Addr, prop_id: Uint128) -> StdResult<Uint128> {
        let item = match UserID::may_load(storage, USER_FUNDING_ID, user.clone())? {
            None => Uint128::zero(),
            Some(i) => i.0.checked_add(Uint128::new(1))?
        };
        UserID(item).save(storage, USER_FUNDING_ID, user.clone())?;
        UserID(prop_id).save(storage, USER_FUNDING, (user, item.u128()))?;
        Ok(item)
    }

    // Stores the proposal's ID so it can be cross searched
    pub fn total_votes<S: Storage>(storage: & S, user: Addr) -> StdResult<Uint128> {
        Ok(UserID::may_load(storage, USER_VOTES_ID, user)?.unwrap_or(UserID(Uint128::zero())).0)
    }

    pub fn votes<S: Storage>(storage: & S, user: Addr, id: Uint128) -> StdResult<Uint128> {
        Ok(UserID::load(storage, USER_VOTES, (user, id.u128()))?.0)
    }

    pub fn add_vote<S: Storage>(storage: &mut S, user: Addr, prop_id: Uint128) -> StdResult<Uint128> {
        let item = match UserID::may_load(storage, USER_VOTES_ID, user.clone())? {
            None => Uint128::zero(),
            Some(i) => i.0.checked_add(Uint128::new(1))?
        };
        UserID(item).save(storage, USER_VOTES_ID, user.clone())?;
        UserID(prop_id).save(storage, USER_VOTES, (user, item.u128()))?;
        Ok(item)
    }
}