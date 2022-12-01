use cosmwasm_std::{Addr, Decimal, Uint128};

use crate::{
    contract_interfaces::dao::adapter,
    utils::{
        asset::Contract,
        generic_response::ResponseStatus,
        storage::plus::{Item, ItemStorage},
        ExecuteCallback,
        InstantiateCallback,
        Query,
    },
};
use super::cycles::{ArbPair, Derivative};
use cosmwasm_schema::cw_serde;

/// Used to determine which direction to arb
#[cw_serde]
pub enum Direction {
    Stake,
    Unbond,
}

#[cw_serde]
pub struct TradingFees {
    pub dex_fee: Decimal,
    pub stake_fee: Decimal,
    pub unbond_fee: Decimal,
}

#[cw_serde]
pub struct Config {
    pub shade_admin_addr: Contract,
    pub derivative: Derivative,
    pub trading_fees: TradingFees,
    pub max_arb_amount: Uint128,
    // TODO: maybe we don't need the period?
    /// Number of seconds between each scheduled arb execution
    pub arb_period: u32,
}

impl ItemStorage for Config {
    const ITEM: Item<'static, Config> = Item::new("item_config");
}

#[cw_serde]
pub struct SelfAddr(pub Addr);

impl ItemStorage for SelfAddr {
    const ITEM: Item<'static, SelfAddr> = Item::new("item_self_addr");
}

#[cw_serde]
pub struct DexPairs(pub Vec<ArbPair>);

impl ItemStorage for DexPairs {
    const ITEM: Item<'static, DexPairs> = Item::new("item_dex_pair");
}

#[cw_serde]
pub struct ViewingKey(pub String);

impl ItemStorage for ViewingKey {
    const ITEM: Item<'static, ViewingKey> = Item::new("item_viewing_key");
}

#[cw_serde]
pub struct Rollover(pub Uint128);

impl ItemStorage for Rollover {
    const ITEM: Item<'static, Rollover> = Item::new("item_rollover");
}

#[cw_serde]
pub struct InstantiateMsg {
    pub shade_admin_addr: Contract,
    pub derivative: Derivative,
    pub trading_fees: TradingFees,
    pub dex_pairs: Vec<ArbPair>,
    pub max_arb_amount: Uint128,
    pub arb_period: u32,
    pub viewing_key: String,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    Arbitrage {
        index: usize,
    }, 
    ArbAllPairs {},
    UpdateConfig {
        // Sender must be authorized on new contract as well
        shade_admin_addr: Option<Contract>,
        // Changing the derivative erases the saved dex pairs for data validation reasons
        derivative: Option<Derivative>,
        trading_fees: Option<TradingFees>,
        max_arb_amount: Option<Uint128>,
        arb_period: Option<u32>,
    },
    SetDexPairs {
        pairs: Vec<ArbPair>,
    },
    AddPair {
        pair: ArbPair,
    },
    SetPair {
        pair: ArbPair,
        // Defaults to the first index
        index: Option<usize>,
    },
    RemovePair {
        index: usize,
    },
    // TODO - SetViewingKey???
    Adapter(adapter::SubExecuteMsg),
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteAnswer {
    Arbitrage {
        status: ResponseStatus,
    },
    ArbAllPairs {
        statuses: Vec<ResponseStatus>,
    },
    SetDexPairs {
        status: ResponseStatus,
    },
    SetPair {
        status: ResponseStatus,
    },
    AddPair {
        status: ResponseStatus,
    },
    RemovePair {
        status: ResponseStatus,
    },
    UpdateConfig {
        status: ResponseStatus,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    DexPairs {},
    CurrentRollover {},
    IsProfitable {
        // Defaults to the first index
        index: Option<usize>,
        max_swap: Option<Uint128>,
    },
    IsAnyPairProfitable {
        max_swap: Option<Uint128>,
    },
    Adapter(adapter::SubQueryMsg),
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Config {
        config: Config
    },
    DexPairs {
        dex_pairs: Vec<ArbPair>,
    },
    CurrentRollover {
        rollover: Uint128,
    },
    IsProfitable {
        is_profitable: bool,
        // TODO: turn into struct
        swap_amounts: Option<(Uint128, Uint128, Uint128)>,
        direction: Option<Direction>,
    },
    IsAnyPairProfitable {
        is_profitable: Vec<bool>,
        swap_amounts: Vec<Option<(Uint128, Uint128, Uint128)>>,
        direction: Vec<Option<Direction>>,
    },
}

