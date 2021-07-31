use cosmwasm_std::{Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use shade_protocol::oracle::OracleConfig;

pub static CONFIG_KEY: &[u8] = b"config";

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
