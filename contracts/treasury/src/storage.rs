use cosmwasm_std::Addr;
use shade_protocol::{
    secret_storage_plus::{Map, Item},
    snip20::helpers::Snip20Asset,
    dao::treasury::{Config, Allowance, Manager},
};

pub const CONFIG: Item<Config> = Item::new("config");
pub const VIEWING_KEY: Item<String> = Item::new("viewing_key");
pub const ASSET_LIST: Item<Vec<Addr>> = Item::new("asset_list");
pub const SELF_ADDRESS: Item<Addr> = Item::new("self_address");
pub const MANAGERS: Item<Vec<Manager>> = Item::new("managers");

pub const ALLOWANCES: Map<Addr, Vec<Allowance>> = Map::new("allowances");
pub const ASSETS: Map<Addr, Snip20Asset> = Map::new("assets");
