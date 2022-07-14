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
use shade_protocol::{
    contract_interfaces::{dao::treasury, snip20::helpers::Snip20Asset},
    utils::asset::Contract,
};

pub static CONFIG_KEY: &[u8] = b"config";
pub static ASSETS: &[u8] = b"assets";
pub static ASSET_LIST: &[u8] = b"asset_list";
pub static VIEWING_KEY: &[u8] = b"viewing_key";
pub static SELF_ADDRESS: &[u8] = b"self_address";
pub static ALLOWANCES: &[u8] = b"allowances";
//pub static CUR_ALLOWANCES: &[u8] = b"allowances";
pub static MANAGERS: &[u8] = b"managers";
pub static UNBONDING: &[u8] = b"unbonding";

pub fn config_w(storage: &mut dyn Storage) -> Singleton<treasury::Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r(storage: &dyn Storage) -> ReadonlySingleton<treasury::Config> {
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

pub fn allowances_r(storage: &dyn Storage) -> ReadonlyBucket<Vec<treasury::Allowance>> {
    bucket_read(storage, ALLOWANCES)
}

pub fn allowances_w(storage: &mut dyn Storage) -> Bucket<Vec<treasury::Allowance>> {
    bucket(storage, ALLOWANCES)
}

/*
pub fn current_allowances_r(storage: &dyn Storage) -> ReadonlyBucket<Addr> {
    bucket_read(storage, CUR_ALLOWANCES)
}

pub fn current_allowances_w(storage: &mut dyn Storage) -> Bucket<Addr> {
    bucket(storage, CUR_ALLOWANCES)
}
*/

pub fn managers_r(storage: &dyn Storage) -> ReadonlySingleton<Vec<treasury::Manager>> {
    singleton_read(storage, MANAGERS)
}

pub fn managers_w(storage: &mut dyn Storage) -> Singleton<Vec<treasury::Manager>> {
    singleton(storage, MANAGERS)
}


// Total unbonding per asset, to be used in rebalance
/*
pub fn total_unbonding_r(storage: &dyn Storage) -> ReadonlyBucket<Uint128> {
    bucket_read(storage, UNBONDING)
}

pub fn total_unbonding_w(storage: &mut dyn Storage) -> Bucket<Uint128> {
    bucket(storage, UNBONDING)
}

// Actually stored in accounts?
pub fn unbondings_r(storage: &dyn Storage) -> ReadonlyBucket<Uint128> {
    bucket_read(storage, UNBONDING)
}

pub fn unbondings_w(storage: &mut dyn Storage) -> Bucket<Uint128> {
    bucket(storage, UNBONDING)
}
*/
