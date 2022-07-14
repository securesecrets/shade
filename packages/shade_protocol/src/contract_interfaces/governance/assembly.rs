use crate::{contract_interfaces::governance::stored_id::ID, utils::flexible_msg::FlexibleMsg};
use crate::c_std::Uint128;
use crate::c_std::{Addr, StdResult, Storage};

use cosmwasm_schema::{cw_serde};

#[cfg(feature = "governance-impl")]
use crate::utils::storage::default::BucketStorage;

#[cw_serde]
pub struct Assembly {
    // Readable name
    pub name: String,
    // Description of the assembly, preferably in base64
    pub metadata: String,
    // List of members in assembly
    pub members: Vec<Addr>,
    // Selected profile
    pub profile: Uint128,
}

#[cfg(feature = "governance-impl")]
impl Assembly {
    pub fn load(storage: &dyn Storage, id: &Uint128) -> StdResult<Self> {
        let desc = Self::description(storage, id)?;
        let data = Self::data(storage, id)?;

        Ok(Self {
            name: desc.name,
            metadata: desc.metadata,
            members: data.members,
            profile: data.profile,
        })
    }

    pub fn may_load(storage: &dyn Storage, id: &Uint128) -> StdResult<Option<Self>> {
        if id > &ID::assembly(storage)? {
            return Ok(None);
        }
        Ok(Some(Self::load(storage, id)?))
    }

    pub fn save(&self, storage: &mut dyn Storage, id: &Uint128) -> StdResult<()> {
        AssemblyData {
            members: self.members.clone(),
            profile: self.profile,
        }
        .save(storage, &id.to_be_bytes())?;

        AssemblyDescription {
            name: self.name.clone(),
            metadata: self.metadata.clone(),
        }
        .save(storage, &id.to_be_bytes())?;

        Ok(())
    }

    pub fn data(storage: &dyn Storage, id: &Uint128) -> StdResult<AssemblyData> {
        AssemblyData::load(storage, &id.to_be_bytes())
    }

    pub fn save_data(
        storage: &mut dyn Storage,
        id: &Uint128,
        data: AssemblyData,
    ) -> StdResult<()> {
        data.save(storage, &id.to_be_bytes())
    }

    pub fn description(storage: &dyn Storage, id: &Uint128) -> StdResult<AssemblyDescription> {
        AssemblyDescription::load(storage, &id.to_be_bytes())
    }

    pub fn save_description(
        storage: &mut dyn Storage,
        id: &Uint128,
        desc: AssemblyDescription,
    ) -> StdResult<()> {
        desc.save(storage, &id.to_be_bytes())
    }
}

#[cfg(feature = "governance-impl")]
#[cw_serde]
pub struct AssemblyData {
    pub members: Vec<Addr>,
    pub profile: Uint128,
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for AssemblyData {
    const NAMESPACE: &'static [u8] = b"assembly_data-";
}

#[cfg(feature = "governance-impl")]
#[cw_serde]
pub struct AssemblyDescription {
    pub name: String,
    pub metadata: String,
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for AssemblyDescription {
    const NAMESPACE: &'static [u8] = b"assembly_description-";
}

#[cw_serde]// A generic msg is created at init, its a black msg where the variable is the start
pub struct AssemblyMsg {
    pub name: String,
    // Assemblies allowed to call this msg
    pub assemblies: Vec<Uint128>,
    // ExecuteMsg template
    pub msg: FlexibleMsg,
}

#[cfg(feature = "governance-impl")]
impl AssemblyMsg {
    pub fn load(storage: &dyn Storage, id: &Uint128) -> StdResult<Self> {
        let desc = Self::description(storage, id)?;
        let data = Self::data(storage, id)?;

        Ok(Self {
            name: desc,
            assemblies: data.assemblies,
            msg: data.msg,
        })
    }

    pub fn may_load(storage: &dyn Storage, id: &Uint128) -> StdResult<Option<Self>> {
        if id > &ID::assembly_msg(storage)? {
            return Ok(None);
        }
        Ok(Some(Self::load(storage, id)?))
    }

    pub fn save(&self, storage: &mut dyn Storage, id: &Uint128) -> StdResult<()> {
        AssemblyMsgData {
            assemblies: self.assemblies.clone(),
            msg: self.msg.clone(),
        }
        .save(storage, &id.to_be_bytes())?;

        AssemblyMsgDescription(self.name.clone()).save(storage, &id.to_be_bytes())?;

        Ok(())
    }

    pub fn data(storage: &dyn Storage, id: &Uint128) -> StdResult<AssemblyMsgData> {
        AssemblyMsgData::load(storage, &id.to_be_bytes())
    }

    pub fn save_data(
        storage: &mut dyn Storage,
        id: &Uint128,
        data: AssemblyMsgData,
    ) -> StdResult<()> {
        data.save(storage, &id.to_be_bytes())
    }

    pub fn description(storage: &dyn Storage, id: &Uint128) -> StdResult<String> {
        Ok(AssemblyMsgDescription::load(storage, &id.to_be_bytes())?.0)
    }

    pub fn save_description(
        storage: &mut dyn Storage,
        id: &Uint128,
        desc: String,
    ) -> StdResult<()> {
        AssemblyMsgDescription(desc).save(storage, &id.to_be_bytes())
    }
}

#[cfg(feature = "governance-impl")]
#[cw_serde]
pub struct AssemblyMsgData {
    pub assemblies: Vec<Uint128>,
    pub msg: FlexibleMsg,
}

#[cfg(feature = "governance-impl")]
impl BucketStorage for AssemblyMsgData {
    const NAMESPACE: &'static [u8] = b"assembly_msg_data-";
}

#[cfg(feature = "governance-impl")]
#[cw_serde]struct AssemblyMsgDescription(pub String);

#[cfg(feature = "governance-impl")]
impl BucketStorage for AssemblyMsgDescription {
    const NAMESPACE: &'static [u8] = b"assembly_msg_description-";
}
