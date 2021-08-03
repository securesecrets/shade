use cosmwasm_std::{Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, PrefixedStorage, ReadonlyPrefixedStorage};
use shade_protocol::oracle::OracleConfig;

pub static CONFIG_KEY: &[u8] = b"config";
pub static PRICE_KEY: &[u8] = b"price";

/*
pub struct ReferenceData {
    pub rate: uint256,
    pub last_updated_base: uint256,
    pub last_updated_quote: uint256,
}
*/

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, OracleConfig> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, OracleConfig> {
    singleton_read(storage, CONFIG_KEY)
}

// price
pub fn price<S: Storage>(storage: &mut S) -> PrefixedStorage<S> {
    PrefixedStorage::new(PRICE_KEY, storage)
}

pub fn price_read<S: Storage>(storage: &S) -> ReadonlyPrefixedStorage<S> {
    ReadonlyPrefixedStorage::new(PRICE_KEY, storage)
}
