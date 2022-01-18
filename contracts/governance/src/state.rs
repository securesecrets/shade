use cosmwasm_std::Storage;
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};
use shade_protocol::{
    asset::Contract,
    governance::{AdminCommand, Config},
};

pub static CONFIG_KEY: &[u8] = b"config";
// Saved contracts
pub static CONTRACT_KEY: &[u8] = b"supported_contracts";
pub static CONTRACT_LIST_KEY: &[u8] = b"supported_contracts_list";
// Admin commands
pub static ADMIN_COMMANDS_KEY: &[u8] = b"admin_commands";
pub static ADMIN_COMMANDS_LIST_KEY: &[u8] = b"admin_commands_list";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG_KEY)
}

// Supported contracts

pub fn supported_contract_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Contract> {
    bucket_read(CONTRACT_KEY, storage)
}

pub fn supported_contract_w<S: Storage>(storage: &mut S) -> Bucket<S, Contract> {
    bucket(CONTRACT_KEY, storage)
}

pub fn supported_contracts_list_w<S: Storage>(storage: &mut S) -> Singleton<S, Vec<String>> {
    singleton(storage, CONTRACT_LIST_KEY)
}

pub fn supported_contracts_list_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Vec<String>> {
    singleton_read(storage, CONTRACT_LIST_KEY)
}

// Admin commands

pub fn admin_commands_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, AdminCommand> {
    bucket_read(ADMIN_COMMANDS_KEY, storage)
}

pub fn admin_commands_w<S: Storage>(storage: &mut S) -> Bucket<S, AdminCommand> {
    bucket(ADMIN_COMMANDS_KEY, storage)
}

pub fn admin_commands_list_w<S: Storage>(storage: &mut S) -> Singleton<S, Vec<String>> {
    singleton(storage, ADMIN_COMMANDS_LIST_KEY)
}

pub fn admin_commands_list_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Vec<String>> {
    singleton_read(storage, ADMIN_COMMANDS_LIST_KEY)
}
