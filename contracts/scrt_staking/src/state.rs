use shade_protocol::c_std::{Addr, Storage, Uint128};
use shade_protocol::storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use shade_protocol::contract_interfaces::dao::scrt_staking;

pub static CONFIG_KEY: &[u8] = b"config";
pub static SELF_ADDRESS: &[u8] = b"self_address";
pub static VIEWING_KEY: &[u8] = b"viewing_key";
pub static UNBONDING: &[u8] = b"unbonding";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, scrt_staking::Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &dyn Storage) -> ReadonlySingleton<S, scrt_staking::Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn self_address_w<S: Storage>(storage: &mut S) -> Singleton<S, Addr> {
    singleton(storage, SELF_ADDRESS)
}

pub fn self_address_r<S: Storage>(storage: &dyn Storage) -> ReadonlySingleton<S, Addr> {
    singleton_read(storage, SELF_ADDRESS)
}

pub fn viewing_key_w<S: Storage>(storage: &mut S) -> Singleton<S, String> {
    singleton(storage, VIEWING_KEY)
}

pub fn viewing_key_r<S: Storage>(storage: &dyn Storage) -> ReadonlySingleton<S, String> {
    singleton_read(storage, VIEWING_KEY)
}

pub fn unbonding_w<S: Storage>(storage: &mut S) -> Singleton<S, Uint128> {
    singleton(storage, UNBONDING)
}

pub fn unbonding_r<S: Storage>(storage: &dyn Storage) -> ReadonlySingleton<S, Uint128> {
    singleton_read(storage, UNBONDING)
}
