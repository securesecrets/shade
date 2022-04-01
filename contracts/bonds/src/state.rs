use cosmwasm_std::{Storage, Uint128};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use shade_protocol::{
    bonds::{Config, Account, BondOpportunity},
    snip20::Snip20Asset,
    utils::asset::Contract,
};

pub static CONFIG: &[u8] = b"config";
pub static GLOBAL_ISSUANCE_LIMIT: &[u8] = b"global_issuance_limit";
pub static GLOBAL_TOTAL_ISSUED: &[u8] = b"global_total_issued";
pub static BOND_ISSUANCE_LIMIT: &[u8] = b"bond_issuance_limit"; 
pub static BOND_TOTAL_ISSUED: &[u8] = b"bond_total_issued";
pub static BONDING_PERIOD: &[u8] = b"bonding_period";
pub static COLLATERAL_ASSETS: &[u8] = b"collateral_assets";
pub static ISSUED_ASSET: &[u8] = b"issued_asset";
pub static ACCOUNTS_KEY: &[u8] = b"accounts";
pub static BOND_OPPORTUNITIES: &[u8] = b"bond_opportunities";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG)
}

/* Global issuance limit for all bond opportunities */
pub fn global_issuance_limit_w<S: Storage>(storage: &mut S) -> Singleton<S, Uint128> {
    singleton(storage, GLOBAL_ISSUANCE_LIMIT)
}

pub fn global_issuance_limit_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Uint128> {
    singleton_read(storage, GLOBAL_ISSUANCE_LIMIT)
}

/* Global amount issued since last issuance reset */
pub fn global_total_issued_w<S: Storage>(storage: &mut S) -> Singleton<S, Uint128> {
    singleton(storage, GLOBAL_TOTAL_ISSUED)
}

pub fn global_total_issued_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Uint128> {
    singleton_read(storage, GLOBAL_TOTAL_ISSUED)
}

/* Issuance limit for particular bond instance */
pub fn bond_issuance_limit_w<S: Storage>(storage: &mut S) -> Singleton<S, Uint128> {
    singleton(storage, BOND_ISSUANCE_LIMIT)
}

pub fn bond_issuance_limit_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Uint128> {
    singleton_read(storage, BOND_ISSUANCE_LIMIT)
}

/* Amount minted during this bond's lifespan (e.g. 14 days) */
pub fn bond_total_issued_w<S: Storage>(storage: &mut S) -> Singleton<S, Uint128> {
    singleton(storage, BOND_TOTAL_ISSUED)
}

pub fn bond_total_issued_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Uint128> {
    singleton_read(storage, BOND_TOTAL_ISSUED)
}

/* Duration after locking up collateral before minted tokens are claimable (e.g. 7 days) */
pub fn bonding_period_w<S: Storage>(storage: &mut S) -> Singleton<S, String> {
    singleton(storage, BONDING_PERIOD)
}

pub fn bonding_period_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, String> {
    singleton_read(storage, BONDING_PERIOD)
}

/* Assets sent to ShadeDAO as collateral */
pub fn collateral_assets_w<S: Storage>(storage: &mut S) -> Bucket<S, Snip20Asset> {
    bucket(COLLATERAL_ASSETS, storage)
}

pub fn collateral_assets_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Snip20Asset> {
    bucket_read(COLLATERAL_ASSETS, storage)
}

/* Asset minted when user claims after bonding period */
pub fn issued_asset_w<S: Storage>(storage: &mut S) -> Singleton<S, Snip20Asset> {
    singleton(storage, ISSUED_ASSET)
}

pub fn issued_asset_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Snip20Asset> {
    singleton_read(storage, ISSUED_ASSET)
}

// Bond account 
pub fn account_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Account> {
    bucket_read(ACCOUNTS_KEY, storage)
}

pub fn account_w<S: Storage>(storage: &mut S) -> Bucket<S, Account> {
    bucket(ACCOUNTS_KEY, storage)
}

pub fn bond_opportunity_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, BondOpportunity> {
    bucket_read(BOND_OPPORTUNITIES, storage)
}

pub fn bond_opportunity_w<S: Storage>(storage: &mut S) -> Bucket<S, BondOpportunity> {
    bucket(BOND_OPPORTUNITIES, storage)
}