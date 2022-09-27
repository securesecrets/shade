use shade_protocol::{
    c_std::Addr,
    secret_storage_plus::{Item, Map},
    utility_router::RouterStatus,
    Contract,
};

/// Maps contract name to contract address
pub const CONTRACTS: Map<String, Contract> = Map::new("contracts");

/// Maps address names to address
pub const ADDRESSES: Map<String, Addr> = Map::new("addresses");

pub const KEYS: Item<Vec<String>> = Item::new("keys");

/// Status of the contract, either Running or UnderMaintenance
pub const STATUS: Item<RouterStatus> = Item::new("is_active");
