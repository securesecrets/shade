use crate::{
    c_std::{StdResult, Storage, Uint128},
    utils::storage::{
        default::NaiveSingletonStorage,
        plus::{NaiveItemStorage, NaiveMapStorage},
    },
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;
use secret_storage_plus::{Item, Json, Map};

#[cw_serde] // Used to get total IDs
pub struct ID(Uint128);

impl NaiveItemStorage for ID {}

const PROP_KEY: Item<'static, ID, Json> = Item::new("proposal_id-");
const COMMITTEE_KEY: Item<'static, ID, Json> = Item::new("assembly_id-");
const COMMITTEE_MSG_KEY: Item<'static, ID, Json> = Item::new("assembly_msg_id-");
const PROFILE_KEY: Item<'static, ID, Json> = Item::new("profile_id-");
const CONTRACT_KEY: Item<'static, ID, Json> = Item::new("allowed_contract_id-");

// Migration specific data
// Used to determine the next ID to migrate over
const LAST_COMMITTEE_KEY: Item<'static, ID, Json> = Item::new("last_assembly_id-");
const LAST_COMMITTEE_MSG_KEY: Item<'static, ID, Json> = Item::new("last_assembly_msg_id-");
const LAST_PROFILE_KEY: Item<'static, ID, Json> = Item::new("last_profile_id-");
const LAST_CONTRACT_KEY: Item<'static, ID, Json> = Item::new("last_allowed_contract_id-");

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
            Some(i) => ID(i.0.checked_add(Uint128::new(1))?),
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

    // Migration
    pub fn init_migration(storage: &mut dyn Storage) -> StdResult<()> {
        ID(Uint128::zero()).save(storage, LAST_COMMITTEE_KEY)?;
        ID(Uint128::zero()).save(storage, LAST_COMMITTEE_MSG_KEY)?;
        ID(Uint128::zero()).save(storage, LAST_PROFILE_KEY)?;
        ID(Uint128::zero()).save(storage, LAST_CONTRACT_KEY)?;
        Ok(())
    }

    pub fn set_committee_migration(storage: &mut dyn Storage, id: Uint128) -> StdResult<()> {
        ID(id).save(storage, LAST_COMMITTEE_KEY)
    }

    pub fn set_committee_msg_migration(storage: &mut dyn Storage, id: Uint128) -> StdResult<()> {
        ID(id).save(storage, LAST_COMMITTEE_MSG_KEY)
    }

    pub fn set_profile_key_migration(storage: &mut dyn Storage, id: Uint128) -> StdResult<()> {
        ID(id).save(storage, LAST_PROFILE_KEY)
    }

    pub fn set_contract_key_migration(storage: &mut dyn Storage, id: Uint128) -> StdResult<()> {
        ID(id).save(storage, LAST_CONTRACT_KEY)
    }
}

#[cw_serde]
// Used for ease of querying
// TODO: use u64 instead for cheaper storage
pub struct UserID(Uint128);

impl NaiveMapStorage<'static> for UserID {}

// Using user ID cause its practically the same type
const USER_PROP_ID: Map<'static, Addr, UserID> = Map::new("user_proposal_id-");
const USER_PROP: Map<'static, (Addr, u128), UserID> = Map::new("user_proposal_list-");

const USER_ASSEMBLY_VOTE_ID: Map<'static, Addr, UserID> = Map::new("user_assembly_votes_id-");
const USER_ASSEMBLY_VOTE: Map<'static, (Addr, u128), UserID> =
    Map::new("user_assembly_votes_list-");

const USER_FUNDING_ID: Map<'static, Addr, UserID> = Map::new("user_funding_id-");
const USER_FUNDING: Map<'static, (Addr, u128), UserID> = Map::new("user_funding_list-");

const USER_VOTES_ID: Map<'static, Addr, UserID> = Map::new("user_votes_id-");
const USER_VOTES: Map<'static, (Addr, u128), UserID> = Map::new("user_votes_list-");

impl UserID {
    // Stores the proposal's id
    pub fn total_proposals(storage: &dyn Storage, user: Addr) -> StdResult<Uint128> {
        Ok(UserID::may_load(storage, USER_PROP_ID, user)?
            .unwrap_or(UserID(Uint128::zero()))
            .0)
    }

