use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{HumanAddr, Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, bucket, Bucket, bucket_read, ReadonlyBucket};

pub static CONFIG_KEY: &[u8] = b"config";
pub static NATIVE_COIN_KEY: &[u8] = b"native_coin";
pub static ASSET_KEY: &[u8] = b"assets";
pub static ASSET_LIST_KEY: &[u8] = b"asset_list";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: HumanAddr,
    pub silk_contract: HumanAddr,
    pub silk_contract_code_hash: String,
    pub oracle_contract: HumanAddr,
    pub oracle_contract_code_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NativeCoin {
    pub burned_tokens: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Asset {
    pub contract: HumanAddr,
    pub code_hash: String,
    pub burned_tokens: Uint128,
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn native_coin<S: Storage>(storage: &mut S) -> Singleton<S, NativeCoin> {
singleton(storage, NATIVE_COIN_KEY)
}

pub fn native_coin_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, NativeCoin> {
    singleton_read(storage, NATIVE_COIN_KEY)
}

pub fn asset_list<S: Storage>(storage: &mut S) -> Singleton<S, Vec<String>> {
    singleton(storage, ASSET_LIST_KEY)
}

pub fn asset_list_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, Vec<String>> {
    singleton_read(storage, ASSET_LIST_KEY)
}

pub fn assets_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, Asset> {
    bucket_read(ASSET_KEY, storage)
}

pub fn assets_w<S: Storage>(storage: &mut S) -> Bucket<S, Asset> {
    bucket(ASSET_KEY, storage)
}