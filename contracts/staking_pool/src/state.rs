use cosmwasm_std::{Storage, HumanAddr};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, bucket, Bucket, bucket_read, ReadonlyBucket};
use shade_protocol::staking_pool;
};

pub static CONFIG_KEY: &[u8] = b"config";
pub static DELEGATIONS: &[u8] = b"delegations";
pub static UNBONDINGS: &[u8] = b"unbondings";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, staking_pool::Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, staking_pool::Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn delegations_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Vec<Delegation>> {
    singleton_read(storage, DELEGATIONS)
}

pub fn delegations_w<S: Storage>(storage: &mut S) -> Singleton<S, Vec<Delegation>> {
    singleton(storage, DELEGATIONS)
}

// User address -> delegations
pub fn unbondings_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Vec<Unbonding>> {
    bucket_read(storage, UNBONDINGS)
}

pub fn unbondings_w<S: Storage>(storage: &mut S) -> Bucket<S, Vec<Unbonding>> {
    bucket(storage, UNBONDINGS)
}
