//! Types used in batch operations

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{Binary, Addr};

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct TransferAction {
    pub recipient: Addr,
    pub amount: Uint128,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct SendAction {
    pub recipient: Addr,
    pub recipient_code_hash: Option<String>,
    pub amount: Uint128,
    pub msg: Option<Binary>,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct TransferFromAction {
    pub owner: Addr,
    pub recipient: Addr,
    pub amount: Uint128,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct SendFromAction {
    pub owner: Addr,
    pub recipient: Addr,
    pub recipient_code_hash: Option<String>,
    pub amount: Uint128,
    pub msg: Option<Binary>,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct MintAction {
    pub recipient: Addr,
    pub amount: Uint128,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct BurnFromAction {
    pub owner: Addr,
    pub amount: Uint128,
    pub memo: Option<String>,
}
