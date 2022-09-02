use shade_protocol::{c_std::Addr, dao::stkd_scrt};

use shade_protocol::secret_storage_plus::Item;

pub const CONFIG: Item<stkd_scrt::Config> = Item::new("config");
pub const SELF_ADDRESS: Item<Addr> = Item::new("self_address");
pub const VIEWING_KEY: Item<String> = Item::new("viewing_key");
