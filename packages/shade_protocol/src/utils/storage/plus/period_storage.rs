use crate::{
    c_std::{StdError, StdResult, Storage, Timestamp},
    serde::{de::DeserializeOwned, Serialize},
    utils::cycle::*,
};
use chrono::prelude::*;
pub use secret_storage_plus::{Item, Json, Map, PrimaryKey, Serde};

use super::iter_item::IterItem;

pub enum Period {
    Hour,
    Day,
    Month,
}

pub fn map_key(seconds: u64, period: Period) -> String {
    let datetime = utc_from_seconds(seconds as i64);
    match period {
        Period::Hour => datetime.format("%Y-%m-%dT%h").to_string(),
        Period::Day => datetime.format("%Y-%m-%d").to_string(),
        Period::Month => datetime.format("%Y-%m").to_string(),
    }
}

pub struct PeriodStorage<'a, T, Ser = Json>
where
    T: Serialize + DeserializeOwned,
    Ser: Serde,
{
    all: Map<'a, u64, Vec<T>, Ser>,
    recent: Item<'a, Vec<u64>>,

    /* keys are date formatted strings "%Y-%m-%dT%h"
     * right-most data is truncated to categorize by higher order
     * e.g. month format is "%Y-%m"
     */
    hour: Map<'a, String, Vec<T>, Ser>,
    day: Map<'a, String, Vec<T>, Ser>,
    month: Map<'a, String, Vec<T>, Ser>,
}

impl<'a, T, Ser> PeriodStorage<'a, T, Ser>
where
    T: Serialize + DeserializeOwned,
    Ser: Serde,
{
    fn load(&self, storage: &dyn Storage, ts: Timestamp) -> StdResult<Vec<T>> {
        self.all.load(storage, ts.seconds())
    }

    fn load_period(
        &self,
        storage: &dyn Storage,
        ts: Timestamp,
        period: Period,
    ) -> StdResult<Vec<T>> {
        match period {
            Period::Hour => self.hour.load(storage, map_key(ts.seconds(), period)),
            Period::Day => self.day.load(storage, map_key(ts.seconds(), period)),
            Period::Month => self.month.load(storage, map_key(ts.seconds(), period)),
        }
    }

    fn may_load(&self, storage: &dyn Storage, ts: Timestamp) -> StdResult<Vec<T>> {
        Ok(self.all.may_load(storage, ts.seconds())?.unwrap_or(vec![]))
    }

    fn push(&self, storage: &mut dyn Storage, ts: Timestamp, item: T) -> StdResult<()> {
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

    fn append(
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

    fn flush(&self, storage: &mut dyn Storage) -> StdResult<()> {
        for seconds in self.recent.load(storage)? {
            let mut items = self.all.load(storage, seconds)?;

            let k = map_key(seconds, Period::Hour);
            let mut cur_items = self.hour.load(storage, k.clone())?;
            cur_items.append(&mut items);
            self.hour.save(storage, k, &cur_items)?;

            let k = map_key(seconds, Period::Day);
            let mut cur_items = self.day.load(storage, k.clone())?;
            cur_items.append(&mut items);
            self.day.save(storage, k, &cur_items)?;

            let k = map_key(seconds, Period::Month);
            let mut cur_items = self.month.load(storage, k.clone())?;
            cur_items.append(&mut items);
            self.month.save(storage, k, &cur_items)?;
        }
        self.recent.save(storage, &vec![])
    }
}
