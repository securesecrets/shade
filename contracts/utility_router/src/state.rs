use shade_protocol::Contract;
use shade_protocol::c_std::Addr;
use shade_protocol::utils::storage::plus::{Map, Item, ItemStorage};
use shade_protocol::utils::storage::plus::MapStorage;


pub struct ProtocolContract(pub Contract);

impl MapStorage<'static, String> for ProtocolContract {
    const MAP: Map<'static, String, Self> = Map::new("protocol-contract-");
}

/// Maps contract name to contract address
pub const CONTRACTS: Map<String, Contract> = Map::new("contracts");

/// Status of the contract, either Running or UnderMaintenance
pub const STATUS: Item<AdminAuthStatus> = Item::new("is_active");