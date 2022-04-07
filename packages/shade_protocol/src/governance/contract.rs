use cosmwasm_std::{StdResult, Storage, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::utils::asset::Contract;
use crate::utils::storage::BucketStorage;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AllowedContract {
    pub name: String,
    pub metadata: String,
    pub contract: Contract
}

#[cfg(feature = "governance-impl")]
impl AllowedContract {
    pub fn load<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Self> {
        let desc = Self::description(storage, id)?;
        let data = Self::data(storage, id)?;

        Ok(Self {
            name: desc.name,
            metadata: desc.metadata,
            contract: data.contract
        })
    }

    pub fn save<S: Storage>(&self, storage: &mut S, id: &Uint128) -> StdResult<()> {
        AllowedContractData {
            contract: *self.contract
        }.save(storage, id.to_string().as_bytes())?;

        AllowedContractDescription {
            name: self.name.clone(),
            metadata: self.metadata.clone(),
        }.save(storage, id.to_string().as_bytes())?;

        Ok(())
    }

    pub fn data<S: Storage>(storage: &S, id: &Uint128) -> StdResult<AllowedContractData> {
        AllowedContractData::load(storage, id.to_string().as_bytes())
    }

    pub fn save_data<S: Storage>(storage: &mut S, id: &Uint128, data: AllowedContractData) -> StdResult<()> {
        data.save(storage, id.to_string().as_bytes())
    }

    pub fn description<S: Storage>(storage: &S, id: &Uint128) -> StdResult<AllowedContractDescription> {
        AllowedContractDescription::load(storage, id.to_string().as_bytes())
    }

    pub fn save_description<S: Storage>(storage: &mut S, id: &Uint128, desc: AllowedContractDescription) -> StdResult<()> {
        desc.save(storage, id.to_string().as_bytes())
    }
}

#[cfg(feature = "governance-impl")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AllowedContractData {
    pub contract: Contract
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for AllowedContractData {
    const NAMESPACE: &'static [u8] = b"allowed_contract_data-";
}

#[cfg(feature = "governance-impl")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AllowedContractDescription {
    pub name: String,
    pub metadata: String,
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for AllowedContractDescription {
    const NAMESPACE: &'static [u8] = b"allowed_contract_description-";
}