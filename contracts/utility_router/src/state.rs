use shade_protocol::{
    secret_storage_plus::{Item, Map},
    utility_router::RouterStatus,
    Contract,
};

/// Maps contract name to contract address
pub const CONTRACTS: Map<String, Contract> = Map::new("contracts");

/// Maps address names to address
pub const ADDRESSES: Map<String, String> = Map::new("addresses");

/// Status of the contract, either Running or UnderMaintenance
pub const STATUS: Item<RouterStatus> = Item::new("is_active");
