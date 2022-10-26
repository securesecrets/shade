use crate::c_std::Uint128;
use crate::c_std::Addr;

use crate::utils::{ExecuteCallback, InstantiateCallback, Query};
use cosmwasm_schema::{cw_serde};

use crate::{
    contract_interfaces::{
        dex::dex::TradingPair,
    },
    utils::{asset::Contract, generic_response::ResponseStatus},
};

#[cw_serde]
pub struct IndexElement {
    pub symbol: String,
    pub weight: Uint128,
}

#[cw_serde]
pub struct OracleConfig {
    pub admin: Addr,
    pub band: Contract,
    pub sscrt: Contract,
}

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<Addr>,
    pub band: Contract,
    pub sscrt: Contract,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        admin: Option<Addr>,
        band: Option<Contract>,
    },
    // Register Secret Swap or Sienna Pair (should be */sSCRT or sSCRT/*)
    RegisterPair {
        pair: Contract,
    },
    // Unregister Secret Swap Pair (opposite action to RegisterSswapPair)
    UnregisterPair {
        symbol: String,
        pair: Contract,
    },

    RegisterIndex {
        symbol: String,
        basket: Vec<IndexElement>,
    },
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteAnswer {
    UpdateConfig {
        status: ResponseStatus,
    },

    RegisterPair {
        status: ResponseStatus,
        symbol: String,
        pair: TradingPair,
    },
    UnregisterPair {
        status: ResponseStatus,
    },
    RegisterIndex {
        status: ResponseStatus,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
    Price { symbol: String },
    Prices { symbols: Vec<String> },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryAnswer {
    Config { config: OracleConfig },
}
