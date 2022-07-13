use crate::{
    contract_interfaces::dao::adapter,
    utils::{asset::Contract, generic_response::ResponseStatus},
};
use crate::c_std::{Binary, Addr, Uint128};

use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use crate::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub admin: Addr,
    pub treasury: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Balance {
    pub token: Addr,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Active,
    Disabled,
    Closed,
    Transferred,
}

//TODO: move accounts to treasury manager
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Holder {
    pub balances: Vec<Balance>,
    pub unbondings: Vec<Balance>,
    //pub claimable: Vec<Balance>,
    pub status: Status,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Unbonding {
    pub holder: Addr,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Allocation {
    pub nick: Option<String>,
    pub contract: Contract,
    pub alloc_type: AllocationType,
    pub amount: Uint128,
    pub tolerance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AllocationType {
    // amount becomes percent * 10^18
    Portion,
    Amount,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AllocationMeta {
    pub nick: Option<String>,
    pub contract: Contract,
    pub alloc_type: AllocationType,
    pub amount: Uint128,
    pub tolerance: Uint128,
    pub balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct InitMsg {
    pub admin: Option<Addr>,
    pub viewing_key: String,
    pub treasury: Addr,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
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
    Adapter(adapter::SubHandleMsg),
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
    Adapter(adapter::HandleAnswer),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Assets {},
    Allocations { asset: Addr },
    PendingAllowance { asset: Addr },
    Holders { },
    Holder { holder: Addr },
    Balance { asset: Addr, holder: Addr },
    Unbonding { asset: Addr, holder: Addr },
    Unbondable { asset: Addr, holder: Addr },
    Claimable { asset: Addr, holder: Addr },
    Adapter(adapter::SubQueryMsg),
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: Config },
    Assets { assets: Vec<Addr> },
    Allocations { allocations: Vec<AllocationMeta> },
    PendingAllowance { amount: Uint128 },
    Holders { holders: Vec<Addr> },
    Holder { holder: Holder },
    Adapter(adapter::QueryAnswer),
}
