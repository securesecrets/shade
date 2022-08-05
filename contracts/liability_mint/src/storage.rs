use shade_protocol::c_std::{
    Uint128,
    Storage,
    Addr,
};
use shade_protocol::secret_storage_plus::{Item};
use shade_protocol::contract_interfaces::mint::liability_mint::{Config};
use shade_protocol::snip20::helpers::Snip20Asset;

pub const CONFIG: Item<Config> = Item::new("config");
pub const LIABILITIES: Item<Uint128> = Item::new("liabilities");
// TODO: change to Contract
pub const TOKEN: Item<Snip20Asset> = Item::new("token");
pub const WHITELIST: Item<Vec<Addr>> = Item::new("whitelist");
