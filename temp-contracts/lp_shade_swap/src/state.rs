use shade_protocol::c_std::{Addr, Storage, Uint128};
use shade_protocol::storage::{bucket, bucket_read, Bucket, ReadonlyBucket, singleton, singleton_read, ReadonlySingleton, Singleton};
use shade_protocol::contract_interfaces::dao::lp_shade_swap;

pub static CONFIG_KEY: &[u8] = b"config";
pub static SELF_ADDRESS: &[u8] = b"self_address";
pub static VIEWING_KEY: &[u8] = b"viewing_key";
pub static UNBONDING: &[u8] = b"unbonding";

pub fn config_w(storage: &mut dyn Storage) -> Singleton<lp_shade_swap::Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r(storage: &dyn Storage) -> ReadonlySingleton<lp_shade_swap::Config> {
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

pub fn unbonding_w(storage: &mut dyn Storage) -> Bucket<Uint128> {
    bucket(storage, UNBONDING)
}

pub fn unbonding_r(storage: &dyn Storage) -> ReadonlyBucket<Uint128> {
    bucket_read(storage, UNBONDING)
}
