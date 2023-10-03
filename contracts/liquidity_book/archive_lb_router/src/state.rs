use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, CanonicalAddr, ContractInfo, Uint128};

use libraries::tokens::TokenType;
use secret_toolkit::storage::Item;

use crate::msg::{Hop, TokenAmount};

pub static CONFIG: Item<Config> = Item::new(b"config");
pub static EPHEMERAL_STORAGE: Item<CurrentSwapInfo> = Item::new(b"ephemeral_storage");

#[cw_serde]
pub struct Config {
    pub factory: ContractInfo,
    pub admins: Vec<CanonicalAddr>,
    pub viewing_key: String,
}

#[cw_serde]
pub struct CurrentSwapInfo {
    pub(crate) amount: TokenAmount,
    pub amount_out_min: Option<Uint128>,
    pub path: Vec<Hop>,
    pub recipient: Addr,
    pub current_index: u32,
    //The next token that will be in the hop
    pub next_token_in: TokenType,
}
