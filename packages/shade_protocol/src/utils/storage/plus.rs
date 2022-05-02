use cosmwasm_std::{StdError, StdResult, Storage};
use secret_storage_plus::{Item, Map, Prefix, PrimaryKey};
use serde::de::DeserializeOwned;
use serde::Serialize;

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

    fn update<A, E, S: Storage>(&self, storage: &mut S, key: K, action: A
    ) -> Result<Self, E>
        where
            A: FnOnce(Option<Self>) -> Result<Self, E>,
            E: From<StdError>,
    {
        Self::MAP.update(storage, key, action)
    }
}
