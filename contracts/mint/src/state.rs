use shade_protocol::c_std::Uint128;
use shade_protocol::c_std::Storage;
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
    contract_interfaces::{
        mint::mint::{Config, SupportedAsset},
        snip20::helpers::Snip20Asset,
    },
    utils::asset::Contract,
};

pub static CONFIG: &[u8] = b"config";
pub static LIMIT: &[u8] = b"mint_limit";
pub static LIMIT_REFRESH: &[u8] = b"limit_refresh";
pub static MINTED: &[u8] = b"minted";
pub static NATIVE_ASSET: &[u8] = b"native_asset";
pub static ASSET_PEG: &[u8] = b"asset_peg";
pub static ASSET: &[u8] = b"assets";
pub static ASSET_LIST: &[u8] = b"asset_list";
pub static BURN_COUNT: &[u8] = b"burn_count";

pub fn config_w(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, CONFIG)
}

pub fn config_r(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, CONFIG)
}

/* Daily limit as (limit * total_supply) at the time of refresh
 */
pub fn limit_w(storage: &mut dyn Storage) -> Singleton<Uint128> {
    singleton(storage, LIMIT)
}

pub fn limit_r(storage: &dyn Storage) -> ReadonlySingleton<Uint128> {
    singleton_read(storage, LIMIT)
}

/* RFC-3339 datetime str, last time limit was refreshed
 */
pub fn limit_refresh_w(storage: &mut dyn Storage) -> Singleton<String> {
    singleton(storage, LIMIT_REFRESH)
}

pub fn limit_refresh_r(storage: &dyn Storage) -> ReadonlySingleton<String> {
    singleton_read(storage, LIMIT_REFRESH)
}

/* Amount minted this cycle (daily)
 */
pub fn minted_w(storage: &mut dyn Storage) -> Singleton<Uint128> {
    singleton(storage, MINTED)
}

pub fn minted_r(storage: &dyn Storage) -> ReadonlySingleton<Uint128> {
    singleton_read(storage, MINTED)
}

pub fn native_asset_w(storage: &mut dyn Storage) -> Singleton<Snip20Asset> {
    singleton(storage, NATIVE_ASSET)
}

pub fn native_asset_r(storage: &dyn Storage) -> ReadonlySingleton<Snip20Asset> {
    singleton_read(storage, NATIVE_ASSET)
}

pub fn asset_peg_w(storage: &mut dyn Storage) -> Singleton<String> {
    singleton(storage, ASSET_PEG)
}

pub fn asset_peg_r(storage: &dyn Storage) -> ReadonlySingleton<String> {
    singleton_read(storage, ASSET_PEG)
}

pub fn asset_list_w(storage: &mut dyn Storage) -> Singleton<Vec<Contract>> {
    singleton(storage, ASSET_LIST)
}

pub fn asset_list_r(storage: &dyn Storage) -> ReadonlySingleton<Vec<Contract>> {
    singleton_read(storage, ASSET_LIST)
}

pub fn assets_r(storage: &dyn Storage) -> ReadonlyBucket<SupportedAsset> {
    bucket_read(storage, ASSET)
}

pub fn assets_w(storage: &mut dyn Storage) -> Bucket<SupportedAsset> {
    bucket(storage, ASSET)
}

pub fn total_burned_r(storage: &dyn Storage) -> ReadonlyBucket<Uint128> {
    bucket_read(storage, BURN_COUNT)
}

pub fn total_burned_w(storage: &mut dyn Storage) -> Bucket<Uint128> {
    bucket(storage, BURN_COUNT)
}
