use crate::{
    c_std::{Addr, StdResult, Storage},
    contract_interfaces::governance::stored_id::ID,
    utils::flexible_msg::FlexibleMsg,
};

use cosmwasm_schema::cw_serde;
use secret_storage_plus::Map;

#[cfg(feature = "governance-impl")]
use crate::utils::storage::plus::MapStorage;

#[cw_serde]
pub struct Assembly {
    // Readable name
    pub name: String,
    // Description of the assembly, preferably in base64
    pub metadata: String,
    // List of members in assembly
    pub members: Vec<Addr>,
    // Selected profile
    pub profile: u16,
}

#[cfg(feature = "governance-impl")]
impl Assembly {
    pub fn load(storage: &dyn Storage, id: u16) -> StdResult<Self> {
        let desc = Self::description(storage, id)?;
        let data = Self::data(storage, id)?;

        Ok(Self {
            name: desc.name,
            metadata: desc.metadata,
            members: data.members,
            profile: data.profile,
        })
    }

    pub fn may_load(storage: &dyn Storage, id: u16) -> StdResult<Option<Self>> {
        if id > ID::assembly(storage)? {
            return Ok(None);
        }
        Ok(Some(Self::load(storage, id)?))
    }

    pub fn save(&self, storage: &mut dyn Storage, id: u16) -> StdResult<()> {
        AssemblyData {
            members: self.members.clone(),
            profile: self.profile,
        }
        .save(storage, id)?;

        AssemblyDescription {
            name: self.name.clone(),
            metadata: self.metadata.clone(),
        }
        .save(storage, id)?;

        Ok(())
    }

    pub fn data(storage: &dyn Storage, id: u16) -> StdResult<AssemblyData> {
        AssemblyData::load(storage, id)
    }

    pub fn save_data(storage: &mut dyn Storage, id: u16, data: AssemblyData) -> StdResult<()> {
        data.save(storage, id)
    }

    pub fn description(storage: &dyn Storage, id: u16) -> StdResult<AssemblyDescription> {
        AssemblyDescription::load(storage, id)
    }

    pub fn save_description(
        storage: &mut dyn Storage,
        id: u16,
        desc: AssemblyDescription,
    ) -> StdResult<()> {
        desc.save(storage, id)
    }
}

#[cfg(feature = "governance-impl")]
#[cw_serde]
pub struct AssemblyData {
    pub members: Vec<Addr>,
    pub profile: u16,
}

#[cfg(feature = "governance-impl")]
impl MapStorage<'static, u16> for AssemblyData {
    const MAP: Map<'static, u16, Self> = Map::new("assembly_data-");
}

#[cfg(feature = "governance-impl")]
#[cw_serde]
pub struct AssemblyDescription {
    pub name: String,
    pub metadata: String,
}

#[cfg(feature = "governance-impl")]
impl MapStorage<'static, u16> for AssemblyDescription {
    const MAP: Map<'static, u16, Self> = Map::new("assembly_description-");
}

#[cw_serde] // A generic msg is created at init, its a black msg where the variable is the start
pub struct AssemblyMsg {
    pub name: String,
    // Assemblies allowed to call this msg
    pub assemblies: Vec<u16>,
    // ExecuteMsg template
    pub msg: FlexibleMsg,
}

#[cfg(feature = "governance-impl")]
impl AssemblyMsg {
    pub fn load(storage: &dyn Storage, id: u16) -> StdResult<Self> {
        let desc = Self::description(storage, id)?;
        let data = Self::data(storage, id)?;

        Ok(Self {
            name: desc,
            assemblies: data.assemblies,
            msg: data.msg,
        })
    }

    pub fn may_load(storage: &dyn Storage, id: u16) -> StdResult<Option<Self>> {
        if id > ID::assembly_msg(storage)? {
            return Ok(None);
        }
        Ok(Some(Self::load(storage, id)?))
    }

    pub fn save(&self, storage: &mut dyn Storage, id: u16) -> StdResult<()> {
        AssemblyMsgData {
            assemblies: self.assemblies.clone(),
            msg: self.msg.clone(),
        }
        .save(storage, id)?;

        AssemblyMsgDescription(self.name.clone()).save(storage, id)?;

        Ok(())
    }

    pub fn data(storage: &dyn Storage, id: u16) -> StdResult<AssemblyMsgData> {
        AssemblyMsgData::load(storage, id)
    }

    pub fn save_data(storage: &mut dyn Storage, id: u16, data: AssemblyMsgData) -> StdResult<()> {
        data.save(storage, id)
    }

    pub fn description(storage: &dyn Storage, id: u16) -> StdResult<String> {
        Ok(AssemblyMsgDescription::load(storage, id)?.0)
    }

    pub fn save_description(storage: &mut dyn Storage, id: u16, desc: String) -> StdResult<()> {
        AssemblyMsgDescription(desc).save(storage, id)
    }
}

#[cfg(feature = "governance-impl")]
#[cw_serde]
pub struct AssemblyMsgData {
    pub assemblies: Vec<u16>,
    pub msg: FlexibleMsg,
}

#[cfg(feature = "governance-impl")]
impl MapStorage<'static, u16> for AssemblyMsgData {
    const MAP: Map<'static, u16, Self> = Map::new("assembly_msg_data-");
}

#[cfg(feature = "governance-impl")]
#[cw_serde]
struct AssemblyMsgDescription(pub String);

#[cfg(feature = "governance-impl")]
impl MapStorage<'static, u16> for AssemblyMsgDescription {
    const MAP: Map<'static, u16, Self> = Map::new("assembly_msg_description-");
}
