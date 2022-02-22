use crate::snip20::InitialBalance;
use crate::utils::generic_response::ResponseStatus;
use cosmwasm_std::{Binary, HumanAddr};
use schemars::JsonSchema;
use secret_toolkit::utils::{HandleCallback, InitCallback, Query};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: HumanAddr,
    pub snip20_id: u64,
    pub snip20_code_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Snip20InitHistory {
    pub label: String,
    pub balances: Option<Vec<InitialBalance>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Snip20ContractInfo {
    pub label: String,
    pub admin: Option<HumanAddr>,
    pub prng_seed: Binary,
    pub initial_balances: Option<Vec<InitialBalance>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub snip20_id: u64,
    pub snip20_code_hash: String,
    pub shade: Snip20ContractInfo,
}

impl InitCallback for InitMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    SetAdmin {
        admin: HumanAddr,
    },

    InitSilk {
        silk: Snip20ContractInfo,
        ticker: String,
        decimals: u8,
    },
}

impl HandleCallback for HandleMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    SetAdmin { status: ResponseStatus },
    InitSilk { status: ResponseStatus },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Contracts {},
    Config {},
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    Contracts {
        shade: Snip20InitHistory,
        silk: Option<Snip20InitHistory>,
    },

    Config {
        config: Config,
    },
}
