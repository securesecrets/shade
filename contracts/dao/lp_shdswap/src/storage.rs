use shade_protocol::{
    c_std::{Addr, Uint128},
    contract_interfaces::dao::lp_shdswap,
    secret_storage_plus::{Item, Map},
};

pub const CONFIG: Item<lp_shdswap::Config> = Item::new("config");
pub const VIEWING_KEY: Item<String> = Item::new("viewing_key");
pub const SELF_ADDRESS: Item<Addr> = Item::new("self_address");
pub const UNBONDING: Map<Addr, Uint128> = Map::new("unbonding");
