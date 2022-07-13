use shade_protocol::c_std::Uint128;
use shade_protocol::c_std::{Addr, Storage};
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
use shade_protocol::{
    contract_interfaces::{mint::mint_router::Config, snip20::helpers::Snip20Asset},
    utils::asset::Contract,
};

pub static CONFIG: &[u8] = b"config";
pub static REGISTERED_ASSETS: &[u8] = b"registered_assets";
pub static CURRENT_ASSETS: &[u8] = b"current_assets";
pub static ASSET_PATH: &[u8] = b"asset_path";
pub static FINAL_ASSET: &[u8] = b"final_asset";
pub static USER: &[u8] = b"user";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG)
}

pub fn registered_asset_w<S: Storage>(storage: &mut S) -> Bucket<S, Contract> {
    bucket(REGISTERED_ASSETS, storage)
}

pub fn registered_asset_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Contract> {
    bucket_read(REGISTERED_ASSETS, storage)
}

/* Given a snip20 asset, gives the mint contract
 * furthest down the path
 */
pub fn asset_path_w<S: Storage>(storage: &mut S) -> Bucket<S, Contract> {
    bucket(ASSET_PATH, storage)
}

pub fn asset_path_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Contract> {
    bucket_read(ASSET_PATH, storage)
}

pub fn final_asset_w<S: Storage>(storage: &mut S) -> Singleton<S, Addr> {
    singleton(storage, FINAL_ASSET)
}

pub fn final_asset_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Addr> {
    singleton_read(storage, FINAL_ASSET)
}

pub fn current_assets_w<S: Storage>(storage: &mut S) -> Singleton<S, Vec<Contract>> {
    singleton(storage, CURRENT_ASSETS)
}

pub fn current_assets_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Vec<Contract>> {
    singleton_read(storage, CURRENT_ASSETS)
}

/* Needs to track the originating user across receive calls
 */
pub fn user_w<S: Storage>(storage: &mut S) -> Singleton<S, Addr> {
    singleton(storage, USER)
}

pub fn user_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Addr> {
    singleton_read(storage, USER)
}
