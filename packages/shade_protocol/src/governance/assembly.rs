use cosmwasm_std::{HumanAddr, StdResult, Storage};
use secret_cosmwasm_math_compat::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::utils::flexible_msg::FlexibleMsg;

#[cfg(feature = "governance-impl")]
use crate::utils::storage::BucketStorage;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Assembly {
    // Readable name
    pub name: String,
    // Description of the assembly, preferably in base64
    pub metadata: String,
    // List of members in assembly
    pub members: Vec<HumanAddr>,
    // Selected profile
    pub profile: Uint128,
}

#[cfg(feature = "governance-impl")]
impl Assembly {
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
        AssemblyData {
            members: self.members.clone(),
            profile: self.profile
        }.save(storage, id.to_string().as_bytes())?;

        AssemblyDescription {
            name: self.name.clone(),
            metadata: self.metadata.clone(),
        }.save(storage, id.to_string().as_bytes())?;

        Ok(())
    }

    pub fn data<S: Storage>(storage: &S, id: &Uint128) -> StdResult<AssemblyData> {
        AssemblyData::load(storage, id.to_string().as_bytes())
    }

    pub fn save_data<S: Storage>(storage: &mut S, id: &Uint128, data: AssemblyData) -> StdResult<()> {
        data.save(storage, id.to_string().as_bytes())
    }

    pub fn description<S: Storage>(storage: &S, id: &Uint128) -> StdResult<AssemblyDescription> {
        AssemblyDescription::load(storage, id.to_string().as_bytes())
    }

    pub fn save_description<S: Storage>(storage: &mut S, id: &Uint128, desc: AssemblyDescription) -> StdResult<()> {
        desc.save(storage, id.to_string().as_bytes())
    }
}

#[cfg(feature = "governance-impl")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AssemblyData {
    pub members: Vec<HumanAddr>,
    pub profile: Uint128,
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for AssemblyData {
    const NAMESPACE: &'static [u8] = b"assembly_data-";
}

#[cfg(feature = "governance-impl")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AssemblyDescription {
    pub name: String,
    pub metadata: String,
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for AssemblyDescription {
    const NAMESPACE: &'static [u8] = b"assembly_description-";
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
// A generic msg is created at init, its a black msg where the variable is the start
pub struct AssemblyMsg {
    pub name: String,
    // Assemblys allowed to call this msg
    pub assemblys: Vec<Uint128>,
    // HandleMsg template
    pub msg: FlexibleMsg
}

#[cfg(feature = "governance-impl")]
impl AssemblyMsg {
    pub fn load<S: Storage>(storage: &S, id: &Uint128) -> StdResult<Self> {
        let desc = Self::description(storage, id)?;
        let data = Self::data(storage, id)?;

        Ok(Self {
            name: desc.name,
            assemblys: data.assemblys,
            msg: data.msg
        })
    }

    pub fn save<S: Storage>(&self, storage: &mut S, id: &Uint128) -> StdResult<()> {
        AssemblyMsgData {
            assemblys: self.assemblys.clone(),
            msg: *self.msg
        }.save(storage, id.to_string().as_bytes())?;

        AssemblyMsgDescription {
            name: self.name.clone(),
        }.save(storage, id.to_string().as_bytes())?;

        Ok(())
    }

    pub fn data<S: Storage>(storage: &S, id: &Uint128) -> StdResult<AssemblyMsgData> {
        AssemblyMsgData::load(storage, id.to_string().as_bytes())
    }

    pub fn save_data<S: Storage>(storage: &mut S, id: &Uint128, data: AssemblyMsgData) -> StdResult<()> {
        data.save(storage, id.to_string().as_bytes())
    }

    pub fn description<S: Storage>(storage: &S, id: &Uint128) -> StdResult<AssemblyMsgDescription> {
        AssemblyMsgDescription::load(storage, id.to_string().as_bytes())
    }

    pub fn save_description<S: Storage>(storage: &mut S, id: &Uint128, desc: AssemblyMsgDescription) -> StdResult<()> {
        desc.save(storage, id.to_string().as_bytes())
    }
}

#[cfg(feature = "governance-impl")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AssemblyMsgData {
    pub assemblys: Vec<Uint128>,
    pub msg: FlexibleMsg
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for AssemblyMsgData {
    const NAMESPACE: &'static [u8] = b"assembly_msg_data-";
}

#[cfg(feature = "governance-impl")]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AssemblyMsgDescription {
    pub name: String
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for AssemblyMsgDescription {
    const NAMESPACE: &'static [u8] = b"assembly_msg_description-";
}