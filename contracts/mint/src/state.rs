use cosmwasm_std::{Storage, Uint128};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use shade_protocol::{
    mint::{Config, SupportedAsset},
    snip20::Snip20Asset,
};

pub static CONFIG_KEY: &[u8] = b"config";
pub static LIMIT: &[u8] = b"mint_limit";
pub static LIMIT_REFRESH: &[u8] = b"limit_refresh";
pub static MINTED: &[u8] = b"minted";
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

/* Daily limit as (limit * total_supply) at the time of refresh
 */
pub fn limit_w<S: Storage>(storage: &mut S) -> Singleton<S, Uint128> {
    singleton(storage, LIMIT)
}

pub fn limit_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Uint128> {
    singleton_read(storage, LIMIT)
}

/* RFC-3339 datetime str, last time limit was refreshed
 */
pub fn limit_refresh_w<S: Storage>(storage: &mut S) -> Singleton<S, String> {
    singleton(storage, LIMIT_REFRESH)
}

pub fn limit_refresh_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, String> {
    singleton_read(storage, LIMIT_REFRESH)
}

/* Amount minted this cycle (daily)
 */
pub fn minted_w<S: Storage>(storage: &mut S) -> Singleton<S, Uint128> {
    singleton(storage, MINTED)
}

pub fn minted_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Uint128> {
    singleton_read(storage, MINTED)
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
