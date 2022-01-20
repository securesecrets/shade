use cosmwasm_std::{HumanAddr, Storage, Uint128};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use shade_protocol::{snip20::Snip20Asset, treasury};

pub static CONFIG_KEY: &[u8] = b"config";
pub static ASSET_KEY: &[u8] = b"assets";
pub static VIEWING_KEY: &[u8] = b"viewing_key";
pub static SELF_ADDRESS: &[u8] = b"self_address";
pub static ALLOCATIONS: &[u8] = b"allocations";
pub static ALLOWANCE_REFRESH: &[u8] = b"allowance_refresh";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, treasury::Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, treasury::Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn assets_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Snip20Asset> {
    bucket_read(ASSET_KEY, storage)
}

pub fn assets_w<S: Storage>(storage: &mut S) -> Bucket<S, Snip20Asset> {
    bucket(ASSET_KEY, storage)
}

pub fn viewing_key_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, String> {
    singleton_read(storage, VIEWING_KEY)
}

pub fn viewing_key_w<S: Storage>(storage: &mut S) -> Singleton<S, String> {
    singleton(storage, VIEWING_KEY)
}

pub fn self_address_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, HumanAddr> {
    singleton_read(storage, SELF_ADDRESS)
}

pub fn self_address_w<S: Storage>(storage: &mut S) -> Singleton<S, HumanAddr> {
    singleton(storage, SELF_ADDRESS)
}

pub fn allocations_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Vec<treasury::Allocation>> {
    bucket_read(ALLOCATIONS, storage)
}

pub fn allocations_w<S: Storage>(storage: &mut S) -> Bucket<S, Vec<treasury::Allocation>> {
    bucket(ALLOCATIONS, storage)
}

pub fn last_allowance_refresh_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, String> {
    singleton_read(storage, ALLOWANCE_REFRESH)
}

pub fn last_allowance_refresh_w<S: Storage>(storage: &mut S) -> Singleton<S, String> {
    singleton(storage, ALLOWANCE_REFRESH)
}

/*
pub fn reserves_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Uint128> {
    bucket_read(RESERVES, storage)
}

pub fn reserves_w<S: Storage>(storage: &mut S) -> Bucket<S, Uint128> {
    bucket(RESERVES, storage)
}
*/
