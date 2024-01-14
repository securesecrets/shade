use shade_protocol::secret_storage_plus::Item;
use shade_protocol::contract_interfaces::dex::secretswap::{
    PairResponse,
    PoolResponse,
};

pub const PAIR_INFO: Item<PairResponse> = Item::new("pair_info");
pub const POOL: Item<PoolResponse> = Item::new("pool");