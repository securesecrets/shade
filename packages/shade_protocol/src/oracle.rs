use cosmwasm_std::{HumanAddr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::{
    msg_traits::{Init, Handle, Query},
    asset::Contract,
    generic_response::ResponseStatus,
    snip20::Snip20Asset,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SswapPair {
    // secretswap_pair contract
    pub pair: Contract,
    // non-sscrt asset, other asset on pair should be sscrt
    pub asset: Snip20Asset,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleConfig {
    pub owner: HumanAddr,
    pub band: Contract,
    pub sscrt: Contract,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub band: Contract,
    pub sscrt: Contract,
}

impl Init<'_> for InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        owner: Option<HumanAddr>,
        band: Option<Contract>,
    },
    // Register Secret Swap Pair (should be */SCRT)
    RegisterSswapPair {
        pair: Contract,
    },
}

impl Handle<'_> for HandleMsg{}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    UpdateConfig { status: ResponseStatus},
    RegisterSswapPair { status: ResponseStatus},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetPrice { symbol: String },
    GetConfig {},
}

impl Query for QueryMsg {}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Config { config: OracleConfig },
}


