use cosmwasm_std::Addr;
use shade_protocol::{
    secret_storage_plus::{Map, Item},
    snip20::helpers::Snip20Asset,
    dao::treasury_manager::{Config, AllocationMeta, Holding},
};


pub const CONFIG: Item<Config> = Item::new("config");
pub const VIEWING_KEY: Item<String> = Item::new("viewing_key");
pub const SELF_ADDRESS: Item<Addr> = Item::new("self_address");

pub const ASSET_LIST: Item<Vec<Addr>> = Item::new("asset_list");
pub const ASSETS: Map<Addr, Snip20Asset> = Map::new("assets");

pub const ALLOCATIONS: Map<Addr, Vec<AllocationMeta>> = Map::new("allocations");
pub const HOLDERS: Item<Vec<Addr>> = Item::new("holders");
pub const HOLDING: Map<Addr, Holding> = Map::new("holding");
//pub const UNBONDINGS: Map<Addr, Vec<Unbonding>> = Map::new("unbondings");
