use cosmwasm_std::{Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, bucket, Bucket, bucket_read, ReadonlyBucket};
use shade_protocol::{
    micro_mint::MintConfig, 
    snip20::Snip20Asset,
};

pub static CONFIG_KEY: &[u8] = b"config";
pub static NATIVE_ASSET: &[u8] = b"native_asset";
pub static ASSET_PEG: &[u8] = b"asset_peg";
pub static ASSET_KEY: &[u8] = b"assets";
pub static ASSET_LIST_KEY: &[u8] = b"asset_list";
pub static BURN_COUNT_KEY: &[u8] = b"burn_count";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, MintConfig> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, MintConfig> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn native_asset_w<S: Storage>(storage: &mut S) -> Singleton<S, Snip20Asset> {
    singleton(storage, NATIVE_ASSET)
}

pub fn native_asset_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Snip20Asset> {
    singleton_read(storage, NATIVE_ASSET)
}

pub fn asset_peg_w<S: Storage>(storage: &mut S) -> Singleton<S, String> {
    singleton(storage, ASSET_PEG)
}

pub fn asset_peg_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, String> {
    singleton_read(storage, ASSET_PEG)
}

pub fn asset_list<S: Storage>(storage: &mut S) -> Singleton<S, Vec<String>> {
    singleton(storage, ASSET_LIST_KEY)
}

pub fn asset_list_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, Vec<String>> {
    singleton_read(storage, ASSET_LIST_KEY)
}

pub fn assets_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, Snip20Asset> {
    bucket_read(ASSET_KEY, storage)
}

pub fn assets_w<S: Storage>(storage: &mut S) -> Bucket<S, Snip20Asset> {
    bucket(ASSET_KEY, storage)
}

pub fn total_burned_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, Uint128> {
    bucket_read(BURN_COUNT_KEY, storage)
}

pub fn total_burned_w<S: Storage>(storage: &mut S) -> Bucket<S, Uint128> {
    bucket(BURN_COUNT_KEY, storage)
}
