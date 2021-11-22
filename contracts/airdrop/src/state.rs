use cosmwasm_std::Storage;
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, bucket, Bucket, bucket_read, ReadonlyBucket};
use shade_protocol::airdrop::{Config, Reward};

pub static CONFIG_KEY: &[u8] = b"config";
pub static REWARDS_KEY: &[u8] = b"rewards";

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