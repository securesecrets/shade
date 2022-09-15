use crate::c_std::Binary;

use crate::c_std::Uint128;
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct TransferAction {
    pub recipient: String,
    pub amount: Uint128,
    pub memo: Option<String>,
}

#[cw_serde]
pub struct SendAction {
    pub recipient: String,
    pub recipient_code_hash: Option<String>,
    pub amount: Uint128,
    pub msg: Option<Binary>,
    pub memo: Option<String>,
}

#[cw_serde]
pub struct TransferFromAction {
    pub owner: String,
    pub recipient: String,
    pub amount: Uint128,
    pub memo: Option<String>,
}

#[cw_serde]
pub struct SendFromAction {
    pub owner: String,
    pub recipient: String,
    pub recipient_code_hash: Option<String>,
    pub amount: Uint128,
    pub msg: Option<Binary>,
    pub memo: Option<String>,
}

#[cw_serde]
pub struct MintAction {
    pub recipient: String,
    pub amount: Uint128,
    pub memo: Option<String>,
}

#[cw_serde]
pub struct BurnFromAction {
    pub owner: String,
    pub amount: Uint128,
    pub memo: Option<String>,
}
