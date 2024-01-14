use shade_protocol::secret_storage_plus::Map;
use shade_protocol::c_std::Uint128;
pub const PRICE: Map<String, Uint128> = Map::new("price");