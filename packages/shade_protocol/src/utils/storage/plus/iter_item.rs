use crate::utils::storage::plus::iter_map::{Increment, IterKey};
use cosmwasm_std::{StdError, StdResult, Storage};
use secret_storage_plus::{Item, Map};
use serde::{de::DeserializeOwned, Serialize};
use std::ops::{Add, AddAssign, Sub};

pub struct IterItem<'a, T, N>
where
    T: Serialize + DeserializeOwned,
    N: Add<N, Output = N>
        + AddAssign
        + Increment
        + Sub<N, Output = N>
        + Serialize
        + DeserializeOwned
        + Clone,
{
    storage: Map<'a, Vec<u8>, T>,
    id_storage: Item<'a, IterKey<N>>,
}

#[macro_export]
macro_rules! new_iter_item {
    ($StoragePath:tt) => {
        IterItem::new_override(
            $StoragePath,
            concat!("iter-item-size-namespace-", $StoragePath),
        )
    };
}

impl<'a, T, N> IterItem<'a, T, N>
where
    T: Serialize + DeserializeOwned,
    N: Add<N, Output = N>
        + AddAssign
        + Increment
        + Sub<N, Output = N>
        + Serialize
        + DeserializeOwned
        + Clone,
{
    pub const fn new_override(namespace: &'a str, size_namespace: &'a str) -> Self {
        IterItem {
            storage: Map::new(namespace),
            id_storage: Item::new(size_namespace),
        }
    }
}

impl<'a, T, N> IterItem<'a, T, N>
where
    T: Serialize + DeserializeOwned,
    N: Add<N, Output = N>
        + AddAssign
        + Increment
        + Sub<N, Output = N>
        + Serialize
        + DeserializeOwned
        + Clone,
{
    pub fn set(&self, store: &mut dyn Storage, id: N, data: &T) -> StdResult<()> {
        self.storage.save(store, IterKey::new(id).to_bytes()?, data)
    }

    pub fn get(&self, store: &dyn Storage, id: N) -> StdResult<T> {
        self.storage.load(store, IterKey::new(id).to_bytes()?)
    }

    pub fn push(&self, store: &mut dyn Storage, data: &T) -> StdResult<N> {
        let id = IterKey::new(match self.id_storage.may_load(store)? {
            None => N::zero(),
            Some(id) => id.item + N::one(),
        });

        self.storage.save(store, id.to_bytes()?, data)?;

        self.id_storage.save(store, &id)?;

        Ok(id.item)
    }

    pub fn remove(&self, store: &mut dyn Storage) -> StdResult<()> {
        let id = match self.id_storage.may_load(store)? {
            None => return Err(StdError::generic_err("Iter map is empty")),
            Some(id) => id,
        };

        self.storage.remove(store, id.to_bytes()?);

        let new_id = IterKey::new(id.item - N::one());
        self.id_storage.save(store, &new_id)?;

        Ok(())
    }

    pub fn pop(&self, store: &mut dyn Storage) -> StdResult<T> {
        let id = match self.id_storage.may_load(store)? {
            None => return Err(StdError::generic_err("Iter map is empty")),
            Some(id) => id,
        };

        let item = self.storage.load(store, id.to_bytes()?)?;
        self.storage.remove(store, id.to_bytes()?);

        let new_id = IterKey::new(id.item - N::one());
        self.id_storage.save(store, &new_id)?;

        Ok(item)
    }

    pub fn size(&'a self, store: &dyn Storage) -> StdResult<N> {
        Ok(match self.id_storage.may_load(store)? {
            None => N::zero(),
            Some(i) => i.item + N::one(),
        })
    }

    pub fn iter_from(
        &'a self,
        store: &'a dyn Storage,
        start_from: N,
    ) -> IndexableIterItem<'a, T, N> {
        IndexableIterItem {
            iter_map: self,
            storage: store,
            index: start_from,
        }
    }

    pub fn iter(&'a self, store: &'a dyn Storage) -> IndexableIterItem<'a, T, N> {
        self.iter_from(store, N::zero())
    }
}

// Make struct IterMapIndexable and implement the cool stuff there
pub struct IndexableIterItem<'a, T, N>
where
    T: Serialize + DeserializeOwned,
    N: Add<N, Output = N>
        + AddAssign
        + Increment
        + Sub<N, Output = N>
        + Serialize
        + DeserializeOwned
        + Clone,
{
    iter_map: &'a IterItem<'a, T, N>,
    storage: &'a dyn Storage,
    index: N,
}

impl<'a, T, N> IndexableIterItem<'a, T, N>
where
    T: Serialize + DeserializeOwned,
    N: Add<N, Output = N>
        + AddAssign
        + Increment
        + Sub<N, Output = N>
        + Serialize
        + DeserializeOwned
        + Clone,
{
    fn next_index(&mut self) {
        self.index += N::one();
    }
}

impl<'a, T, N> Iterator for IndexableIterItem<'a, T, N>
where
    T: Serialize + DeserializeOwned,
    N: Add<N, Output = N>
        + AddAssign
        + Increment
        + Sub<N, Output = N>
        + Serialize
        + DeserializeOwned
        + Clone,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let item = self.iter_map.get(self.storage.clone(), self.index.clone());

        self.next_index();

        match item {
            Ok(i) => Some(i),
            Err(_) => None,
        }
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

    const MACRO_TEST: IterItem<Uint64, u64> = new_iter_item!("MACRO_TEST");

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
    fn remove() {
        let mut storage = MockStorage::new();

        let iter: IterItem<Uint64, u64> = IterItem::new_override("TEST", "SIZE-TEST");

        for i in 0..10 {
            iter.push(&mut storage, &Uint64::new(i as u64)).unwrap();
        }

        let item = iter.remove(&mut storage).unwrap();

        assert_eq!(9, iter.size(&storage).unwrap());
    }

    #[test]
    fn pop() {
        let mut storage = MockStorage::new();

        let iter: IterItem<Uint64, u64> = IterItem::new_override("TEST", "SIZE-TEST");

        for i in 0..10 {
            iter.push(&mut storage, &Uint64::new(i as u64)).unwrap();
        }

        let item = iter.pop(&mut storage).unwrap();

        assert_eq!(item, Uint64::new(9));
        assert_eq!(9, iter.size(&storage).unwrap());
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

        let iter: IterItem<Uint64, u64> = IterItem::new_override("TEST", "SIZE-TEST");

        for i in 0..10 {
            iter.push(&mut storage, &Uint64::new(i as u64)).unwrap();
        }

        for (i, item) in iter.iter(&storage).enumerate() {
            assert_eq!(item, Uint64::new(i as u64))
        }
    }
}
