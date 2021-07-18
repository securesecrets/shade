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
        owner: HumanAddr,
        silk_contract: HumanAddr,
        silk_contract_code_hash: String,
        oracle_contract: HumanAddr,
        oracle_contract_code_hash: String,
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetNativeBurned {},
    GetSupportedAssets {},
    GetAsset {
        contract: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SupportedAssetsResponse {
    pub assets: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AssetResponse {
    pub asset: Asset,
}

// Contract interactions
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OracleCall {
    pub contract: HumanAddr,
}