use cosmwasm_std::{Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, bucket, Bucket, bucket_read, ReadonlyBucket};

use shade_protocol::staking::{Config, StakeState, Unbonding, UserStakeState};
use binary_heap_plus::{BinaryHeap, MinComparator};

pub static CONFIG_KEY: &[u8] = b"config";
pub static STAKE_STATE_KEY: &[u8] = b"stake_state";
pub static STAKER_KEY: &[u8] = b"staker";
pub static UNBONDING_KEY: &[u8] = b"unbonding";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn stake_state_w<S: Storage>(storage: &mut S) -> Singleton<S, StakeState> {
    singleton(storage, STAKE_STATE_KEY)
}

pub fn stake_state_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, StakeState> {
    singleton_read(storage, STAKE_STATE_KEY)
}

pub fn staker_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, UserStakeState> {
    bucket_read(STAKER_KEY, storage)
}

pub fn staker_w<S: Storage>(storage: &mut S) -> Bucket<S, UserStakeState> {
    bucket(STAKER_KEY, storage)
}

pub fn unbonding_w<S: Storage>(storage: &mut S) -> Singleton<S, BinaryHeap<Unbonding, MinComparator>> {
    singleton(storage, UNBONDING_KEY)
}

pub fn unbonding_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, BinaryHeap<Unbonding, MinComparator>> {
    singleton_read(storage, UNBONDING_KEY)
}