use cosmwasm_std::{Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, bucket, Bucket, bucket_read, ReadonlyBucket};
use shade_protocol::{
    mint::{MintConfig, SupportedAsset},
};

pub static CONFIG_KEY: &[u8] = b"config";
pub static NATIVE_COIN_KEY: &[u8] = b"native_coin";
pub static ASSET_KEY: &[u8] = b"assets";
pub static ASSET_LIST_KEY: &[u8] = b"asset_list";

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, MintConfig> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, MintConfig> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn asset_list<S: Storage>(storage: &mut S) -> Singleton<S, Vec<String>> {
    singleton(storage, ASSET_LIST_KEY)
}

pub fn asset_list_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, Vec<String>> {
    singleton_read(storage, ASSET_LIST_KEY)
}

pub fn assets_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, SupportedAsset> {
    bucket_read(ASSET_KEY, storage)
}

pub fn assets_w<S: Storage>(storage: &mut S) -> Bucket<S, SupportedAsset> {
    bucket(ASSET_KEY, storage)
}