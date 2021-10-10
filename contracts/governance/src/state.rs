use cosmwasm_std::{Storage, Uint128};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, bucket, Bucket, bucket_read, ReadonlyBucket};
use shade_protocol::{
    governance::{Config, Proposal},
    asset::Contract,
};
use shade_protocol::governance::AdminCommand;

pub static CONFIG_KEY: &[u8] = b"config";
pub static CONTRACT_KEY: &[u8] = b"supported_contracts";
pub static CONTRACT_LIST_KEY: &[u8] = b"supported_contracts_list";
pub static PROPOSAL_KEY: &[u8] = b"proposals";
pub static TOTAL_PROPOSAL_KEY: &[u8] = b"total_proposals";
pub static ADMIN_COMMANDS_KEY: &[u8] = b"admin_commands";
pub static ADMIN_COMMANDS_LIST_KEY: &[u8] = b"admin_commands_list";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, CONFIG_KEY)
}

// Allows to to keep track of total proposals in an easier manner
pub fn total_proposals_w<S: Storage>(storage: &mut S) -> Singleton<S, Uint128> {
    singleton(storage, TOTAL_PROPOSAL_KEY)
}

pub fn total_proposals_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, Uint128> {
    singleton_read(storage, TOTAL_PROPOSAL_KEY)
}

pub fn proposal_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, Proposal> {
    bucket_read(PROPOSAL_KEY, storage)
}

pub fn proposal_w<S: Storage>(storage: &mut S) -> Bucket<S, Proposal> {
    bucket(PROPOSAL_KEY, storage)
}

pub fn supported_contract_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, Contract> {
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

pub fn admin_commands_r<S: Storage>(storage: & S) -> ReadonlyBucket<S, AdminCommand> {
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
