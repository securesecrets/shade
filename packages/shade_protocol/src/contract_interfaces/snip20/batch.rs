use crate::c_std::{Binary, Addr};

use crate::serde::{Deserialize, Serialize};
use crate::c_std::Uint128;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct TransferAction {
    pub recipient: Addr,
    pub amount: Uint128,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct SendAction {
    pub recipient: Addr,
    pub recipient_code_hash: Option<String>,
    pub amount: Uint128,
    pub msg: Option<Binary>,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct TransferFromAction {
    pub owner: Addr,
    pub recipient: Addr,
    pub amount: Uint128,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct SendFromAction {
    pub owner: Addr,
    pub recipient: Addr,
    pub recipient_code_hash: Option<String>,
    pub amount: Uint128,
    pub msg: Option<Binary>,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct MintAction {
    pub recipient: Addr,
    pub amount: Uint128,
    pub memo: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub struct BurnFromAction {
    pub owner: Addr,
    pub amount: Uint128,
    pub memo: Option<String>,
}