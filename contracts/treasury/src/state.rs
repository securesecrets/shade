use cosmwasm_std::{HumanAddr, Storage, Uint128};
use cosmwasm_storage::{
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
pub static ACCOUNT_LIST: &[u8] = b"account_list";
pub static ACCOUNT: &[u8] = b"account";
pub static UNBONDING: &[u8] = b"unbonding";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, treasury::Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, treasury::Config> {
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

pub fn allowances_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Vec<treasury::Allowance>> {
    bucket_read(ALLOWANCES, storage)
}

pub fn allowances_w<S: Storage>(storage: &mut S) -> Bucket<S, Vec<treasury::Allowance>> {
    bucket(ALLOWANCES, storage)
}

/*
pub fn current_allowances_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, HumanAddr> {
    bucket_read(CUR_ALLOWANCES, storage)
}

pub fn current_allowances_w<S: Storage>(storage: &mut S) -> Bucket<S, HumanAddr> {
    bucket(CUR_ALLOWANCES, storage)
}
*/

pub fn managers_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Vec<treasury::Manager>> {
    singleton_read(storage, MANAGERS)
}

pub fn managers_w<S: Storage>(storage: &mut S) -> Singleton<S, Vec<treasury::Manager>> {
    singleton(storage, MANAGERS)
}

pub fn account_list_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Vec<HumanAddr>> {
    singleton_read(storage, ACCOUNT_LIST)
}
pub fn account_list_w<S: Storage>(storage: &mut S) -> Singleton<S, Vec<HumanAddr>> {
    singleton(storage, ACCOUNT_LIST)
}

pub fn account_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, treasury::Account> {
    bucket_read(ACCOUNT, storage)
}

pub fn account_w<S: Storage>(storage: &mut S) -> Bucket<S, treasury::Account> {
    bucket(ACCOUNT, storage)
}

// Total unbonding per asset, to be used in rebalance
pub fn total_unbonding_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Uint128> {
    bucket_read(UNBONDING, storage)
}

pub fn total_unbonding_w<S: Storage>(storage: &mut S) -> Bucket<S, Uint128> {
    bucket(UNBONDING, storage)
}

pub fn unbondings_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Uint128> {
    bucket_read(UNBONDING, storage)
}

pub fn unbondings_w<S: Storage>(storage: &mut S) -> Bucket<S, Uint128> {
    bucket(UNBONDING, storage)
}
