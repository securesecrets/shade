use shade_protocol::c_std::{Addr, Storage, Uint128};
use shade_protocol::storage::{
    bucket,
    bucket_read,
    singleton,
    singleton_read,
    Bucket,
    ReadonlyBucket,
    ReadonlySingleton,
    Singleton,
};
use shade_protocol::contract_interfaces::{dao::rewards_emission, snip20::helpers::Snip20Asset};

use shade_protocol::{
    secret_storage_plus::{Map, Item},
};

pub const CONFIG: Item<rewards_emission::Config> = Item::new("config");
pub const SELF_ADDRESS: Item<Addr> = Item::new("self_address");
pub const VIEWING_KEY: Item<String> = Item::new("viewing_key");
pub const TOKEN: Item<Snip20Asset> = Item::new("token");
pub const REWARD: Map<Addr, rewards_emission::Reward> = Map::new("rewards");

