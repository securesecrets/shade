use crate::{
    c_std::{StdResult, Storage},
    contract_interfaces::governance::stored_id::ID,
    utils::asset::Contract,
};

use crate::utils::storage::plus::MapStorage;
use cosmwasm_schema::cw_serde;
use secret_storage_plus::Map;

#[cw_serde]
pub struct AllowedContract {
    pub name: String,
    pub metadata: String,
    // If none then anyone can use it
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assemblies: Option<Vec<u16>>,
    pub contract: Contract,
}

#[cfg(feature = "governance-impl")]
impl AllowedContract {
    pub fn load(storage: &dyn Storage, id: u16) -> StdResult<Self> {
        let desc = Self::description(storage, id)?;
        let data = Self::data(storage, id)?;

        Ok(Self {
            name: desc.name,
            metadata: desc.metadata,
            contract: data.contract,
            assemblies: data.assemblies,
        })
    }

    pub fn may_load(storage: &dyn Storage, id: u16) -> StdResult<Option<Self>> {
        if id > ID::contract(storage)? {
            return Ok(None);
        }
        Ok(Some(Self::load(storage, id)?))
    }

    pub fn save(&self, storage: &mut dyn Storage, id: u16) -> StdResult<()> {
        AllowedContractData {
            contract: self.contract.clone(),
            assemblies: self.assemblies.clone(),
        }
        .save(storage, id)?;

        AllowedContractDescription {
            name: self.name.clone(),
            metadata: self.metadata.clone(),
        }
        .save(storage, id)?;

        Ok(())
    }

    pub fn data(storage: &dyn Storage, id: u16) -> StdResult<AllowedContractData> {
        AllowedContractData::load(storage, id)
    }

    pub fn save_data(
        storage: &mut dyn Storage,
        id: u16,
        data: AllowedContractData,
    ) -> StdResult<()> {
        data.save(storage, id)
    }

    pub fn description(storage: &dyn Storage, id: u16) -> StdResult<AllowedContractDescription> {
        AllowedContractDescription::load(storage, id)
    }

    pub fn save_description(
        storage: &mut dyn Storage,
        id: u16,
        desc: AllowedContractDescription,
    ) -> StdResult<()> {
        desc.save(storage, id)
    }
}

#[cfg(feature = "governance-impl")]
#[cw_serde]
pub struct AllowedContractData {
    pub contract: Contract,
    pub assemblies: Option<Vec<u16>>,
}

#[cfg(feature = "governance-impl")]
impl MapStorage<'static, u16> for AllowedContractData {
    const MAP: Map<'static, u16, Self> = Map::new("allowed_contract_data-");
}

#[cfg(feature = "governance-impl")]
#[cw_serde]
pub struct AllowedContractDescription {
    pub name: String,
    pub metadata: String,
}

#[cfg(feature = "governance-impl")]
impl MapStorage<'static, u16> for AllowedContractDescription {
    const MAP: Map<'static, u16, Self> = Map::new("allowed_contract_description-");
}
