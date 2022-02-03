use crate::snip20::Snip20Asset;
use cosmwasm_std::{HumanAddr, Uint128};
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use serde::{Deserialize, Serialize};

use crate::utils::asset::Contract;
use crate::utils::generic_response::ResponseStatus;
#[cfg(test)]
use secretcli::secretcli::{TestHandle, TestInit, TestQuery};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Pair {
    // secretswap_pair contract
    pub pair: Contract,
    // non-sscrt asset, other asset on pair should be sscrt
    pub asset: Snip20Asset,
}

/*
pub struct SiennaPair {
    // secretswap_pair contract
    pub pair: Contract,
    // non-sscrt asset, other asset on pair should be sscrt
    pub asset: Snip20Asset,
}
*/

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct IndexElement {
    pub symbol: String,
    //TODO: Decimal, when better implementation is available
    pub weight: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleConfig {
    pub admin: HumanAddr,
    pub band: Contract,
    pub sscrt: Contract,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub band: Contract,
    pub sscrt: Contract,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cfg(test)]
impl TestInit for InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        admin: Option<HumanAddr>,
        band: Option<Contract>,
    },
    // Register Secret Swap or Sienna Pair (should be */sSCRT or sSCRT/*)
    RegisterPair {
        pair: Contract,
    },
    // Unregister Secret Swap Pair (opposite action to RegisterSswapPair)
    UnregisterPair {
        pair: Contract,
    },

    RegisterIndex {
        symbol: String,
        basket: Vec<IndexElement>,
    },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cfg(test)]
impl TestHandle for HandleMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    UpdateConfig { status: ResponseStatus },

    RegisterPair {
        status: ResponseStatus,
        dex: String,
        symbol: String,
    },
    UnregisterPair { status: ResponseStatus },
    RegisterIndex { status: ResponseStatus },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Price { symbol: String },
    Prices { symbols: Vec<String> },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cfg(test)]
impl TestQuery<QueryAnswer> for QueryMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: OracleConfig },
}
