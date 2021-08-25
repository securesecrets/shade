use cosmwasm_std::{Storage, HumanAddr};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, bucket, Bucket, bucket_read, ReadonlyBucket};
use shade_protocol::{
    treasury::{TreasuryConfig, Snip20Asset},
};

pub static CONFIG_KEY: &[u8] = b"config";
pub static ASSET_KEY: &[u8] = b"assets";
pub static ASSET_LIST_KEY: &[u8] = b"asset_list";
pub static VIEWING_KEY: &[u8] = b"viewing_key";
pub static SELF_ADDRESS: &[u8] = b"self_address";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, TreasuryConfig> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, TreasuryConfig> {
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
