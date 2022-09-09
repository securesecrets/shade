use crate::{
    c_std::{StdError, StdResult, Storage, Timestamp},
    chrono::prelude::*,
    cosmwasm_schema::cw_serde,
    serde::{de::DeserializeOwned, Serialize},
    utils::cycle::*,
};
pub use secret_storage_plus::{Item, Json, Map, PrimaryKey, Serde};

use const_format::{concatcp, formatcp};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[cw_serde]
#[derive(EnumIter)]
pub enum Period {
    Hour,
    Day,
    Month,
}

pub fn map_key(seconds: u64, period: Period) -> String {
    let datetime = utc_from_seconds(seconds as i64);
    match period {
        Period::Hour => datetime.format("%Y-%m-%dT%H").to_string(),
        Period::Day => datetime.format("%Y-%m-%d").to_string(),
        Period::Month => datetime.format("%Y-%m").to_string(),
    }
}

pub struct PeriodStorage<'a, T, Ser = Json>
where
    T: Serialize + DeserializeOwned + Clone,
    Ser: Serde,
{
    all: Map<'a, u64, Vec<T>, Ser>,
    recent: Item<'a, Vec<u64>>,

    /* keys are date formatted strings "%Y-%m-%dT%h"
     * right-most data is truncated to categorize by higher order
     * e.g. month format is "%Y-%m"
     */
    timed: Map<'a, String, Vec<T>, Ser>,
}

impl<'a, T, Ser> PeriodStorage<'a, T, Ser>
where
    T: Serialize + DeserializeOwned + Clone,
    Ser: Serde,
{
    pub const fn new(all: &'a str, recent: &'a str, timed: &'a str) -> Self {
        PeriodStorage {
            all: Map::new(all),
            recent: Item::new(recent),
            timed: Map::new(timed),
        }
    }

    pub fn load(&self, storage: &dyn Storage, ts: Timestamp) -> StdResult<Vec<T>> {
        self.all.load(storage, ts.seconds())
    }

    pub fn load_period(
        &self,
        storage: &dyn Storage,
        seconds: u64,
        period: Period,
    ) -> StdResult<Vec<T>> {
        Ok(self
            .timed
            .load(storage, map_key(seconds, period))
            .unwrap_or(vec![]))
    }

    pub fn may_load(&self, storage: &dyn Storage, ts: Timestamp) -> StdResult<Vec<T>> {
        Ok(self.all.may_load(storage, ts.seconds())?.unwrap_or(vec![]))
    }

    pub fn push(&self, storage: &mut dyn Storage, ts: Timestamp, item: T) -> StdResult<()> {
        let key = ts.seconds();
        let mut recent = self.recent.may_load(storage)?.unwrap_or(vec![]);
        if !recent.contains(&key) {
            recent.push(key);
            self.recent.save(storage, &recent)?;
        }
        let mut all = self.all.may_load(storage, key)?.unwrap_or(vec![]);
        all.push(item);
        self.all.save(storage, key, &all)
    }

    /* push + flush */
    pub fn pushf(&self, storage: &mut dyn Storage, ts: Timestamp, item: T) -> StdResult<()> {
        self.push(storage, ts, item)?;
        self.flush(storage)
    }

    pub fn appendf(
        &self,
        storage: &mut dyn Storage,
        ts: Timestamp,
        items: &mut Vec<T>,
    ) -> StdResult<()> {
        self.append(storage, ts, items)?;
        self.flush(storage)
    }

    pub fn append(
        &self,
        storage: &mut dyn Storage,
        ts: Timestamp,
        items: &mut Vec<T>,
    ) -> StdResult<()> {
        let key = ts.seconds();
        let mut recent = self.recent.may_load(storage)?.unwrap_or(vec![]);
        if !recent.contains(&key) {
            recent.push(key);
            self.recent.save(storage, &recent)?;
        }
        let mut all = self.all.may_load(storage, key)?.unwrap_or(vec![]);
        all.append(items);
        self.all.save(storage, key, &all)
    }

    /* This will move all "recents" into the time based storage
     * This should likely be called at the end of execution that adds items
     */
    pub fn flush(&self, storage: &mut dyn Storage) -> StdResult<()> {
        for seconds in self.recent.load(storage)? {
            let items = self.all.load(storage, seconds)?;

            for period in Period::iter() {
                let k = map_key(seconds, period);
                let mut cur_items = self.timed.may_load(storage, k.clone())?.unwrap_or(vec![]);
                cur_items.append(&mut items.clone());
                self.timed.save(storage, k, &cur_items)?;
            }
        }
        self.recent.save(storage, &vec![])
    }
}
