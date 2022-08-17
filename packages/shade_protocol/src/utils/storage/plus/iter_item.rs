use crate::utils::storage::plus::iter_map::{Increment, IndexableIterMap, IterKey, IterMap};
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
    storage: Map<'a, Vec<u8>, T>,
    id_storage: Item<'a, IterKey<N>>,
    serialization_type: PhantomData<*const Ser>,
}

const PREFIX: &str = "iter-map-size-namespace-";

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
            storage: Map::new(namespace),
            id_storage: Item::new(size_namespace),
            serialization_type: PhantomData,
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

    pub fn append(&self, store: &mut dyn Storage, data: &mut Vec<T>) -> StdResult<N> {
        let mut id = match self.id_storage.may_load(store)? {
            None => N::zero(),
            Some(id) => id.item,
        };

        for d in data {
            id = id + N::one();
            self.storage
                .save(store, IterKey::new(id.clone()).to_bytes()?, &d)?;
        }

        self.id_storage.save(store, &IterKey::new(id.clone()))?;

        Ok(id)
    }

    pub fn pop(&self, store: &mut dyn Storage) -> StdResult<()> {
        let id = match self.id_storage.may_load(store)? {
            None => return Err(StdError::generic_err("Iter map is empty")),
            Some(id) => id,
        };

        self.storage.remove(store, id.to_bytes()?);

        let new_id = IterKey::new(id.item - N::one());
        self.id_storage.save(store, &new_id)?;

        Ok(())
    }

    pub fn last(&self, store: &mut dyn Storage) -> StdResult<T> {
        let id = match self.id_storage.may_load(store)? {
            None => return Err(StdError::generic_err("Iter map is empty")),
            Some(id) => id,
        };

        self.storage.load(store, id.to_bytes()?)
    }

    pub fn clear(&self, store: &mut dyn Storage) -> StdResult<()> {
        self.id_storage.save(store, &IterKey::new(N::zero()))
    }

    pub fn size(&'a self, store: &dyn Storage) -> StdResult<N> {
        Ok(self.id_storage.load(store)?.item + N::one())
    }

    pub fn iter_from(
        &'a self,
        store: &'a dyn Storage,
        start_from: N,
    ) -> IndexableIterItem<'a, T, N, Ser> {
        IndexableIterItem {
            iter_map: self,
            storage: store,
            index: start_from,
        }
    }

    pub fn iter(&'a self, store: &'a dyn Storage) -> IndexableIterItem<'a, T, N, Ser> {
        self.iter_from(store, N::zero())
    }
}

// Make struct IterMapIndexable and implement the cool stuff there
pub struct IndexableIterItem<'a, T, N, Ser>
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
    iter_map: &'a IterItem<'a, T, N, Ser>,
    storage: &'a dyn Storage,
    index: N,
}

impl<'a, T, N, Ser> IndexableIterItem<'a, T, N, Ser>
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
    fn next_index(&mut self) {
        self.index += N::one();
    }
}

impl<'a, T, N, Ser> Iterator for IndexableIterItem<'a, T, N, Ser>
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

        let iter: IterItem<Uint64, u64> = IterItem::new_override("TEST", "SIZE-TEST");

        for i in 0..10 {
            iter.push(&mut storage, &Uint64::new(i as u64)).unwrap();
        }

        for (i, item) in iter.iter(&storage).enumerate() {
            assert_eq!(item, Uint64::new(i as u64))
        }
    }
}
