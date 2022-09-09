use shade_protocol::Contract;
use shade_protocol::utility_router::RouterStatus;
use shade_protocol::utils::storage::plus::{Map, Item};


/// Maps contract name to contract address
pub const CONTRACTS: Map<String, Contract> = Map::new("contracts");

/// Status of the contract, either Running or UnderMaintenance
pub const STATUS: Item<RouterStatus> = Item::new("is_active");