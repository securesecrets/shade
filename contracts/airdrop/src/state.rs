use cosmwasm_std::Storage;
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, bucket, Bucket, bucket_read, ReadonlyBucket};
use shade_protocol::airdrop::{Config, Reward};

pub static CONFIG_KEY: &[u8] = b"config";
pub static REWARDS_KEY: &[u8] = b"rewards";
pub static CLAIM_STATUS_KEY: &[u8] = b"claim_status";

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

pub fn claim_status_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, bool> {
    bucket_read(CLAIM_STATUS_KEY, storage)
}

pub fn claim_status_w<S: Storage>(storage: &mut S) -> Bucket<S, bool> {
    bucket(CLAIM_STATUS_KEY, storage)
}
