use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Uint128};
use crate::msg_traits::{Init, Handle, Query};
use crate::asset::Contract;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleConfig {
    // Band protocol contract address
    pub band: Contract,
    pub owner: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub band: Contract,
    pub admin: Option<HumanAddr>,
}

impl Init<'_> for InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
}

impl Handle<'_> for HandleMsg{}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetScrtPrice {},
    GetPrice { symbol: String },
    GetConfig {},
    GetReferenceData {
        base_symbol: String,
        quote_symbol: String,
    },
}

impl Query for QueryMsg {}

/*
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryExtMsg {
    GetReferenceData {
        base_symbol: String,
        quote_symbol: String,
    }
}
impl Query for QueryExtMsg {}
*/

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PriceResponse {
    pub price: Uint128,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Price { price: PriceResponse },
    Config { config: OracleConfig },
}
