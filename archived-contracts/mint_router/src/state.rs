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

pub fn config_w(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, CONFIG)
}

pub fn config_r(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, CONFIG)
}

pub fn registered_asset_w(storage: &mut dyn Storage) -> Bucket<Contract> {
    bucket(storage, REGISTERED_ASSETS)
}

pub fn registered_asset_r(storage: &dyn Storage) -> ReadonlyBucket<Contract> {
    bucket_read(storage, REGISTERED_ASSETS)
}

/* Given a snip20 asset, gives the mint contract
 * furthest down the path
 */
pub fn asset_path_w(storage: &mut dyn Storage) -> Bucket<Contract> {
    bucket(storage, ASSET_PATH)
}

pub fn asset_path_r(storage: &dyn Storage) -> ReadonlyBucket<Contract> {
    bucket_read(storage, ASSET_PATH)
}

pub fn final_asset_w(storage: &mut dyn Storage) -> Singleton<Addr> {
    singleton(storage, FINAL_ASSET)
}

pub fn final_asset_r(storage: &dyn Storage) -> ReadonlySingleton<Addr> {
    singleton_read(storage, FINAL_ASSET)
}

pub fn current_assets_w(storage: &mut dyn Storage) -> Singleton<Vec<Contract>> {
    singleton(storage, CURRENT_ASSETS)
}

pub fn current_assets_r(storage: &dyn Storage) -> ReadonlySingleton<Vec<Contract>> {
    singleton_read(storage, CURRENT_ASSETS)
}

/* Needs to track the originating user across receive calls
 */
pub fn user_w(storage: &mut dyn Storage) -> Singleton<Addr> {
    singleton(storage, USER)
}

pub fn user_r(storage: &dyn Storage) -> ReadonlySingleton<Addr> {
    singleton_read(storage, USER)
}
