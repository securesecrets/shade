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
    ops::{Add, AddAssign, Index, Sub},
};

pub trait Increment {
    fn one() -> Self;
    fn zero() -> Self;
}

macro_rules! impl_increment {
    ($($t:ty)*) => ($(
        impl Increment for $t {
            fn one() -> Self {
                1
            }

            fn zero() -> Self {
                0
            }
        }
    )*)
}

impl_increment! { usize u8 u16 u32 u64 u128 }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IterKey<
    N: Add<N, Output = N> + AddAssign + Sub<N, Output = N> + Clone + Serialize + Increment,
> {
    pub item: N,
}

impl<N> IterKey<N>
where
    N: Add<N, Output = N> + Increment + AddAssign + Sub<N, Output = N> + Clone + Serialize,
{
    pub fn new(item: N) -> Self {
        Self { item }
    }

    pub fn to_bytes(&self) -> StdResult<Vec<u8>> {
        Ok(to_binary(self)?.0)
    }
}

pub struct IterMap<'a, K, T, N, Ser = Json>
where
    T: Serialize + DeserializeOwned,
    K: PrimaryKey<'a> + Prefixer<'a> + KeyDeserialize,
    N: Add<N, Output = N>
        + AddAssign
        + Increment
        + Sub<N, Output = N>
        + Serialize
        + DeserializeOwned
        + Clone,
    Ser: Serde,
{
    storage: Map<'a, (K, Vec<u8>), T>,
    id_storage: Map<'a, K, IterKey<N>>,
    serialization_type: PhantomData<*const Ser>,
}

const PREFIX: &str = "iter-map-size-namespace-";

impl<'a, K, T, N, Ser> IterMap<'a, K, T, N, Ser>
where
    T: Serialize + DeserializeOwned,
    K: PrimaryKey<'a> + Prefixer<'a> + KeyDeserialize,
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
        IterMap {
            storage: Map::new(namespace),
            id_storage: Map::new(size_namespace),
            serialization_type: PhantomData,
        }
    }
}

impl<'a, K, T, N, Ser> IterMap<'a, K, T, N, Ser>
where
    T: Serialize + DeserializeOwned,
    K: PrimaryKey<'a> + Prefixer<'a> + KeyDeserialize,
    N: Add<N, Output = N>
        + AddAssign
        + Increment
        + Sub<N, Output = N>
        + Serialize
        + DeserializeOwned
        + Clone,
    Ser: Serde,
{
    pub fn set(&self, store: &mut dyn Storage, key: K, id: N, data: &T) -> StdResult<()> {
        self.storage
            .save(store, (key, IterKey::new(id).to_bytes()?), data)
    }

    pub fn get(&self, store: &dyn Storage, key: K, id: N) -> StdResult<T> {
        self.storage
            .load(store, (key, IterKey::new(id).to_bytes()?))
    }

    pub fn push(&self, store: &mut dyn Storage, key: K, data: &T) -> StdResult<N> {
        let id = IterKey::new(match self.id_storage.may_load(store, key.clone())? {
            None => N::zero(),
            Some(id) => id.item + N::one(),
        });

        self.storage
            .save(store, (key.clone(), id.to_bytes()?), data)?;

        self.id_storage.save(store, key, &id)?;

        Ok(id.item)
    }

    pub fn pop(&self, store: &mut dyn Storage, key: K) -> StdResult<()> {
        let id = match self.id_storage.may_load(store, key.clone())? {
            None => return Err(StdError::generic_err("Iter map is empty")),
            Some(id) => id,
        };

        self.storage.remove(store, (key.clone(), id.to_bytes()?));

        let new_id = IterKey::new(id.item - N::one());
        self.id_storage.save(store, key, &new_id)?;

        Ok(())
    }

    pub fn size(&'a self, store: &dyn Storage, key: K) -> StdResult<N> {
        Ok(self.id_storage.load(store, key)?.item + N::one())
    }

    pub fn iter_from(
        &'a self,
        store: &'a dyn Storage,
        key: K,
        start_from: N,
    ) -> IndexableIterMap<'a, K, T, N, Ser> {
        IndexableIterMap {
            iter_map: self,
            storage: store,
            key: key.clone(),
            index: start_from,
        }
    }

    pub fn iter(&'a self, store: &'a dyn Storage, key: K) -> IndexableIterMap<'a, K, T, N, Ser> {
        self.iter_from(store, key, N::zero())
    }
}

