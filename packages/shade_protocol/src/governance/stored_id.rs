use cosmwasm_std::{StdResult, Storage};
use secret_cosmwasm_math_compat::Uint128;
use serde::{Deserialize, Serialize};
use crate::utils::storage::NaiveSingletonStorage;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
// Used to get total IDs
pub struct ID(Uint128);

impl NaiveSingletonStorage for ID {}

impl ID {
    const PROP_KEY: &'static [u8] = b"proposal_id-";
    const COMMITTEE_KEY: &'static [u8] = b"assembly_id-";
    const COMMITTEE_MSG_KEY: &'static [u8] = b"assembly_msg_id-";
    const PROFILE_KEY: &'static [u8] = b"profile_id-";
    const CONTRACT_KEY: &'static [u8] = b"allowed_contract_id-";

    // Load current ID related proposals
    pub fn set_proposal<S: Storage>(storage: &mut S, id: Uint128) -> StdResult<()> {
        ID::write(storage, PROP_KEY).save(&ID(id))
    }

    pub fn proposal<S: Storage>(storage: &S) -> StdResult<Uint128> {
        Ok(ID::read(storage, PROP_KEY).load()?.0)
    }

    pub fn add_proposal<S: Storage>(storage: &mut S) -> StdResult<Uint128> {
        let mut item = ID::read(storage, PROP_KEY).load()?;
        item.0 += Uint128(1);
        ID::write(storage, PROP_KEY).save(&item)?;
        Ok(item.0)
    }

    // Assembly
    pub fn set_assembly<S: Storage>(storage: &mut S, id: Uint128) -> StdResult<()> {
        ID::write(storage, COMMITTEE_KEY).save(&ID(id))
    }

    pub fn assembly<S: Storage>(storage: &S) -> StdResult<Uint128> {
        Ok(ID::read(storage, COMMITTEE_KEY).load()?.0)
    }

    pub fn add_assembly<S: Storage>(storage: &mut S) -> StdResult<Uint128> {
        let mut item = ID::read(storage, COMMITTEE_KEY).load()?;
        item.0 += Uint128(1);
        ID::write(storage, COMMITTEE_KEY).save(&item)?;
        Ok(item.0)
    }

    // Assembly Msg
    pub fn set_assembly_msg<S: Storage>(storage: &mut S, id: Uint128) -> StdResult<()> {
        ID::write(storage, COMMITTEE_MSG_KEY).save(&ID(id))
    }

    pub fn assembly_msg<S: Storage>(storage: &S) -> StdResult<Uint128> {
        Ok(ID::read(storage, COMMITTEE_MSG_KEY).load()?.0)
    }

    pub fn add_assembly_msg<S: Storage>(storage: &mut S) -> StdResult<Uint128> {
        let mut item = ID::read(storage, COMMITTEE_MSG_KEY).load()?;
        item.0 += Uint128(1);
        ID::write(storage, COMMITTEE_MSG_KEY).save(&item)?;
        Ok(item.0)
    }

    // Profile
    pub fn set_profile<S: Storage>(storage: &mut S, id: Uint128) -> StdResult<()> {
        ID::write(storage, PROFILE_KEY).save(&ID(id))
    }

    pub fn profile<S: Storage>(storage: &S) -> StdResult<Uint128> {
        Ok(ID::read(storage, PROFILE_KEY).load()?.0)
    }

    pub fn add_profile<S: Storage>(storage: &mut S) -> StdResult<Uint128> {
        let mut item = ID::read(storage, PROFILE_KEY).load()?;
        item.0 += Uint128(1);
        ID::write(storage, PROFILE_KEY).save(&item)?;
        Ok(item.0)
    }

    // Contract
    // Profile
    pub fn set_contract<S: Storage>(storage: &mut S, id: Uint128) -> StdResult<()> {
        ID::write(storage, CONTRACT_KEY).save(&ID(id))
    }

    pub fn contract<S: Storage>(storage: &S) -> StdResult<Uint128> {
        Ok(ID::read(storage, CONTRACT_KEY).load()?.0)
    }

    pub fn add_contract<S: Storage>(storage: &mut S) -> StdResult<Uint128> {
        let mut item = ID::read(storage, CONTRACT_KEY).load()?;
        item.0 += Uint128(1);
        ID::write(storage, CONTRACT_KEY).save(&item)?;
        Ok(item.0)
    }

}