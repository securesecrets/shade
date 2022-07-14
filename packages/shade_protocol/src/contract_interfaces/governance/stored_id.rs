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
                let item = ID(i.0.checked_add(Uint128::new(1))?);
                item
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
