use cosmwasm_std::{HumanAddr, StdResult, Storage, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::utils::flexible_msg::FlexibleMsg;

#[cfg(feature = "governance-impl")]
use crate::utils::storage::BucketStorage;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Committee {
    // Readable name
    pub name: String,
    // Description of the committee, preferably in base64
    pub metadata: String,
    // List of members in committee
    pub members: Vec<HumanAddr>,
    // Selected profile
    pub profile: Uint128,
}

#[cfg(feature = "governance-impl")]
impl Committee {
    pub fn load<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Self> {
        let desc = Self::description(storage, id)?;
        let data = Self::data(storage, id)?;

        Ok(Self {
            name: desc.name,
            metadata: desc.metadata,
            members: data.members,
            profile: data.profile
        })
    }

    pub fn save<S: Storage>(&self, storage: &mut S, id: &Uint128) -> StdResult<()> {
        CommitteeData {
            members: self.members.clone(),
            profile: self.profile
        }.save(storage, id.to_string().as_bytes())?;

        CommitteeDescription {
            name: self.name.clone(),
            metadata: self.metadata.clone(),
        }.save(storage, id.to_string().as_bytes())?;

        Ok(())
    }

    pub fn data<S: Storage>(storage: &S, id: &Uint128) -> StdResult<CommitteeData> {
        CommitteeData::load(storage, id.to_string().as_bytes())
    }

    pub fn save_data<S: Storage>(storage: &mut S, id: &Uint128, data: CommitteeData) -> StdResult<()> {
        data.save(storage, id.to_string().as_bytes())
    }

    pub fn description<S: Storage>(storage: &S, id: &Uint128) -> StdResult<CommitteeDescription> {
        CommitteeDescription::load(storage, id.to_string().as_bytes())
    }

    pub fn save_description<S: Storage>(storage: &mut S, id: &Uint128, desc: CommitteeDescription) -> StdResult<()> {
        desc.save(storage, id.to_string().as_bytes())
    }
}

#[cfg(feature = "governance-impl")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CommitteeData {
    pub members: Vec<HumanAddr>,
    pub profile: Uint128,
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for CommitteeData {
    const NAMESPACE: &'static [u8] = b"committee_data-";
}

#[cfg(feature = "governance-impl")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CommitteeDescription {
    pub name: String,
    pub metadata: String,
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for CommitteeDescription {
    const NAMESPACE: &'static [u8] = b"committee_description-";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
// A generic msg is created at init, its a black msg where the variable is the start
pub struct CommitteeMsg {
    pub name: String,
    // Committees allowed to call this msg
    pub committees: Vec<Uint128>,
    // HandleMsg template
    pub msg: FlexibleMsg
}

#[cfg(feature = "governance-impl")]
impl CommitteeMsg {
    pub fn load<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Self> {
        let desc = Self::description(storage, id)?;
        let data = Self::data(storage, id)?;

        Ok(Self {
            name: desc.name,
            committees: data.committees,
            msg: data.msg
        })
    }

    pub fn save<S: Storage>(&self, storage: &mut S, id: &Uint128) -> StdResult<()> {
        CommitteeMsgData {
            committees: self.committees.clone(),
            msg: *self.msg
        }.save(storage, id.to_string().as_bytes())?;

        CommitteeMsgDescription {
            name: self.name.clone(),
        }.save(storage, id.to_string().as_bytes())?;

        Ok(())
    }

    pub fn data<S: Storage>(storage: &S, id: &Uint128) -> StdResult<CommitteeMsgData> {
        CommitteeMsgData::load(storage, id.to_string().as_bytes())
    }

    pub fn save_data<S: Storage>(storage: &mut S, id: &Uint128, data: CommitteeMsgData) -> StdResult<()> {
        data.save(storage, id.to_string().as_bytes())
    }

    pub fn description<S: Storage>(storage: &S, id: &Uint128) -> StdResult<CommitteeMsgDescription> {
        CommitteeMsgDescription::load(storage, id.to_string().as_bytes())
    }

    pub fn save_description<S: Storage>(storage: &mut S, id: &Uint128, desc: CommitteeMsgDescription) -> StdResult<()> {
        desc.save(storage, id.to_string().as_bytes())
    }
}

#[cfg(feature = "governance-impl")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CommitteeMsgData {
    pub committees: Vec<Uint128>,
    pub msg: FlexibleMsg
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for CommitteeMsgData {
    const NAMESPACE: &'static [u8] = b"committee_msg_data-";
}

#[cfg(feature = "governance-impl")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CommitteeMsgDescription {
    pub name: String
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for CommitteeMsgDescription {
    const NAMESPACE: &'static [u8] = b"committee_msg_description-";
}