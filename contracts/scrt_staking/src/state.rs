use shade_protocol::c_std::{Addr, Storage, Uint128};
use shade_protocol::storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use shade_protocol::contract_interfaces::dao::scrt_staking;

pub static CONFIG_KEY: &[u8] = b"config";
pub static SELF_ADDRESS: &[u8] = b"self_address";
pub static VIEWING_KEY: &[u8] = b"viewing_key";
pub static UNBONDING: &[u8] = b"unbonding";

pub fn config_w(storage: &mut dyn Storage) -> Singleton<scrt_staking::Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r(storage: &dyn Storage) -> ReadonlySingleton<scrt_staking::Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn self_address_w(storage: &mut dyn Storage) -> Singleton<Addr> {
    singleton(storage, SELF_ADDRESS)
}

pub fn self_address_r(storage: &dyn Storage) -> ReadonlySingleton<Addr> {
    singleton_read(storage, SELF_ADDRESS)
}

pub fn viewing_key_w(storage: &mut dyn Storage) -> Singleton<String> {
    singleton(storage, VIEWING_KEY)
}

pub fn viewing_key_r(storage: &dyn Storage) -> ReadonlySingleton<String> {
    singleton_read(storage, VIEWING_KEY)
}

pub fn unbonding_w(storage: &mut dyn Storage) -> Singleton<Uint128> {
    singleton(storage, UNBONDING)
}

pub fn unbonding_r(storage: &dyn Storage) -> ReadonlySingleton<Uint128> {
    singleton_read(storage, UNBONDING)
}
