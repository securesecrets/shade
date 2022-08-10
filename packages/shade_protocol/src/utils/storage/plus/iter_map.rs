use cosmwasm_std::{StdError, StdResult, Storage, Uint128};
use secret_storage_plus::{Item, Json, Key, KeyDeserialize, Map, Prefixer, PrimaryKey, Serde};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    marker::PhantomData,
    ops::{Add, AddAssign, Index, Sub},
};

pub struct IterMap<'a, K, T, N, Ser = Json> {
    storage: Map<'a, (K, N), T>,
    id_storage: Item<'a, N>,
    serialization_type: PhantomData<*const Ser>,
}

const PREFIX: &str = "iter-map-size-namespace-";

impl<'a, K, T, N, Ser> IterMap<'a, K, T, N, Ser>
where
    T: Serialize + DeserializeOwned,
    K: PrimaryKey<'a> + Prefixer<'a> + KeyDeserialize,
    N: Serialize + DeserializeOwned + PrimaryKey<'a> + KeyDeserialize,
    Ser: Serde,
{
    // TODO: gotta figure this out
    // pub const fn new(namespace: &'a str) -> Self {
    //     Self::new_override(namespace, PREFIX.as_bytes() + namespace.as_bytes())
    // }

    pub const fn new_override(namespace: &'a str, size_namespace: &'a str) -> Self {
        IterMap {
            storage: Map::new(namespace),
            id_storage: Item::new(size_namespace),
            serialization_type: PhantomData,
        }
    }
}

impl<'a, K, T, N, Ser> IterMap<'a, K, T, N, Ser>
where
    T: Serialize + DeserializeOwned,
    K: PrimaryKey<'a> + Prefixer<'a> + KeyDeserialize,
    N: Add<N, Output = N>
        + Sub<N, Output = N>
        + Serialize
        + DeserializeOwned
        + Into<u8>
        + From<u8>
        + PrimaryKey<'a>
        + KeyDeserialize,
    Ser: Serde,
{
    pub fn set(&self, store: &mut dyn Storage, key: K, id: N, data: &T) -> StdResult<()> {
        self.storage.save(store, (key, id), data)
    }

    pub fn get(&self, store: &dyn Storage, key: K, id: N) -> StdResult<T> {
        self.storage.load(store, (key, id))
    }

    pub fn push(&self, store: &mut dyn Storage, key: K, data: &T) -> StdResult<N> {
        let id = match self.id_storage.may_load(store)? {
            None => N::from(0),
            Some(id) => id + N::from(1),
        };

        self.storage.save(store, (key, id.clone()), data)?;

        self.id_storage.save(store, &id)?;

        Ok(id)
    }

    pub fn remove(&self, store: &mut dyn Storage, key: K) -> StdResult<()> {
        let id = match self.id_storage.may_load(store)? {
            None => return Err(StdError::generic_err("Iter map is empty")),
            Some(id) => id,
        };

        self.storage.remove(store, (key, id.clone()));

        self.id_storage.save(store, &(id - N::from(1)))?;

        Ok(())
    }
}

// Make struct IterMapIndexable and implement the cool stuff there
pub struct IndexableIterMap<'a, K, T, N, Ser> {
    iter_map: IterMap<'a, K, T, N, Ser>,
    storage: &'a dyn Storage,
    key: K,
    index: N,
}

impl<'a, K, T, N, Ser> IndexableIterMap<'a, K, T, N, Ser>
where
    N: Add<N, Output = N> + AddAssign + Into<u8> + From<u8> + PrimaryKey<'a> + KeyDeserialize,
{
    fn next_index(&mut self) {
        self.index += N::from(1);
    }
}

impl<'a, K, T, N, Ser> Iterator for IndexableIterMap<'a, K, T, N, Ser>
where
    T: Serialize + DeserializeOwned,
    K: PrimaryKey<'a> + Prefixer<'a> + KeyDeserialize,
    N: Add<N, Output = N>
        + AddAssign
        + Sub<N, Output = N>
        + Serialize
        + DeserializeOwned
        + Into<u8>
        + From<u8>
        + PrimaryKey<'a>
        + KeyDeserialize,
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
