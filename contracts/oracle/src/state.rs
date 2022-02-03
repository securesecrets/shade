use cosmwasm_std::Storage;
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use shade_protocol::oracle::{IndexElement, OracleConfig, SswapPair};

pub static CONFIG_KEY: &[u8] = b"config";
pub static SSWAP_PAIRS: &[u8] = b"sswap_pairs";
pub static SIENNA_PAIRS: &[u8] = b"sienna_pairs";
pub static INDEX: &[u8] = b"index";

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, OracleConfig> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, OracleConfig> {
    singleton(storage, CONFIG_KEY)
}

pub fn sswap_pairs_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, SswapPair> {
    bucket_read(SSWAP_PAIRS, storage)
}

pub fn sswap_pairs_w<S: Storage>(storage: &mut S) -> Bucket<S, SswapPair> {
    bucket(SSWAP_PAIRS, storage)
}

pub fn sienna_pairs_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, SswapPair> {
    bucket_read(SIENNA_PAIRS, storage)
}

pub fn sienna_pairs_w<S: Storage>(storage: &mut S) -> Bucket<S, SswapPair> {
    bucket(SIENNA_PAIRS, storage)
}

pub fn index_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Vec<IndexElement>> {
    bucket_read(INDEX, storage)
}

pub fn index_w<S: Storage>(storage: &mut S) -> Bucket<S, Vec<IndexElement>> {
    bucket(INDEX, storage)
}
