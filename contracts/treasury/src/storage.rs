use shade_protocol::{
    c_std::Addr,
    dao::treasury::{Allowance, AllowanceMeta, Config, Metric, RunLevel},
    secret_storage_plus::{Item, Map},
    snip20::helpers::Snip20Asset,
    utils::{
        asset::Contract,
        storage::plus::{iter_item::IterItem, period_storage::PeriodStorage},
    },
};

use chrono::*;

pub const CONFIG: Item<Config> = Item::new("config");
pub const VIEWING_KEY: Item<String> = Item::new("viewing_key");
pub const SELF_ADDRESS: Item<Addr> = Item::new("self_address");

pub const ASSET_LIST: IterItem<Addr, u64> = IterItem::new_override("asset_list", "asset_list_2");
pub const ASSET: Map<Addr, Snip20Asset> = Map::new("asset");

pub const MANAGER: Map<Addr, Contract> = Map::new("managers");
pub const ALLOWANCES: Map<Addr, Vec<AllowanceMeta>> = Map::new("allowances");

pub const RUN_LEVEL: Item<RunLevel> = Item::new("runlevel");

/*
pub fn metric_key(datetime: DateTime<Utc>) -> String {
    datetime.format("%Y-%m-%d").to_string()
}
*/

pub const METRICS: PeriodStorage<Metric> =
    PeriodStorage::new("metrics-all", "metrics-recent", "metrics-timed");
