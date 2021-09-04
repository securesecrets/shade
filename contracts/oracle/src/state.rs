use cosmwasm_std::{
    Storage
};
use cosmwasm_storage::{
    singleton, singleton_read, 
    Singleton, ReadonlySingleton,
    bucket, bucket_read,
    Bucket, ReadonlyBucket
};
use shade_protocol::{
    oracle::{ OracleConfig, SswapPair },
    band::ReferenceData,
    asset::Contract,
};

pub static CONFIG_KEY: &[u8] = b"config";
pub static HARD_CODED: &[u8] = b"hard_coded";
pub static SSWAP_PAIRS: &[u8] = b"sswap_pairs";


pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, OracleConfig> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, OracleConfig> {
    singleton(storage, CONFIG_KEY)
}

pub fn hard_coded_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, ReferenceData> {
    bucket_read(HARD_CODED, storage)
}

pub fn hard_coded_w<S: Storage>(storage: &mut S) -> Bucket<S, ReferenceData> {
    bucket(HARD_CODED, storage)
}

pub fn sswap_pairs_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, SswapPair> {
    bucket_read(SSWAP_PAIRS, storage)
}

pub fn sswap_pairs_w<S: Storage>(storage: &mut S) -> Bucket<S, SswapPair> {
    bucket(SSWAP_PAIRS, storage)
}
