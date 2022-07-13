use shade_protocol::c_std::{Addr, Storage, Uint128};
use shade_protocol::storage::{
    bucket,
    bucket_read,
    singleton,
    singleton_read,
    Bucket,
    ReadonlyBucket,
    ReadonlySingleton,
    Singleton,
};
use shade_protocol::contract_interfaces::{dao::rewards_emission, snip20::helpers::Snip20Asset};

pub static CONFIG_KEY: &[u8] = b"config";
pub static SELF_ADDRESS: &[u8] = b"self_address";
pub static VIEWING_KEY: &[u8] = b"viewing_key";
pub static ASSETS: &[u8] = b"assets";
pub static ASSET: &[u8] = b"asset";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, rewards_emission::Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, rewards_emission::Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn self_address_w<S: Storage>(storage: &mut S) -> Singleton<S, Addr> {
    singleton(storage, SELF_ADDRESS)
}

pub fn self_address_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Addr> {
    singleton_read(storage, SELF_ADDRESS)
}

pub fn viewing_key_w<S: Storage>(storage: &mut S) -> Singleton<S, String> {
    singleton(storage, VIEWING_KEY)
}

pub fn viewing_key_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, String> {
    singleton_read(storage, VIEWING_KEY)
}

pub fn assets_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Vec<Addr>> {
    singleton_read(storage, ASSETS)
}

pub fn assets_w<S: Storage>(storage: &mut S) -> Singleton<S, Vec<Addr>> {
    singleton(storage, ASSETS)
}

pub fn asset_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Snip20Asset> {
    bucket_read(ASSET, storage)
}

pub fn asset_w<S: Storage>(storage: &mut S) -> Bucket<S, Snip20Asset> {
    bucket(ASSET, storage)
}
