use cosmwasm_std::{Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, bucket, Bucket, bucket_read, ReadonlyBucket};
use shade_protocol::airdrop::{Config, StoredDelegator};

pub static CONFIG_KEY: &[u8] = b"config";
pub static SN_DELEGATORS_KEY: &[u8] = b"sn_delegators";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn sn_delegators_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, StoredDelegator> {
    bucket_read(SN_DELEGATORS_KEY, storage)
}

pub fn sn_delegators_w<S: Storage>(storage: &mut S) -> Bucket<S, StoredDelegator> {
    bucket(SN_DELEGATORS_KEY, storage)
}