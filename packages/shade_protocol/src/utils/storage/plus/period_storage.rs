use crate::{
    c_std::{StdResult, Storage, Timestamp},
    cosmwasm_schema::cw_serde,
    serde::{de::DeserializeOwned, Serialize},
    utils::cycle::*,
};
pub use secret_storage_plus::{Item, Json, Map, PrimaryKey, Serde};


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
        self.all.save(storage, key, &all)?;
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
        self.all.save(storage, key, &all)?;

        self.flush(storage)
    }

    /* This will move all "recents" into the time based storage
     * This should likely be called at the end of execution that adds items
     */
    fn flush(&self, storage: &mut dyn Storage) -> StdResult<()> {
        for seconds in self.recent.load(storage)? {
            let mut items = self.all.load(storage, seconds)?;

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

#[cfg(test)]
mod test {

    use super::*;
    use crate::c_std::{MemoryStorage, Timestamp, Uint128};

    fn test_push(now: String) {
        let now = parse_utc_datetime(&"1995-11-13T00:00:00.00Z".to_string()).unwrap();
        let mut storage = MemoryStorage::new();
        pub const STORAGE: PeriodStorage<u128> = PeriodStorage::new("all", "recent", "timed");

        let data = vec![1, 2, 3, 5, 10];

        for d in data.clone() {
            STORAGE
                .push(
                    &mut storage,
                    Timestamp::from_seconds(now.timestamp() as u64),
                    d,
                )
                .unwrap();
        }
        assert_eq!(
            STORAGE
                .load_period(&storage, now.timestamp() as u64, Period::Hour)
                .unwrap(),
            data
        );
        assert_eq!(
            STORAGE
                .load_period(&storage, now.timestamp() as u64, Period::Day)
                .unwrap(),
            data
        );
        assert_eq!(
            STORAGE
                .load_period(&storage, now.timestamp() as u64, Period::Month)
                .unwrap(),
            data
        );
    }

    fn test_append(now: String) {
        let now = parse_utc_datetime(&"1995-11-13T00:00:00.00Z".to_string()).unwrap();
        let mut storage = MemoryStorage::new();
        pub const STORAGE: PeriodStorage<u128> = PeriodStorage::new("all", "recent", "timed");

        let mut data = vec![1, 2, 3, 5, 10];

        STORAGE
            .append(
                &mut storage,
                Timestamp::from_seconds(now.timestamp() as u64),
                &mut data.clone(),
            )
            .unwrap();
        assert_eq!(
            STORAGE
                .load_period(&storage, now.timestamp() as u64, Period::Hour)
                .unwrap(),
            data
        );
        assert_eq!(
            STORAGE
                .load_period(&storage, now.timestamp() as u64, Period::Day)
                .unwrap(),
            data
        );
        assert_eq!(
            STORAGE
                .load_period(&storage, now.timestamp() as u64, Period::Month)
                .unwrap(),
            data
        );
    }

    fn test_hour_timed(now: String) {
        let mut now = parse_utc_datetime(&"1995-11-13T00:00:00.00Z".to_string()).unwrap();

        let mut storage = MemoryStorage::new();
        pub const STORAGE: PeriodStorage<u128> = PeriodStorage::new("all", "recent", "timed");

        let mut data = vec![1, 2, 3, 5, 10];
        let mut added = vec![11, 12, 13, 15, 20];

        STORAGE
            .append(
                &mut storage,
                Timestamp::from_seconds(now.timestamp() as u64),
                &mut data,
            )
            .unwrap();
        assert_eq!(
            STORAGE
                .load_period(&storage, now.timestamp() as u64, Period::Hour)
                .unwrap(),
            data
        );

        let now = parse_utc_datetime(&"1995-11-13T01:00:00.00Z".to_string()).unwrap();
        assert!(
            STORAGE
                .load_period(&storage, now.timestamp() as u64, Period::Hour)
                .unwrap()
                .is_empty(),
        );

        STORAGE
            .append(
                &mut storage,
                Timestamp::from_seconds(now.timestamp() as u64),
                &mut added,
            )
            .unwrap();
        assert_eq!(
            STORAGE
                .load_period(&storage, now.timestamp() as u64, Period::Hour)
                .unwrap(),
            added
        );

        let mut all_data = data;
        all_data.append(&mut added);
        assert_eq!(
            STORAGE
                .load_period(&storage, now.timestamp() as u64, Period::Day)
                .unwrap(),
            all_data
        );
    }

    fn test_day_timed(now: String) {
        let mut now = parse_utc_datetime(&"1995-11-13T00:00:00.00Z".to_string()).unwrap();

        let mut storage = MemoryStorage::new();
        pub const STORAGE: PeriodStorage<u128> = PeriodStorage::new("all", "recent", "timed");

        let mut data = vec![1, 2, 3, 5, 10];
        let mut added = vec![11, 12, 13, 15, 20];

        STORAGE
            .append(
                &mut storage,
                Timestamp::from_seconds(now.timestamp() as u64),
                &mut data,
            )
            .unwrap();
        assert_eq!(
            STORAGE
                .load_period(&storage, now.timestamp() as u64, Period::Day)
                .unwrap(),
            data
        );

        let now = parse_utc_datetime(&"1995-11-14T00:00:00.00Z".to_string()).unwrap();
        assert!(
            STORAGE
                .load_period(&storage, now.timestamp() as u64, Period::Day)
                .unwrap()
                .is_empty(),
        );

        STORAGE
            .append(
                &mut storage,
                Timestamp::from_seconds(now.timestamp() as u64),
                &mut added,
            )
            .unwrap();
        assert_eq!(
            STORAGE
                .load_period(&storage, now.timestamp() as u64, Period::Day)
                .unwrap(),
            added
        );

        let mut all_data = data;
        all_data.append(&mut added);
        assert_eq!(
            STORAGE
                .load_period(&storage, now.timestamp() as u64, Period::Month)
                .unwrap(),
            all_data
        );
    }

    fn test_month_timed(now: String) {
        let mut now = parse_utc_datetime(&"1995-11-13T00:00:00.00Z".to_string()).unwrap();

        let mut storage = MemoryStorage::new();
        pub const STORAGE: PeriodStorage<u128> = PeriodStorage::new("all", "recent", "timed");

        let mut data = vec![1, 2, 3, 5, 10];
        let mut added = vec![11, 12, 13, 15, 20];

        STORAGE
            .append(
                &mut storage,
                Timestamp::from_seconds(now.timestamp() as u64),
                &mut data,
            )
            .unwrap();
        assert_eq!(
            STORAGE
                .load_period(&storage, now.timestamp() as u64, Period::Month)
                .unwrap(),
            data
        );

        let now = parse_utc_datetime(&"1995-12-13T00:00:00.00Z".to_string()).unwrap();
        assert!(
            STORAGE
                .load_period(&storage, now.timestamp() as u64, Period::Month)
                .unwrap()
                .is_empty(),
        );

        STORAGE
            .append(
                &mut storage,
                Timestamp::from_seconds(now.timestamp() as u64),
                &mut added,
            )
            .unwrap();
        assert_eq!(
            STORAGE
                .load_period(&storage, now.timestamp() as u64, Period::Month)
                .unwrap(),
            added
        );
    }
}
