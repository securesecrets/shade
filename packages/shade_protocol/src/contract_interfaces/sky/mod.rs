#[cfg(feature = "sky-utils")]
pub mod cycles;

use crate::{
    contract_interfaces::{dao::adapter, sky::cycles::Cycle},
    utils::{
        asset::Contract,
        storage::plus::ItemStorage,
        ExecuteCallback,
        InstantiateCallback,
        Query,
    },
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, Uint128};
use secret_storage_plus::Item;

#[cw_serde]
pub struct Config {
    pub shade_admin: Contract,
    pub shd_token: Contract,
    pub silk_token: Contract,
    pub sscrt_token: Contract,
    pub treasury: Contract,
    pub payback_rate: Decimal,
}

impl ItemStorage for Config {
    const ITEM: Item<'static, Config> = Item::new("item_config");
}

#[cw_serde]
pub struct ViewingKeys(pub String);

impl ItemStorage for ViewingKeys {
    const ITEM: Item<'static, ViewingKeys> = Item::new("item_view_keys");
}

#[cw_serde]
pub struct SelfAddr(pub Addr);

impl ItemStorage for SelfAddr {
    const ITEM: Item<'static, SelfAddr> = Item::new("item_self_addr");
}

#[cw_serde]
pub struct Cycles(pub Vec<Cycle>);

impl ItemStorage for Cycles {
    const ITEM: Item<'static, Cycles> = Item::new("item_cycles");
}

#[cw_serde]
pub struct InstantiateMsg {
    pub shade_admin: Contract,
    pub shd_token: Contract,
    pub silk_token: Contract,
    pub sscrt_token: Contract,
    pub treasury: Contract,
    pub viewing_key: String,
    pub payback_rate: Decimal,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        shade_admin: Option<Contract>,
        shd_token: Option<Contract>,
        silk_token: Option<Contract>,
        sscrt_token: Option<Contract>,
        treasury: Option<Contract>,
        payback_rate: Option<Decimal>,
        padding: Option<String>,
    },
    SetCycles {
        cycles: Vec<Cycle>,
        padding: Option<String>,
    },
    AppendCycles {
        cycle: Vec<Cycle>,
        padding: Option<String>,
    },
    UpdateCycle {
        cycle: Cycle,
        index: Uint128,
        padding: Option<String>,
    },
    RemoveCycle {
        index: Uint128,
        padding: Option<String>,
    },
    ArbCycle {
        amount: Uint128,
        index: Uint128,
        padding: Option<String>,
    },
    ArbAllCycles {
        amount: Uint128,
        padding: Option<String>,
    },
    Adapter(adapter::SubExecuteMsg),
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteAnswer {
    Init {
        status: bool,
    },
    UpdateConfig {
        status: bool,
    },
    SetCycles {
        status: bool,
    },
    AppendCycles {
        status: bool,
    },
    UpdateCycle {
        status: bool,
    },
    RemoveCycle {
        status: bool,
    },
    ExecuteArbCycle {
        status: bool,
        swap_amounts: Vec<Uint128>,
        payback_amount: Uint128,
    },
    ArbAllCycles {
        status: bool,
        payback_amount: Uint128,
    },
}

#[cw_serde]
pub enum QueryMsg {
    GetConfig {},
    Balance {},
    GetCycles {},
    IsCycleProfitable { amount: Uint128, index: Uint128 },
    IsAnyCycleProfitable { amount: Uint128 },
    Adapter(adapter::SubQueryMsg),
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Config {
        config: Config,
    },
    Balance {
        shd_bal: Uint128,
        silk_bal: Uint128, //should be zero or close to
        sscrt_bal: Uint128,
    },
    GetCycles {
        cycles: Vec<Cycle>,
    },
    IsCycleProfitable {
        is_profitable: bool,
        direction: Cycle,
        swap_amounts: Vec<Uint128>,
        profit: Uint128,
    },
    IsAnyCycleProfitable {
        is_profitable: Vec<bool>,
        direction: Vec<Cycle>,
        swap_amounts: Vec<Vec<Uint128>>,
        profit: Vec<Uint128>,
    },
}
