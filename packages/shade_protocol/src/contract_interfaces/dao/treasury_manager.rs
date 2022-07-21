use crate::{
    contract_interfaces::dao::adapter,
    utils::{asset::Contract, generic_response::ResponseStatus},
};
use crate::c_std::{Binary, Addr, Uint128};

use crate::utils::{ExecuteCallback, InstantiateCallback, Query};
use cosmwasm_schema::{cw_serde};

pub mod storage {
    use secret_storage_plus::{Map, Item};
    use cosmwasm_std::Addr;
    use crate::contract_interfaces::snip20::helpers::Snip20Asset;

    pub const CONFIG: Item<super::Config> = Item::new("config");
    pub const VIEWING_KEY: Item<String> = Item::new("viewing_key");
    pub const SELF_ADDRESS: Item<Addr> = Item::new("self_address");

    pub const ASSET_LIST: Item<Vec<Addr>> = Item::new("asset_list");
    pub const ASSETS: Map<Addr, Snip20Asset> = Map::new("assets");

    pub const ALLOCATIONS: Map<Addr, Vec<super::AllocationMeta>> = Map::new("allocations");
    pub const HOLDERS: Item<Vec<super::Addr>> = Item::new("holders");
    pub const HOLDING: Map<Addr, super::Holding> = Map::new("holding");
    //pub const UNBONDINGS: Map<Addr, Vec<super::Unbonding>> = Map::new("unbondings");
}

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub treasury: Addr,
}

#[cw_serde]
pub struct Balance {
    pub token: Addr,
    pub amount: Uint128,
}

#[cw_serde]
pub enum Status {
    Active,
    Disabled,
    Closed,
    Transferred,
}

//TODO: move accounts to treasury manager
#[cw_serde]
pub struct Holding {
    pub balances: Vec<Balance>,
    pub unbondings: Vec<Balance>,
    //pub claimable: Vec<Balance>,
    pub status: Status,
}

#[cw_serde]
pub struct Unbonding {
    pub holder: Addr,
    pub amount: Uint128,
}

#[cw_serde]
pub struct Allocation {
    pub nick: Option<String>,
    pub contract: Contract,
    pub alloc_type: AllocationType,
    pub amount: Uint128,
    pub tolerance: Uint128,
}

#[cw_serde]
pub enum AllocationType {
    // amount becomes percent * 10^18
    Portion,
    Amount,
}

#[cw_serde]
pub struct AllocationMeta {
    pub nick: Option<String>,
    pub contract: Contract,
    pub alloc_type: AllocationType,
    pub amount: Uint128,
    pub tolerance: Uint128,
    pub balance: Uint128,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<Addr>,
    pub viewing_key: String,
    pub treasury: Addr,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive {
        sender: Addr,
        from: Addr,
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
        asset: Addr,
        allocation: Allocation,
    },
    AddHolder {
        holder: Addr,
    },
    RemoveHolder {
        holder: Addr,
    },
    Manager(manager::SubHandleMsg),
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum HandleAnswer {
    Init {
        status: ResponseStatus,
        address: Addr,
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

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Assets {},
    Allocations { asset: Addr },
    PendingAllowance { asset: Addr },
    Holders { },
    Holding { holder: Addr },
    /*
    Balance { asset: Addr, holder: Addr },
    Unbonding { asset: Addr, holder: Addr },
    Unbondable { asset: Addr, holder: Addr },
    Claimable { asset: Addr, holder: Addr },
    */
    Manager(manager::SubQueryMsg),
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Config { config: Config },
    Assets { assets: Vec<Addr> },
    Allocations { allocations: Vec<AllocationMeta> },
    PendingAllowance { amount: Uint128 },
    Holders { holders: Vec<Addr> },
    Holding { holding: Holding},
    Manager(manager::QueryAnswer),
}
