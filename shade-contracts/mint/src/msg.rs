use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, CosmosMsg};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub silk_contract: CanonicalAddr,
    pub silk_contract_code_hash: String,
    pub oracle_contract: CanonicalAddr,
    pub oracle_contract_code_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    UpdateConfig {
        owner: CanonicalAddr,
        silk_contract: CanonicalAddr,
        silk_contract_code_hash: String,
        oracle_contract: CanonicalAddr,
        oracle_contract_code_hash: String,
    },
    RegisterAsset {
        contract: CanonicalAddr,
        code_hash: String,
    },
    UpdateAsset {
        asset: CanonicalAddr,
        contract: CanonicalAddr,
        code_hash: String,
    },
    // ReceiveNative {
    //     amount: uint128
    // },
    Receive {
        sender: CanonicalAddr,
        from: CanonicalAddr,
        amount: uint128,
        msg: Option<CosmosMsg>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetMinted {},
    GetNativeBurned {},
    GetAssetBurned {
        contract: CanonicalAddr,
    },

    GetCount {},
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CountResponse {
    pub count: i32,
}

// Contract interactions
pub struct OracleCall {
    pub contract: CanonicalAddr,
}