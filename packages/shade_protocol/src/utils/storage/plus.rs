use cosmwasm_std::{StdError, StdResult, Storage};
use secret_storage_plus::{Item, Map, PrimaryKey};
use serde::{de::DeserializeOwned, Serialize};

pub trait NaiveItemStorage: Serialize + DeserializeOwned {
    fn load<S: Storage>(storage: &S, item: Item<Self>) -> StdResult<Self> {
        item.load(storage)
    }

    fn may_load<S: Storage>(storage: &S, item: Item<Self>) -> StdResult<Option<Self>> {
        item.may_load(storage)
    }

    fn save<S: Storage>(&self, storage: &mut S, item: Item<Self>) -> StdResult<()> {
        item.save(storage, self)
    }

    fn update<A, E, S: Storage>(&self, storage: &mut S, item: Item<Self>, action: A) -> Result<Self, E>
        where
            A: FnOnce(Self) -> Result<Self, E>,
            E: From<StdError>,
    {
        item.update(storage, action)
    }
}

pub trait ItemStorage: Serialize + DeserializeOwned {
    const ITEM: Item<'static, Self>;

    fn load<S: Storage>(storage: &S) -> StdResult<Self> {
        Self::ITEM.load(storage)
    }

    fn may_load<S: Storage>(storage: &S) -> StdResult<Option<Self>> {
        Self::ITEM.may_load(storage)
    }

    fn save<S: Storage>(&self, storage: &mut S) -> StdResult<()> {
        Self::ITEM.save(storage, self)
    }

    fn update<A, E, S: Storage>(&self, storage: &mut S, action: A) -> Result<Self, E>
    where
        A: FnOnce(Self) -> Result<Self, E>,
        E: From<StdError>,
    {
        Self::ITEM.update(storage, action)
    }
}

pub trait NaiveMapStorage<'a>: Serialize + DeserializeOwned {
    fn load<S: Storage, K: PrimaryKey<'a>>(storage: &S, map: Map<'a, K, Self>, key: K) -> StdResult<Self> {
        map.load(storage, key)
    }

    fn may_load<S: Storage, K: PrimaryKey<'a>>(storage: &S, map: Map<'a, K, Self>, key: K) -> StdResult<Option<Self>> {
        map.may_load(storage, key)
    }

    fn save<S: Storage, K: PrimaryKey<'a>>(&self, storage: &mut S, map: Map<'a, K, Self>, key: K) -> StdResult<()> {
        map.save(storage, key, self)
    }

    fn update<A, E, S: Storage, K: PrimaryKey<'a>>(&self, storage: &mut S, map: Map<'a, K, Self>, key: K, action: A) -> Result<Self, E>
        where
            A: FnOnce(Option<Self>) -> Result<Self, E>,
            E: From<StdError>,
    {
        map.update(storage, key, action)
    }
}

pub trait MapStorage<'a, K: PrimaryKey<'a>>: Serialize + DeserializeOwned {
    const MAP: Map<'static, K, Self>;

    fn load<S: Storage>(storage: &S, key: K) -> StdResult<Self> {
        Self::MAP.load(storage, key)
    }

    fn may_load<S: Storage>(storage: &S, key: K) -> StdResult<Option<Self>> {
        Self::MAP.may_load(storage, key)
    }

    fn save<S: Storage>(&self, storage: &mut S, key: K) -> StdResult<()> {
        Self::MAP.save(storage, key, self)
    }

    fn update<A, E, S: Storage>(&self, storage: &mut S, key: K, action: A) -> Result<Self, E>
    where
        A: FnOnce(Option<Self>) -> Result<Self, E>,
        E: From<StdError>,
    {
        Self::MAP.update(storage, key, action)
    }
}
