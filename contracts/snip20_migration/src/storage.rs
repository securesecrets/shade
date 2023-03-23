use shade_protocol::{
    c_std::{Addr, Uint128},
    contract_interfaces::snip20_migration::Config,
    secret_storage_plus::{Item, Map},
    snip20_migration::RegisteredToken,
};

pub const CONFIG: Item<Config> = Item::new("config");
pub const REGISTERD_TOKENS: Map<'static, Addr, RegisteredToken> = Map::new("registered_tokens");
pub const AMOUNT_MINTED: Map<'static, Addr, Uint128> = Map::new("amount_minted");