    pub fn proposal(storage: &dyn Storage, user: Addr, id: Uint128) -> StdResult<Uint128> {
        Ok(UserID::load(storage, USER_PROP, (user, id.u128()))?.0)
    }

    pub fn add_proposal(
        storage: &mut dyn Storage,
        user: Addr,
        prop_id: &Uint128,
    ) -> StdResult<Uint128> {
        let item = match UserID::may_load(storage, USER_PROP_ID, user.clone())? {
            None => Uint128::zero(),
            Some(i) => i.0.checked_add(Uint128::new(1))?,
        };
        UserID(item).save(storage, USER_PROP_ID, user.clone())?;
        UserID(prop_id.clone()).save(storage, USER_PROP, (user, item.u128()))?;
        Ok(item)
    }

    // Stores the proposal's ID so it can be cross searched
    pub fn total_assembly_votes(storage: &dyn Storage, user: Addr) -> StdResult<Uint128> {
        Ok(UserID::may_load(storage, USER_ASSEMBLY_VOTE_ID, user)?
            .unwrap_or(UserID(Uint128::zero()))
            .0)
    }

    pub fn assembly_vote(storage: &dyn Storage, user: Addr, id: Uint128) -> StdResult<Uint128> {
        Ok(UserID::load(storage, USER_ASSEMBLY_VOTE, (user, id.u128()))?.0)
    }

    pub fn add_assembly_vote(
        storage: &mut dyn Storage,
        user: Addr,
        prop_id: Uint128,
    ) -> StdResult<Uint128> {
        let item = match UserID::may_load(storage, USER_ASSEMBLY_VOTE_ID, user.clone())? {
            None => Uint128::zero(),
            Some(i) => i.0.checked_add(Uint128::new(1))?,
        };
        UserID(item).save(storage, USER_ASSEMBLY_VOTE_ID, user.clone())?;
        UserID(prop_id).save(storage, USER_ASSEMBLY_VOTE, (user, item.u128()))?;
        Ok(item)
    }

    // Stores the proposal's ID so it can be cross searched
    pub fn total_funding(storage: &dyn Storage, user: Addr) -> StdResult<Uint128> {
        Ok(UserID::may_load(storage, USER_FUNDING_ID, user)?
            .unwrap_or(UserID(Uint128::zero()))
            .0)
    }

    pub fn funding(storage: &dyn Storage, user: Addr, id: Uint128) -> StdResult<Uint128> {
        Ok(UserID::load(storage, USER_FUNDING, (user, id.u128()))?.0)
    }

    pub fn add_funding(
        storage: &mut dyn Storage,
        user: Addr,
        prop_id: Uint128,
    ) -> StdResult<Uint128> {
        let item = match UserID::may_load(storage, USER_FUNDING_ID, user.clone())? {
            None => Uint128::zero(),
            Some(i) => i.0.checked_add(Uint128::new(1))?,
        };
        UserID(item).save(storage, USER_FUNDING_ID, user.clone())?;
        UserID(prop_id).save(storage, USER_FUNDING, (user, item.u128()))?;
        Ok(item)
    }

    // Stores the proposal's ID so it can be cross searched
    pub fn total_votes(storage: &dyn Storage, user: Addr) -> StdResult<Uint128> {
        Ok(UserID::may_load(storage, USER_VOTES_ID, user)?
            .unwrap_or(UserID(Uint128::zero()))
            .0)
    }

    pub fn votes(storage: &dyn Storage, user: Addr, id: Uint128) -> StdResult<Uint128> {
        Ok(UserID::load(storage, USER_VOTES, (user, id.u128()))?.0)
    }

    pub fn add_vote(storage: &mut dyn Storage, user: Addr, prop_id: Uint128) -> StdResult<Uint128> {
        let item = match UserID::may_load(storage, USER_VOTES_ID, user.clone())? {
            None => Uint128::zero(),
            Some(i) => i.0.checked_add(Uint128::new(1))?,
        };
        UserID(item).save(storage, USER_VOTES_ID, user.clone())?;
        UserID(prop_id).save(storage, USER_VOTES, (user, item.u128()))?;
        Ok(item)
    }
}
