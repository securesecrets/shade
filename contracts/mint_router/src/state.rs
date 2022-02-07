use cosmwasm_std::{Storage, Uint128};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use shade_protocol::{
    mint_router::Config,
    snip20::Snip20Asset,
    utils::asset::Contract,
};

pub static CONFIG: &[u8] = b"config";
pub static ASSET: &[u8] = b"assets";
pub static ASSET_LIST: &[u8] = b"asset_list";
pub static MINT: &[u8] = b"mint";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG)
}
