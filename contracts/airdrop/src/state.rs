use cosmwasm_std::{Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, bucket, Bucket, bucket_read, ReadonlyBucket};
use shade_protocol::airdrop::{Config, claim_info::Reward, account::Account};

pub static CONFIG_KEY: &[u8] = b"config";
pub static REWARDS_KEY: &[u8] = b"rewards";
pub static REWARD_IN_ACCOUNT_KEY: &[u8] = b"reward_in_account";
pub static ACCOUNTS_KEY: &[u8] = b"accounts";
pub static TOTAL_CLAIMED_KEY: &[u8] = b"total_claimed";
pub static USER_TOTAL_CLAIMED_KEY: &[u8] = b"user_total_claimed";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG_KEY)
}

// Airdrop eligible address
pub fn airdrop_address_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, Reward> {
    bucket_read(REWARDS_KEY, storage)
}

pub fn airdrop_address_w<S: Storage>(storage: &mut S) -> Bucket<S, Reward> {
    bucket(REWARDS_KEY, storage)
}

// Is address added to an account
pub fn address_in_account_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, bool> {
    bucket_read(REWARD_IN_ACCOUNT_KEY, storage)
}

pub fn address_in_account_w<S: Storage>(storage: &mut S) -> Bucket<S, bool> {
    bucket(REWARD_IN_ACCOUNT_KEY, storage)
}

// airdrop account
pub fn account_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, Account> {
    bucket_read(ACCOUNTS_KEY, storage)
}

pub fn account_w<S: Storage>(storage: &mut S) -> Bucket<S, Account> {
    bucket(ACCOUNTS_KEY, storage)
}

// If not found then its unrewarded; if true then claimed
pub fn claim_status_r<S: Storage>(storage: & S, index: usize) -> ReadonlyBucket<S, bool> {
    bucket_read(&[index as u8], storage)
}

pub fn claim_status_w<S: Storage>(storage: &mut S, index: usize) -> Bucket<S, bool> {
    bucket(&[index as u8], storage)
}

// Total claimed
pub fn total_claimed_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Uint128> {
    singleton_read(storage, TOTAL_CLAIMED_KEY)
}

pub fn total_claimed_w<S: Storage>(storage: &mut S) -> Singleton<S, Uint128> {
    singleton(storage, TOTAL_CLAIMED_KEY)
}

// Total account claimed
pub fn account_total_claimed_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, Uint128> {
    bucket_read(USER_TOTAL_CLAIMED_KEY, storage)
}

pub fn account_total_claimed_w<S: Storage>(storage: &mut S) -> Bucket<S, Uint128> {
    bucket(USER_TOTAL_CLAIMED_KEY, storage)
}