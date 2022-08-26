use crate::{
    c_std::{Addr, Binary, Uint128},
    contract_interfaces::dao::manager,
    utils::{
        asset::{Contract, RawContract},
        generic_response::ResponseStatus,
    },
};

use crate::utils::{ExecuteCallback, InstantiateCallback, Query};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct Config {
    pub admin_auth: Contract,
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
    pub admin_auth: RawContract,
    pub viewing_key: String,
    pub treasury: String,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    Receive {
        sender: String,
        from: String,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<Binary>,
    },
    UpdateConfig {
        config: Config,
    },
    RegisterAsset {
        contract: RawContract,
    },
    Allocate {
        asset: String,
        allocation: Allocation,
    },
    AddHolder {
        holder: String,
    },
    RemoveHolder {
        holder: String,
    },
    Manager(manager::SubExecuteMsg),
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteAnswer {
    Init {
        status: ResponseStatus,
        address: String,
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
    Manager(manager::ExecuteAnswer),
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Assets {},
    Allocations { asset: String },
    PendingAllowance { asset: String },
    Holders {},
    Holding { holder: String },
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
    Holding { holding: Holding },
}
