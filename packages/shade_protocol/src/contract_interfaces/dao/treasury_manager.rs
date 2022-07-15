use crate::{
    contract_interfaces::dao::manager,
    utils::{asset::Contract, generic_response::ResponseStatus},
};
use crate::c_std::{Binary, HumanAddr, Uint128};
use crate::schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use crate::serde::{Deserialize, Serialize};

pub mod storage {
    use secret_storage_plus::{Map, Item};
    use cosmwasm_std::HumanAddr;
    use crate::contract_interfaces::snip20::helpers::Snip20Asset;

    pub const CONFIG: Item<super::Config> = Item::new("config");
    pub const VIEWING_KEY: Item<String> = Item::new("viewing_key");
    pub const SELF_ADDRESS: Item<HumanAddr> = Item::new("self_address");

    pub const ASSET_LIST: Item<Vec<HumanAddr>> = Item::new("asset_list");
    pub const ASSETS: Map<HumanAddr, Snip20Asset> = Map::new("assets");

    pub const ALLOCATIONS: Map<HumanAddr, Vec<super::AllocationMeta>> = Map::new("allocations");
    pub const HOLDERS: Item<Vec<super::HumanAddr>> = Item::new("holders");
    pub const HOLDING: Map<HumanAddr, super::Holding> = Map::new("holding");
    //pub const UNBONDINGS: Map<HumanAddr, Vec<super::Unbonding>> = Map::new("unbondings");
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub admin: HumanAddr,
    pub treasury: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Balance {
    pub token: HumanAddr,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Active,
    Disabled,
    Closed,
    Transferred,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Holding {
    pub balances: Vec<Balance>,
    pub unbondings: Vec<Balance>,
    //pub claimable: Vec<Balance>,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Unbonding {
    pub holder: HumanAddr,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Allocation {
    pub nick: Option<String>,
    pub contract: Contract,
    pub alloc_type: AllocationType,
    pub amount: Uint128,
    pub tolerance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AllocationType {
    // amount becomes percent * 10^18
    Portion,
    Amount,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AllocationMeta {
    pub nick: Option<String>,
    pub contract: Contract,
    pub alloc_type: AllocationType,
    pub amount: Uint128,
    pub tolerance: Uint128,
    pub balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub viewing_key: String,
    pub treasury: HumanAddr,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
    UpdateConfig {
        config: Config,
    },
    RegisterAsset {
        contract: Contract,
    },
    Allocate {
        asset: HumanAddr,
        allocation: Allocation,
    },
    AddHolder {
        holder: HumanAddr,
    },
    RemoveHolder {
        holder: HumanAddr,
    },
    Manager(manager::SubHandleMsg),
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    Init {
        status: ResponseStatus,
        address: HumanAddr,
    },
    Receive {
        status: ResponseStatus,
    },
    UpdateConfig {
        status: ResponseStatus,
    },
    RegisterAsset {
        status: ResponseStatus,
    },
    Allocate {
        status: ResponseStatus,
    },
    AddHolder {
        status: ResponseStatus,
    },
    RemoveHolder {
        status: ResponseStatus,
    },
    Manager(manager::HandleAnswer),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Assets {},
    Allocations { asset: HumanAddr },
    PendingAllowance { asset: HumanAddr },
    Holders { },
    Holding { holder: HumanAddr },
    /*
    Balance { asset: HumanAddr, holder: HumanAddr },
    Unbonding { asset: HumanAddr, holder: HumanAddr },
    Unbondable { asset: HumanAddr, holder: HumanAddr },
    Claimable { asset: HumanAddr, holder: HumanAddr },
    */
    Manager(manager::SubQueryMsg),
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: Config },
    Assets { assets: Vec<HumanAddr> },
    Allocations { allocations: Vec<AllocationMeta> },
    PendingAllowance { amount: Uint128 },
    Holders { holders: Vec<HumanAddr> },
    Holding { holding: Holding},
    Manager(manager::QueryAnswer),
}