pub struct IndexableIterMap<'a, K, T, N, Ser>
where
    T: Serialize + DeserializeOwned,
    K: PrimaryKey<'a> + Prefixer<'a> + KeyDeserialize,
    N: Add<N, Output = N>
        + AddAssign
        + Increment
        + Sub<N, Output = N>
        + Serialize
        + DeserializeOwned
        + Clone,
    Ser: Serde,
{
    iter_map: &'a IterMap<'a, K, T, N, Ser>,
    storage: &'a dyn Storage,
    key: K,
    index: N,
}

impl<'a, K, T, N, Ser> IndexableIterMap<'a, K, T, N, Ser>
where
    T: Serialize + DeserializeOwned,
    K: PrimaryKey<'a> + Prefixer<'a> + KeyDeserialize,
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

impl<'a, K, T, N, Ser> Iterator for IndexableIterMap<'a, K, T, N, Ser>
where
    T: Serialize + DeserializeOwned,
    K: PrimaryKey<'a> + Prefixer<'a> + KeyDeserialize,
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
        let item = self
            .iter_map
            .get(self.storage.clone(), self.key.clone(), self.index.clone());

        self.next_index();

        match item {
            Ok(i) => Some(i),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::storage::plus::iter_map::IterMap;
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

        let iter: IterMap<(Addr), Uint64, u64> = IterMap::new_override("TEST", "SIZE-TEST");
    }

    fn generate(size: u8, storage: &mut dyn Storage) -> IterMap<(String), Uint64, u64> {
        let iter: IterMap<(String), Uint64, u64> = IterMap::new_override("TEST", "SIZE-TEST");

        for i in 0..size {
            iter.push(storage, "TESTING".to_string(), &Uint64::new(i as u64))
                .unwrap();
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

        let iter: IterMap<String, Uint64, u64> = IterMap::new_override("TEST", "SIZE-TEST");

        for i in 0..10 {
            iter.push(&mut storage, "TESTING".to_string(), &Uint64::new(i as u64))
                .unwrap();
        }

        iter.pop(&mut storage, "TESTING".to_string()).unwrap();

        assert_eq!(9, iter.size(&storage, "TESTING".to_string()).unwrap());
    }

    #[test]
    fn set() {
        let mut storage = MockStorage::new();

        let iter: IterMap<String, Uint64, u64> = IterMap::new_override("TEST", "SIZE-TEST");

        for i in 0..10 {
            iter.push(&mut storage, "TESTING".to_string(), &Uint64::new(i as u64))
                .unwrap();
        }

        iter.set(&mut storage, "TESTING".to_string(), 3, &Uint64::new(5))
            .unwrap();

        assert_eq!(
            Uint64::new(5),
            iter.get(&storage, "TESTING".to_string(), 3).unwrap()
        )
    }

    #[test]
    fn get() {
        let mut storage = MockStorage::new();

        let iter: IterMap<String, Uint64, u64> = IterMap::new_override("TEST", "SIZE-TEST");

        for i in 0..10 {
            iter.push(&mut storage, "TESTING".to_string(), &Uint64::new(i as u64))
                .unwrap();
        }

        assert_eq!(
            Uint64::new(3),
            iter.get(&storage, "TESTING".to_string(), 3).unwrap()
        )
    }

    #[test]
    fn total() {
        let mut storage = MockStorage::new();

        let iter: IterMap<String, Uint64, u64> = IterMap::new_override("TEST", "SIZE-TEST");

        for i in 0..10 {
            iter.push(&mut storage, "TESTING".to_string(), &Uint64::new(i as u64))
                .unwrap();
        }

        assert_eq!(10, iter.size(&storage, "TESTING".to_string()).unwrap())
    }

    #[test]
    fn iterate() {
        let mut storage = MockStorage::new();

        let iter: IterMap<String, Uint64, u64> = IterMap::new_override("TEST", "SIZE-TEST");

        for i in 0..10 {
            iter.push(&mut storage, "TESTING".to_string(), &Uint64::new(i as u64))
                .unwrap();
        }

        for (i, item) in iter.iter(&storage, "TESTING".to_string()).enumerate() {
            assert_eq!(item, Uint64::new(i as u64))
        }
    }
}
