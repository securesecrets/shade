use cosmwasm_std::{Storage, HumanAddr};
use cosmwasm_storage::{
    singleton, Singleton, 
    singleton_read, ReadonlySingleton, 
};
use shade_protocol::scrt_staking;

pub static CONFIG_KEY: &[u8] = b"config";
pub static SELF_ADDRESS: &[u8] = b"self_address";
pub static VIEWING_KEY: &[u8] = b"viewing_key";

//pub static DELEGATIONS: &[u8] = b"delegations";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, scrt_staking::Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, scrt_staking::Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn self_address_w<S: Storage>(storage: &mut S) -> Singleton<S, HumanAddr> {
    singleton(storage, SELF_ADDRESS)
}

pub fn self_address_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, HumanAddr> {
    singleton_read(storage, SELF_ADDRESS)
}

pub fn viewing_key_w<S: Storage>(storage: &mut S) -> Singleton<S, String> {
    singleton(storage, VIEWING_KEY)
}

pub fn viewing_key_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, String> {
    singleton_read(storage, VIEWING_KEY)
}

/*
pub fn delegations_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Vec<Delegation>> {
    singleton_read(storage, DELEGATIONS)
}

pub fn delegations_w<S: Storage>(storage: &mut S) -> Singleton<S, Vec<Delegation>> {
    singleton(storage, DELEGATIONS)
}
*/
