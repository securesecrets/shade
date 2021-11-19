use cosmwasm_std::{Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, bucket, Bucket, bucket_read, ReadonlyBucket};
use shade_protocol::airdrop::{Config, Reward};

pub static CONFIG_KEY: &[u8] = b"config";
pub static REWARDS_KEY: &[u8] = b"rewards";
pub static TOTAL_CLAIMED_KEY: &[u8] = b"total_claimed";
pub static USER_TOTAL_CLAIMED_KEY: &[u8] = b"user_total_claimed";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn reward_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, Reward> {
    bucket_read(REWARDS_KEY, storage)
}

pub fn reward_w<S: Storage>(storage: &mut S) -> Bucket<S, Reward> {
    bucket(REWARDS_KEY, storage)
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

// Total user claimed
pub fn user_total_claimed_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, Uint128> {
    bucket_read(USER_TOTAL_CLAIMED_KEY, storage)
}

pub fn user_total_claimed_w<S: Storage>(storage: &mut S) -> Bucket<S, Uint128> {
    bucket(USER_TOTAL_CLAIMED_KEY, storage)
}