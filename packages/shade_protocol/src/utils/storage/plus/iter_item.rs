use crate::utils::storage::plus::iter_map::{Increment, IndexableIterMap, IterMap};
use cosmwasm_std::{to_binary, StdError, StdResult, Storage, Uint128};
use secret_storage_plus::{Item, Json, Key, KeyDeserialize, Map, Prefixer, PrimaryKey, Serde};
use serde::{
    de::{self, DeserializeOwned},
    ser,
    Deserialize,
    Serialize,
};
use std::{
    marker::PhantomData,
    ops::{Add, AddAssign, Index, Sub, SubAssign},
};

const KEY: &str = "ITER-ITEM-KEY-";

pub struct IterItem<'a, T, N, Ser = Json>
where
    T: Serialize + DeserializeOwned,
    N: Add<N, Output = N>
        + AddAssign
        + Increment
        + Sub<N, Output = N>
        + Serialize
        + DeserializeOwned
        + Clone,
    Ser: Serde,
{
    iter_map: IterMap<'a, &'static str, T, N, Ser>,
}

impl<'a, T, N, Ser> IterItem<'a, T, N, Ser>
where
    T: Serialize + DeserializeOwned,
    N: Add<N, Output = N>
        + AddAssign
        + Increment
        + Sub<N, Output = N>
        + Serialize
        + DeserializeOwned
        + Clone,
    Ser: Serde,
{
    // TODO: gotta figure this out
    // pub const fn new(namespace: &'a str) -> Self {
    //     Self::new_override(namespace, PREFIX.as_bytes() + namespace.as_bytes())
    // }

    pub const fn new_override(namespace: &'a str, size_namespace: &'a str) -> Self {
        IterItem {
            iter_map: IterMap::new_override(namespace, size_namespace),
        }
    }
}

impl<'a, T, N, Ser> IterItem<'a, T, N, Ser>
where
    T: Serialize + DeserializeOwned,
    N: Add<N, Output = N>
        + AddAssign
        + Increment
        + Sub<N, Output = N>
        + Serialize
        + DeserializeOwned
        + Clone,
    Ser: Serde,
{
    pub fn set(&self, store: &mut dyn Storage, id: N, data: &T) -> StdResult<()> {
        self.iter_map.set(store, KEY, id, data)
    }

    pub fn get(&self, store: &dyn Storage, id: N) -> StdResult<T> {
        self.iter_map.get(store, KEY, id)
    }

    pub fn push(&self, store: &mut dyn Storage, data: &T) -> StdResult<N> {
        self.iter_map.push(store, KEY, data)
    }

    pub fn remove(&self, store: &mut dyn Storage) -> StdResult<()> {
        self.iter_map.remove(store, KEY)
    }

    pub fn size(&'a self, store: &dyn Storage) -> StdResult<N> {
        self.iter_map.size(store, KEY)
    }

    pub fn iter_from(
        &'a self,
        store: &'a dyn Storage,
        start_from: N,
    ) -> IndexableIterMap<'a, &str, T, N, Ser> {
        self.iter_map.iter_from(store, KEY, start_from)
    }

    pub fn iter(&'a self, store: &'a dyn Storage) -> IndexableIterMap<'a, &str, T, N, Ser> {
        self.iter_map.iter(store, KEY)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::storage::plus::iter_item::IterItem;
    use cosmwasm_std::{
        testing::{MockApi, MockQuerier, MockStorage},
        Addr,
        CustomQuery,
        OwnedDeps,
        Storage,
        Uint64,
    };
    use serde::{
        de::{self, DeserializeOwned},
        ser,
        Deserialize,
        Serialize,
    };
    use std::marker::PhantomData;

    #[derive(Clone, Serialize, Deserialize)]
    struct MyQuery;
    impl CustomQuery for MyQuery {}

    #[test]
    fn initialization() {
        let mut storage = MockStorage::new();

        let iter: IterItem<Uint64, u64> = IterItem::new_override("TEST", "SIZE-TEST");
    }

    fn generate(size: u8, storage: &mut dyn Storage) -> IterItem<Uint64, u64> {
        let iter: IterItem<Uint64, u64> = IterItem::new_override("TEST", "SIZE-TEST");

        for i in 0..size {
            iter.push(storage, &Uint64::new(i as u64)).unwrap();
        }

        iter
    }

    #[test]
    fn push() {
        let mut storage = MockStorage::new();

        generate(10, &mut storage);
    }

    #[test]
    fn pop() {
        let mut storage = MockStorage::new();

        generate(10, &mut storage);
    }

    #[test]
    fn set() {
        let mut storage = MockStorage::new();

        let iter: IterItem<Uint64, u64> = IterItem::new_override("TEST", "SIZE-TEST");

        for i in 0..10 {
            iter.push(&mut storage, &Uint64::new(i as u64)).unwrap();
        }

        iter.set(&mut storage, 3, &Uint64::new(5)).unwrap();

        assert_eq!(Uint64::new(5), iter.get(&storage, 3).unwrap())
    }

    #[test]
    fn get() {
        let mut storage = MockStorage::new();

        let iter: IterItem<Uint64, u64> = IterItem::new_override("TEST", "SIZE-TEST");

        for i in 0..10 {
            iter.push(&mut storage, &Uint64::new(i as u64)).unwrap();
        }

        assert_eq!(Uint64::new(3), iter.get(&storage, 3).unwrap())
    }

    #[test]
    fn total() {
        let mut storage = MockStorage::new();

        let iter: IterItem<Uint64, u64> = IterItem::new_override("TEST", "SIZE-TEST");

        for i in 0..10 {
            iter.push(&mut storage, &Uint64::new(i as u64)).unwrap();
        }

        assert_eq!(10, iter.size(&storage).unwrap())
    }

    #[test]
    fn iterate() {
        let mut storage = MockStorage::new();

        let iter: IterItem<String, Uint64, u64> = IterItem::new_override("TEST", "SIZE-TEST");

        for i in 0..10 {
            iter.push(&mut storage, &Uint64::new(i as u64)).unwrap();
        }

        for (i, item) in iter.iter(&storage).enumerate() {
            assert_eq!(item, Uint64::new(i as u64))
        }
    }
}
