use crate::c_std::{Binary, Addr};

use cosmwasm_schema::{cw_serde};
use crate::c_std::Uint128;

#[cw_serde]
pub struct TransferAction {
    pub recipient: Addr,
    pub amount: Uint128,
    pub memo: Option<String>,
}

#[cw_serde]
pub struct SendAction {
    pub recipient: Addr,
    pub recipient_code_hash: Option<String>,
    pub amount: Uint128,
    pub msg: Option<Binary>,
    pub memo: Option<String>,
}

#[cw_serde]
pub struct TransferFromAction {
    pub owner: Addr,
    pub recipient: Addr,
    pub amount: Uint128,
    pub memo: Option<String>,
}

#[cw_serde]
pub struct SendFromAction {
    pub owner: Addr,
    pub recipient: Addr,
    pub recipient_code_hash: Option<String>,
    pub amount: Uint128,
    pub msg: Option<Binary>,
    pub memo: Option<String>,
}

#[cw_serde]
pub struct MintAction {
    pub recipient: Addr,
    pub amount: Uint128,
    pub memo: Option<String>,
}

#[cw_serde]
pub struct BurnFromAction {
    pub owner: Addr,
    pub amount: Uint128,
    pub memo: Option<String>,
}