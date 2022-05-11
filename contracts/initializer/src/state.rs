use cosmwasm_std::Storage;
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use shade_protocol::contract_interfaces::initializer::{Config, Snip20InitHistory};

pub static CONFIG_KEY: &[u8] = b"config";
pub static SHADE_KEY: &[u8] = b"shade";
pub static SILK_KEY: &[u8] = b"silk";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG_KEY)
}
pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn shade_w<S: Storage>(storage: &mut S) -> Singleton<S, Snip20InitHistory> {
    singleton(storage, SHADE_KEY)
}
pub fn shade_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Snip20InitHistory> {
    singleton_read(storage, SHADE_KEY)
}

pub fn silk_w<S: Storage>(storage: &mut S) -> Singleton<S, Snip20InitHistory> {
    singleton(storage, SILK_KEY)
}
pub fn silk_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Snip20InitHistory> {
    singleton_read(storage, SILK_KEY)
}
