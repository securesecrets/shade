use cosmwasm_schema::cw_serde;
use cosmwasm_std::CanonicalAddr;
use cosmwasm_std::Uint256;
use secret_toolkit::storage::{Item, Keymap};

pub static CONFIG: Item<Config> = Item::new(b"config");
/// Mapping from account to token id to account balance. key = (account, token id)
pub static BALANCES: Keymap<(CanonicalAddr, u32), Uint256> = Keymap::new(b"balances");
/// Mapping from token id to total supply. key = token id
pub static TOTAL_SUPPLY: Keymap<u32, Uint256> = Keymap::new(b"total_supply");
/// Mapping from account to spender approvals. key = (owner, spender)
pub static SPENDER_APPROVALS: Keymap<(CanonicalAddr, CanonicalAddr), bool> =
    Keymap::new(b"spender_approvals");

#[cw_serde]
pub struct Config {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub admin: CanonicalAddr,
    pub lb_pair: CanonicalAddr,
}
