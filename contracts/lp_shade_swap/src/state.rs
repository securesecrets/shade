use cosmwasm_std::{HumanAddr, Storage, Uint128};
use cosmwasm_storage::{bucket, bucket_read, Bucket, ReadonlyBucket, singleton, singleton_read, ReadonlySingleton, Singleton};
use shade_protocol::lp_shade_swap;

pub static CONFIG_KEY: &[u8] = b"config";
pub static SELF_ADDRESS: &[u8] = b"self_address";
pub static VIEWING_KEY: &[u8] = b"viewing_key";
pub static UNBONDING: &[u8] = b"unbonding";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, lp_shade_swap::Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, lp_shade_swap::Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn self_address_w<S: Storage>(storage: &mut S) -> Singleton<S, HumanAddr> {
    singleton(storage, SELF_ADDRESS)
}

pub fn self_address_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, HumanAddr> {
    singleton_read(storage, SELF_ADDRESS)
}

pub fn viewing_key_w<S: Storage>(storage: &mut S) -> Singleton<S, String> {
    singleton(storage, VIEWING_KEY)
}

pub fn viewing_key_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, String> {
    singleton_read(storage, VIEWING_KEY)
}

pub fn unbonding_w<S: Storage>(storage: &mut S) -> Bucket<S, Uint128> {
    bucket(UNBONDING, storage)
}

pub fn unbonding_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Uint128> {
    bucket_read(UNBONDING, storage)
}
