use cosmwasm_std::{Storage, Uint128};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use shade_protocol::{
    bonds::{Config, Account},
    snip20::Snip20Asset,
    utils::asset::Contract,
};

pub static CONFIG: &[u8] = b"config";
pub static GLOBAL_ISSUANCE_CAP: &[u8] = b"global_issuance_cap";
pub static GLOBAL_TOTAL_MINTED: &[u8] = b"global_total_minted";
pub static BONDING_PERIOD: &[u8] = b"bonding_period";
pub static COLLATERAL_ASSET: &[u8] = b"collateral_asset";
pub static MINTED_ASSET: &[u8] = b"minted_asset";
pub static CLAIMED_STATUS_KEY: &[u8] = b"claimed_status";
pub static IS_CLAIMABLE_STATUS_KEY: &[u8] = b"is_claimable_status";
pub static ACCOUNTS_KEY: &[u8] = b"accounts";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG)
}

/* Issuance limit for particular bond instance */
pub fn issuance_cap_w<S: Storage>(storage: &mut S) -> Singleton<S, Uint128> {
    singleton(storage, ISSUANCE_CAP)
}

pub fn issuance_cap_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Uint128> {
    singleton_read(storage, ISSUANCE_CAP)
}

/* Amount minted during this bond's lifespan (e.g. 14 days) */
pub fn total_minted_w<S: Storage>(storage: &mut S) -> Singleton<S, Uint128> {
    singleton(storage, TOTAL_MINTED)
}

pub fn total_minted_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Uint128> {
    singleton_read(storage, TOTAL_MINTED)
}

/* Lifespan of the bond opportunity (e.g. 14 days) */
pub fn lifespan_w<S: Storage>(storage: &mut S) -> Singleton<S, String> {
    singleton(storage, LIFESPAN)
}

pub fn lifespan_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, String> {
    singleton_read(storage, LIFESPAN)
}

/* Duration after locking up collateral before minted tokens are claimable (e.g. 7 days) */
pub fn bonding_period_w<S: Storage>(storage: &mut S) -> Singleton<S, String> {
    singleton(storage, BONDING_PERIOD)
}

pub fn bonding_period_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, String> {
    singleton_read(storage, BONDING_PERIOD)
}

/* Asset sent to ShadeDAO as collateral */
pub fn collateral_asset_w<S: Storage>(storage: &mut S) -> Singleton<S, Snip20Asset> {
    singleton(storage, COLLATERAL_ASSET)
}

pub fn collateral_asset_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Snip20Asset> {
    singleton_read(storage, COLLATERAL_ASSET)
}

/* Asset minted when user claims after bonding period */
pub fn minted_asset_w<S: Storage>(storage: &mut S) -> Singleton<S, Snip20Asset> {
    singleton(storage, MINTED_ASSET)
}

pub fn minted_asset_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Snip20Asset> {
    singleton_read(storage, MINTED_ASSET)
}

// If true, has been claimed. If not found, then unclaimed
pub fn claimed_status_r<S: Storage>(storage: &S, index: usize) -> ReadonlyBucket<S, bool> {
    let mut key = CLAIMED_STATUS_KEY.to_vec();
    key.push(index as u8);
    bucket_read(&key, storage)
}

pub fn claimed_status_w<S: Storage>(storage: &mut S, index: usize) -> Bucket<S, bool> {
    let mut key = CLAIMED_STATUS_KEY.to_vec();
    key.push(index as u8);
    bucket(&key, storage)
}

// If true, is claimable. If false, not claimable
pub fn is_claimable_status_r<S: Storage>(storage: &S, index: usize) -> ReadonlyBucket<S, bool> {
    let mut key = IS_CLAIMABLE_STATUS_KEY.to_vec();
    key.push(index as u8);
    bucket_read(&key, storage)
}

pub fn is_claimable_status_w<S: Storage>(storage: &mut S, index: usize) -> Bucket<S, bool> {
    let mut key = IS_CLAIMABLE_STATUS_KEY.to_vec();
    key.push(index as u8);
    bucket(&key, storage)
}

// Bond account 
pub fn account_r<S: Storage>(storage: &S, index: usize) -> ReadonlyBucket<S, Account> {
    bucket_read(ACCOUNTS_KEY, storage)
}

pub fn account_w<S: Storage>(storage: &mut S, index: usize) -> Bucket<S, Account> {
    bucket(ACCOUNTS_KEY, storage)
}