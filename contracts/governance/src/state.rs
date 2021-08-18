use cosmwasm_std::{Storage, HumanAddr};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, bucket, Bucket, bucket_read, ReadonlyBucket};
use shade_protocol::{
    governance::{GovernanceConfig, AssetPair},
};

pub static CONFIG_KEY: &[u8] = b"config";
pub static ASSET_KEY: &[u8] = b"assets";
pub static ASSET_LIST_KEY: &[u8] = b"asset_list";
pub static VIEWING_KEY: &[u8] = b"viewing_key";
pub static SELF_ADDRESS: &[u8] = b"self_address";
pub static MINT_KEY: &[u8] = b"mint_key";
pub static ASSET_PAIRS: &[u8] = b"asset_pairs";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, GovernanceConfig> 
{
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, GovernanceConfig> 
{
    singleton_read(storage, CONFIG_KEY)
}

pub fn asset_pairs<S: Storage>(storage: &mut S) -> Singleton<S, Vec<AssetPair>> 
{
    singleton(storage, ASSET_PAIRS)
}

pub fn asset_pairs_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, Vec<AssetPair>> 
{
    singleton_read(storage, ASSET_PAIRS)
}

// Governance shouldn't need viewing key, no reason for funds to be on governance
/*
pub fn viewing_key_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, String> {
    singleton_read(storage, VIEWING_KEY)
}

pub fn viewing_key_w<S: Storage>(storage: &mut S) -> Singleton<S, String> {
    singleton(storage, VIEWING_KEY)
}

pub fn self_address_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, HumanAddr> {
    singleton_read(storage, SELF_ADDRESS)
}

pub fn self_address_w<S: Storage>(storage: &mut S) -> Singleton<S, HumanAddr> {
    singleton(storage, SELF_ADDRESS)
}
*/
