use cosmwasm_std::{HumanAddr, Storage, Uint128};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use shade_protocol::{snip20::Snip20Asset, finance_manager};

pub static CONFIG_KEY: &[u8] = b"config";
pub static ASSETS: &[u8] = b"assets";
pub static ASSET_LIST: &[u8] = b"asset_list";
pub static VIEWING_KEY: &[u8] = b"viewing_key";
pub static SELF_ADDRESS: &[u8] = b"self_address";
pub static ALLOCATIONS: &[u8] = b"allocations";
//pub static ALLOWANCE_REFRESH: &[u8] = b"allowance_refresh";
//pub static REWARDS: &[u8] = b"rewards_tracking";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, finance_manager::Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, finance_manager::Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn asset_list_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Vec<HumanAddr>> {
    singleton_read(storage, ASSET_LIST)
}

pub fn asset_list_w<S: Storage>(storage: &mut S) -> Singleton<S, Vec<HumanAddr>> {
    singleton(storage, ASSET_LIST)
}

pub fn assets_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Snip20Asset> {
    bucket_read(ASSETS, storage)
}

pub fn assets_w<S: Storage>(storage: &mut S) -> Bucket<S, Snip20Asset> {
    bucket(ASSETS, storage)
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

pub fn allocations_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Vec<finance_manager::AllocationMeta>> {
    bucket_read(ALLOCATIONS, storage)
}

pub fn allocations_w<S: Storage>(storage: &mut S) -> Bucket<S, Vec<finance_manager::AllocationMeta>> {
    bucket(ALLOCATIONS, storage)
}
