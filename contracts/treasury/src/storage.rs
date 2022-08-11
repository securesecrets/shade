use cosmwasm_std::Addr;
use shade_protocol::{
    dao::treasury::{Allowance, AllowanceMeta, Config, RunLevel},
    secret_storage_plus::{Item, Map},
    snip20::helpers::Snip20Asset,
    utils::asset::Contract,
};

pub const CONFIG: Item<Config> = Item::new("config");
pub const VIEWING_KEY: Item<String> = Item::new("viewing_key");
pub const SELF_ADDRESS: Item<Addr> = Item::new("self_address");

pub const ASSET_LIST: Item<Vec<Addr>> = Item::new("asset_list");

pub const ASSET: Map<Addr, Snip20Asset> = Map::new("asset");
pub const MANAGER: Map<Addr, Contract> = Map::new("managers");
pub const ALLOWANCES: Map<Addr, Vec<AllowanceMeta>> = Map::new("allowances");

pub const RUN_LEVEL: Item<RunLevel> = Item::new("runlevel");
