use cosmwasm_std::{Addr, Storage, Uint128};
use cosmwasm_storage::{
    bucket,
    bucket_read,
    singleton,
    singleton_read,
    Bucket,
    ReadonlyBucket,
    ReadonlySingleton,
    Singleton,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use shade_protocol::{utils::liquidity_book::tokens::TokenType, Contract};
use shadeswap_shared::{core::TokenAmount, router::Hop};

pub static CONFIG: &[u8] = b"config";
pub static REGISTERED_TOKENS: &[u8] = b"registered_tokens";
pub static REGISTERED_TOKENS_LIST: &[u8] = b"registered_tokens_list";
pub const EPHEMERAL_STORAGE_KEY: &[u8] = b"ephemeral_storage";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub viewing_key: String,
    pub admin_auth: Contract,
    pub airdrop_address: Option<Contract>,
}

pub fn config_w(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, CONFIG)
}

pub fn config_r(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, CONFIG)
}

// { addr: code_hash }
pub fn registered_tokens_w(storage: &mut dyn Storage) -> Bucket<String> {
    bucket(storage, REGISTERED_TOKENS)
}

pub fn registered_tokens_r(storage: &dyn Storage) -> ReadonlyBucket<String> {
    bucket_read(storage, REGISTERED_TOKENS)
}

pub fn registered_tokens_list_w(storage: &mut dyn Storage) -> Singleton<Vec<Addr>> {
    singleton(storage, REGISTERED_TOKENS_LIST)
}

pub fn registered_tokens_list_r(storage: &dyn Storage) -> ReadonlySingleton<Vec<Addr>> {
    singleton_read(storage, REGISTERED_TOKENS_LIST)
}

pub fn epheral_storage_w(storage: &mut dyn Storage) -> Singleton<CurrentSwapInfo> {
    singleton(storage, EPHEMERAL_STORAGE_KEY)
}

pub fn epheral_storage_r(storage: &dyn Storage) -> ReadonlySingleton<CurrentSwapInfo> {
    singleton_read(storage, EPHEMERAL_STORAGE_KEY)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurrentSwapInfo {
    pub(crate) amount: TokenAmount,
    pub amount_out_min: Option<Uint128>,
    pub path: Vec<Hop>,
    pub recipient: Addr,
    pub current_index: u32,
    //The next token that will be in the hop
    pub next_token_in: TokenType,
}
