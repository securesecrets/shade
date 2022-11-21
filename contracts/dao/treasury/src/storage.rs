use shade_protocol::{
    c_std::Addr,
    dao::treasury::{AllowanceMeta, Config, Metric, RunLevel},
    secret_storage_plus::{Item, Map},
    snip20::helpers::Snip20Asset,
    utils::{
        asset::Contract,
        storage::plus::{iter_item::IterItem, period_storage::PeriodStorage},
    },
};

pub const CONFIG: Item<Config> = Item::new("config");
pub const VIEWING_KEY: Item<String> = Item::new("viewing_key");

pub const ASSET_LIST: IterItem<Addr, u64> = IterItem::new_override("asset_list", "asset_list_2");
pub const ASSET: Map<Addr, Snip20Asset> = Map::new("asset");

// { denom: snip20 }
pub const WRAP: Map<String, Addr> = Map::new("wrap");

pub const MANAGER: Map<Addr, Contract> = Map::new("managers");
pub const ALLOWANCES: Map<Addr, Vec<AllowanceMeta>> = Map::new("allowances");

pub const RUN_LEVEL: Item<RunLevel> = Item::new("runlevel");

pub const METRICS: PeriodStorage<Metric> =
    PeriodStorage::new("metrics-all", "metrics-recent", "metrics-timed");
