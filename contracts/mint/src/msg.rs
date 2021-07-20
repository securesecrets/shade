use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{HumanAddr, CosmosMsg, Uint128, Binary};
use crate::state::Asset;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub silk_contract: HumanAddr,
    pub silk_contract_code_hash: String,
    pub oracle_contract: HumanAddr,
    pub oracle_contract_code_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        owner: Option<HumanAddr>,
        silk_contract: Option<HumanAddr>,
        silk_contract_code_hash: Option<String>,
        oracle_contract: Option<HumanAddr>,
        oracle_contract_code_hash: Option<String>,
    },
    RegisterAsset {
        contract: HumanAddr,
        code_hash: String,
    },
    UpdateAsset {
        asset: HumanAddr,
        contract: HumanAddr,
        code_hash: String,
    },
    Receive {
        sender: HumanAddr,
        from: HumanAddr,
        amount: Uint128,
        memo: Option<Binary>,
        msg: Option<CosmosMsg>,
    },
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    UpdateConfig { status: ResponseStatus},
    RegisterAsset { status: ResponseStatus},
    UpdateAsset { status: ResponseStatus},
    Burn { status: ResponseStatus, mint_amount: Uint128 }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetSupportedAssets {},
    GetAsset {
        contract: String,
    },
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    SupportedAssets { assets: Vec<String>, },
    Asset { asset: Asset }
}

// Contract interactions
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleCall {
    pub contract: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Failure,
}