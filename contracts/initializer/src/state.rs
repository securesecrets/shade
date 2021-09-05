use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use shade_protocol::initializer::InitializerConfig;

pub static CONFIG_KEY: &[u8] = b"config";

pub fn config_w<S: Storage>(storage: &mut S) -> Singleton<S, InitializerConfig> {
    singleton(storage, CONFIG_KEY)
}
pub fn config_r<S: Storage>(storage: &S) -> ReadonlySingleton<S, InitializerConfig> {
    singleton_read(storage, CONFIG_KEY)
}
