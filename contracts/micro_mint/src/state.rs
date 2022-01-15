use cosmwasm_std::{Storage, Uint128};
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
use shade_protocol::{
    micro_mint::{Config, MintLimit, SupportedAsset},
    snip20::Snip20Asset,
};

pub static CONFIG_KEY: &[u8] = b"config";
pub static MINT_LIMIT: &[u8] = b"mint_limit";
pub static NATIVE_ASSET: &[u8] = b"native_asset";
pub static ASSET_PEG: &[u8] = b"asset_peg";
pub static ASSET_KEY: &[u8] = b"assets";
pub static ASSET_LIST_KEY: &[u8] = b"asset_list";
pub static BURN_COUNT_KEY: &[u8] = b"burn_count";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn limit_w<S: Storage>(storage: &mut S) -> Singleton<S, MintLimit> {
    singleton(storage, MINT_LIMIT)
}

pub fn limit_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, MintLimit> {
    singleton_read(storage, MINT_LIMIT)
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

pub fn asset_list_w<S: Storage>(storage: &mut S) -> Singleton<S, Vec<String>> {
    singleton(storage, ASSET_LIST_KEY)
}

pub fn asset_list_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Vec<String>> {
    singleton_read(storage, ASSET_LIST_KEY)
}

pub fn assets_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, SupportedAsset> {
    bucket_read(ASSET_KEY, storage)
}

pub fn assets_w<S: Storage>(storage: &mut S) -> Bucket<S, SupportedAsset> {
    bucket(ASSET_KEY, storage)
}

pub fn total_burned_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Uint128> {
    bucket_read(BURN_COUNT_KEY, storage)
}

pub fn total_burned_w<S: Storage>(storage: &mut S) -> Bucket<S, Uint128> {
    bucket(BURN_COUNT_KEY, storage)
}
