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
use shade_protocol::contract_interfaces::{dao::treasury_manager, snip20::helpers::Snip20Asset};

pub static CONFIG_KEY: &[u8] = b"config";
pub static ASSETS: &[u8] = b"assets";
pub static ASSET_LIST: &[u8] = b"asset_list";
pub static VIEWING_KEY: &[u8] = b"viewing_key";
pub static SELF_ADDRESS: &[u8] = b"self_address";
pub static ALLOCATIONS: &[u8] = b"allocations";
pub static HOLDERS: &[u8] = b"holders";
pub static HOLDER: &[u8] = b"holder";
pub static UNBONDINGS: &[u8] = b"unbondings";

pub fn config_w(storage: &mut dyn Storage) -> Singleton<treasury_manager::Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r(storage: &dyn Storage) -> ReadonlySingleton<treasury_manager::Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn asset_list_r(storage: &dyn Storage) -> ReadonlySingleton<Vec<Addr>> {
    singleton_read(storage, ASSET_LIST)
}

pub fn asset_list_w(storage: &mut dyn Storage) -> Singleton<Vec<Addr>> {
    singleton(storage, ASSET_LIST)
}

pub fn assets_r(storage: &dyn Storage) -> ReadonlyBucket<Snip20Asset> {
    bucket_read(storage, ASSETS)
}

pub fn assets_w(storage: &mut dyn Storage) -> Bucket<Snip20Asset> {
    bucket(storage, ASSETS)
}

pub fn viewing_key_r(storage: &dyn Storage) -> ReadonlySingleton<String> {
    singleton_read(storage, VIEWING_KEY)
}

pub fn viewing_key_w(storage: &mut dyn Storage) -> Singleton<String> {
    singleton(storage, VIEWING_KEY)
}

pub fn self_address_r(storage: &dyn Storage) -> ReadonlySingleton<Addr> {
    singleton_read(storage, SELF_ADDRESS)
}

pub fn self_address_w(storage: &mut dyn Storage) -> Singleton<Addr> {
    singleton(storage, SELF_ADDRESS)
}

pub fn allocations_r(
    storage: &dyn Storage,
) -> ReadonlyBucket<Vec<treasury_manager::AllocationMeta>> {
    bucket_read(storage, ALLOCATIONS)
}

pub fn allocations_w(
    storage: &mut dyn Storage,
) -> Bucket<Vec<treasury_manager::AllocationMeta>> {
    bucket(storage, ALLOCATIONS)
}

pub fn holders_r(storage: &dyn Storage) -> ReadonlySingleton<Vec<Addr>> {
    singleton_read(storage, HOLDERS)
}

pub fn holders_w(storage: &mut dyn Storage) -> Singleton<Vec<Addr>> {
    singleton(storage, HOLDERS)
}

pub fn holder_r(storage: &dyn Storage) -> ReadonlyBucket<treasury_manager::Holder> {
    bucket_read(storage, HOLDER)
}

pub fn holder_w(storage: &mut dyn Storage) -> Bucket<treasury_manager::Holder> {
    bucket(storage, HOLDER)
}

pub fn unbondings_r(storage: &dyn Storage) -> ReadonlyBucket<Vec<treasury_manager::Unbonding>> {
    bucket_read(storage, UNBONDINGS)
}

pub fn unbondings_w(storage: &mut dyn Storage) -> Bucket<Vec<treasury_manager::Unbonding>> {
    bucket(storage, HOLDER)
}

/*
pub fn unbonding_r(storage: &dyn Storage) -> ReadonlyBucket<Uint128> {
    bucket_read(storage, UNBONDING)
}

pub fn unbonding_w(storage: &mut dyn Storage) -> Bucket<Uint128> {
    bucket(storage, UNBONDING)
}
*/
