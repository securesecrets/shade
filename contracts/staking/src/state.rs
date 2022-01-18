use cosmwasm_std::Storage;
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

use binary_heap_plus::{BinaryHeap, MinComparator};
use shade_protocol::staking::{
    stake::{Stake, Unbonding, UserStake},
    Config,
};

pub static CONFIG_KEY: &[u8] = b"config";
pub static STAKE_STATE_KEY: &[u8] = b"stake_state";
pub static STAKER_KEY: &[u8] = b"staker";
pub static UNBONDING_KEY: &[u8] = b"unbonding";
pub static USER_UNBONDING_KEY: &[u8] = b"user_unbonding";
pub static VIEWKING_KEY: &[u8] = b"viewing_key";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn stake_state_w<S: Storage>(storage: &mut S) -> Singleton<S, Stake> {
    singleton(storage, STAKE_STATE_KEY)
}

pub fn stake_state_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Stake> {
    singleton_read(storage, STAKE_STATE_KEY)
}

pub fn staker_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, UserStake> {
    bucket_read(STAKER_KEY, storage)
}

pub fn staker_w<S: Storage>(storage: &mut S) -> Bucket<S, UserStake> {
    bucket(STAKER_KEY, storage)
}

// Ideally these queues will be removed
pub fn unbonding_w<S: Storage>(
    storage: &mut S,
) -> Singleton<S, BinaryHeap<Unbonding, MinComparator>> {
    singleton(storage, UNBONDING_KEY)
}

pub fn unbonding_r<S: Storage>(
    storage: &S,
) -> ReadonlySingleton<S, BinaryHeap<Unbonding, MinComparator>> {
    singleton_read(storage, UNBONDING_KEY)
}

pub fn user_unbonding_r<S: Storage>(
    storage: &S,
) -> ReadonlyBucket<S, BinaryHeap<Unbonding, MinComparator>> {
    bucket_read(USER_UNBONDING_KEY, storage)
}

pub fn user_unbonding_w<S: Storage>(
    storage: &mut S,
) -> Bucket<S, BinaryHeap<Unbonding, MinComparator>> {
    bucket(USER_UNBONDING_KEY, storage)
}

pub fn viewking_key_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, String> {
    bucket_read(VIEWKING_KEY, storage)
}

pub fn viewking_key_w<S: Storage>(storage: &mut S) -> Bucket<S, String> {
    bucket(VIEWKING_KEY, storage)
}
