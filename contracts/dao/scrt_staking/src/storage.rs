use shade_protocol::c_std::{Addr, Uint128};
use shade_protocol::dao::scrt_staking;

use shade_protocol::secret_storage_plus::Item;

pub const CONFIG: Item<scrt_staking::Config> = Item::new("config");
pub const SELF_ADDRESS: Item<Addr> = Item::new("self_address");
pub const VIEWING_KEY: Item<String> = Item::new("viewing_key");
pub const UNBONDING: Item<Uint128> = Item::new("unbonding");
