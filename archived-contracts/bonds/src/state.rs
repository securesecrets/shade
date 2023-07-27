use shade_protocol::c_std::Uint128;
use shade_protocol::c_std::{Addr, Storage};
use shade_protocol::storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use shade_protocol::contract_interfaces::{
    bonds::{Account, BondOpportunity, Config},
    snip20::helpers::Snip20Asset,
};

pub static CONFIG: &[u8] = b"config";
pub static GLOBAL_TOTAL_ISSUED: &[u8] = b"global_total_issued";
pub static GLOBAL_TOTAL_CLAIMED: &[u8] = b"global_total_claimed";
pub static DEPOSIT_ASSETS: &[u8] = b"deposit_assets";
pub static ISSUED_ASSET: &[u8] = b"issued_asset";
pub static ACCOUNTS_KEY: &[u8] = b"accounts";
pub static BOND_OPPORTUNITIES: &[u8] = b"bond_opportunities";
pub static ALLOCATED_ALLOWANCE: &[u8] = b"allocated_allowance";
pub static ALLOWANCE_VIEWING_KEY: &[u8] = b"allowance_viewing_key";

pub fn config_w(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, CONFIG)
}

pub fn config_r(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, CONFIG)
}

/* Global amount issued since last issuance reset */
pub fn global_total_issued_w(storage: &mut dyn Storage) -> Singleton<Uint128> {
    singleton(storage, GLOBAL_TOTAL_ISSUED)
}

pub fn global_total_issued_r(storage: &dyn Storage) -> ReadonlySingleton<Uint128> {
    singleton_read(storage, GLOBAL_TOTAL_ISSUED)
}

/* Global amount claimed since last issuance reset */
pub fn global_total_claimed_w(storage: &mut dyn Storage) -> Singleton<Uint128> {
    singleton(storage, GLOBAL_TOTAL_CLAIMED)
}

pub fn global_total_claimed_r(storage: &dyn Storage) -> ReadonlySingleton<Uint128> {
    singleton_read(storage, GLOBAL_TOTAL_CLAIMED)
}

/* List of assets that have bond opportunities stored */
pub fn deposit_assets_w(storage: &mut dyn Storage) -> Singleton<Vec<Addr>> {
    singleton(storage, DEPOSIT_ASSETS)
}

pub fn deposit_assets_r(storage: &dyn Storage) -> ReadonlySingleton<Vec<Addr>> {
    singleton_read(storage, DEPOSIT_ASSETS)
}

/* Asset minted when user claims after bonding period */
pub fn issued_asset_w(storage: &mut dyn Storage) -> Singleton<Snip20Asset> {
    singleton(storage, ISSUED_ASSET)
}

pub fn issued_asset_r(storage: &dyn Storage) -> ReadonlySingleton<Snip20Asset> {
    singleton_read(storage, ISSUED_ASSET)
}

// Bond account
pub fn account_r(storage: &dyn Storage) -> ReadonlyBucket<Account> {
    bucket_read(storage, ACCOUNTS_KEY)
}

pub fn account_w(storage: &mut dyn Storage) -> Bucket<Account> {
    bucket(storage, ACCOUNTS_KEY)
}

pub fn bond_opportunity_r(storage: &dyn Storage) -> ReadonlyBucket<BondOpportunity> {
    bucket_read(storage, BOND_OPPORTUNITIES)
}

pub fn bond_opportunity_w(storage: &mut dyn Storage) -> Bucket<BondOpportunity> {
    bucket(storage, BOND_OPPORTUNITIES)
}

// The amount of allowance already allocated/unclaimed from opportunities
pub fn allocated_allowance_w(storage: &mut dyn Storage) -> Singleton<Uint128> {
    singleton(storage, ALLOCATED_ALLOWANCE)
}

pub fn allocated_allowance_r(storage: &dyn Storage) -> ReadonlySingleton<Uint128> {
    singleton_read(storage, ALLOCATED_ALLOWANCE)
}

// Stores the bond contracts viewing key to see its own allowance
pub fn allowance_key_w(storage: &mut dyn Storage) -> Singleton<String> {
    singleton(storage, ALLOWANCE_VIEWING_KEY)
}

pub fn allowance_key_r(storage: &dyn Storage) -> ReadonlySingleton<String> {
    singleton_read(storage, ALLOWANCE_VIEWING_KEY)
}
