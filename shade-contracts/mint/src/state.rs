use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use std::collections::HashMap;

pub static CONFIG_KEY: &[u8] = b"config";
pub static NATIVE_COIN_KEY: &[u8] = b"native_coin";
pub static ASSET_KEY: &[u8] = b"assets";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub silk_contract: CanonicalAddr,
    pub silk_contract_code_hash: String,
    pub oracle_contract: CanonicalAddr,
    pub oracle_contract_code_hash: String,
}

pub struct Native_Coin {
    pub burned_tokens: uint128,
}

pub struct Asset {
    pub contract: CanonicalAddr,
    pub code_hash: String,
    pub burned_tokens: uint128,
}

pub struct Assets {
    pub coins: HashMap<String, Asset>
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn native_coin<S: Storage>(storage: &mut S) -> Singleton<S, Native_Coin> {
singleton(storage, NATIVE_COIN_KEY)
}

pub fn native_coin_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, Native_Coin> {
    singleton_read(storage, NATIVE_COIN_KEY)
}

pub fn assets<S: Storage>(storage: &mut S) -> Singleton<S, Assets> {
singleton(storage, ASSET_KEY)
}

pub fn assets_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, Assets> {
    singleton_read(storage, ASSET_KEY)
}
