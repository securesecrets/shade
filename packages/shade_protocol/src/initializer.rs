use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::snip20::InitialBalance;
use cosmwasm_std::{HumanAddr, Binary};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitializerConfig {
    pub contracts: Vec<String>,
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
    pub snip20_id: u64,
    pub snip20_code_hash: String,
    pub shade: Snip20ContractInfo,
    pub silk: Snip20ContractInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Contracts {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ContractsAnswer {
    pub contracts: Vec<String>,
}