use crate::{
    contract_interfaces::dao::adapter,
    utils::{asset::Contract, generic_response::ResponseStatus},
};
use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use serde::{Deserialize, Serialize};

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

//TODO: move accounts to treasury manager
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Holder {
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
    Adapter(adapter::SubHandleMsg),
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
    Adapter(adapter::HandleAnswer),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Assets {},
    Allocations { asset: HumanAddr },
    PendingAllowance { asset: HumanAddr },
    Holders { },
    Holder { holder: HumanAddr },
    Balance { asset: HumanAddr, holder: HumanAddr },
    Unbonding { asset: HumanAddr, holder: HumanAddr },
    Unbondable { asset: HumanAddr, holder: HumanAddr },
    Claimable { asset: HumanAddr, holder: HumanAddr },
    Adapter(adapter::SubQueryMsg),
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
    Holder { holder: Holder },
    Adapter(adapter::QueryAnswer),
}
