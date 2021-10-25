use cosmwasm_std::{Storage, HumanAddr};
use cosmwasm_storage::{
    singleton, singleton_read,
    Singleton, ReadonlySingleton,
    Bucket, ReadonlyBucket,
    bucket, bucket_read,
};
use shade_protocol::{
    treasury,
    snip20::Snip20Asset,
};

pub static CONFIG_KEY: &[u8] = b"config";
pub static ASSET_KEY: &[u8] = b"assets";
pub static ASSET_LIST_KEY: &[u8] = b"asset_list";
pub static VIEWING_KEY: &[u8] = b"viewing_key";
pub static SELF_ADDRESS: &[u8] = b"self_address";
pub static APPS: &[u8] = b"applications";
//pub static APP: &[u8] = b"application";
pub static ALLOCS: &[u8] = b"allocations";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, treasury::Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, treasury::Config> {
    singleton_read(storage, CONFIG_KEY)
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

pub fn apps_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Vec<HumanAddr>> {
    singleton_read(storage, APPS)
}

pub fn apps_w<S: Storage>(storage: &mut S) -> Singleton<S, Vec<HumanAddr>> {
    singleton(storage, APPS)
}

pub fn allocations_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Vec<treasury::Application>> {
    bucket_read(ALLOCS, storage)
}

pub fn allocations_w<S: Storage>(storage: &mut S) -> Bucket<S, Vec<treasury::Application>> {
    bucket(ALLOCS, storage)
}
