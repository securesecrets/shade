use cosmwasm_std::{StdResult, Storage};
use cosmwasm_storage::{
    bucket,
    bucket_read,
    singleton,
    singleton_read,
    Bucket,
    ReadonlyBucket,
    ReadonlySingleton,
    Singleton,
};
use serde::{de::DeserializeOwned, Serialize};

pub trait NaiveSingletonStorage: Serialize + DeserializeOwned {
    fn read<'a, S: Storage>(storage: &'a S, namespace: &'a [u8]) -> ReadonlySingleton<'a, S, Self> {
        singleton_read(storage, namespace)
    }

    fn load<'a, S: Storage>(storage: &'a S, namespace: &'a [u8]) -> StdResult<Self> {
        Self::read(storage, namespace).load()
    }

    fn may_load<'a, S: Storage>(storage: &'a S, namespace: &'a [u8]) -> StdResult<Option<Self>> {
        Self::read(storage, namespace).may_load()
    }

    fn write<'a, S: Storage>(storage: &'a mut S, namespace: &'a [u8]) -> Singleton<'a, S, Self> {
        singleton(storage, namespace)
    }

    fn save<'a, S: Storage>(&self, storage: &'a mut S, namespace: &'a [u8]) -> StdResult<()> {
        Self::write(storage, namespace).save(self)
    }
}

pub trait SingletonStorage: Serialize + DeserializeOwned {
    const NAMESPACE: &'static [u8];

    fn read<S: Storage>(storage: &S) -> ReadonlySingleton<S, Self> {
        singleton_read(storage, Self::NAMESPACE)
    }

    fn load<S: Storage>(storage: &S) -> StdResult<Self> {
        Self::read(storage).load()
    }

    fn may_load<S: Storage>(storage: &S) -> StdResult<Option<Self>> {
        Self::read(storage).may_load()
    }

    fn write<S: Storage>(storage: &mut S) -> Singleton<S, Self> {
        singleton(storage, Self::NAMESPACE)
    }

    fn save<S: Storage>(&self, storage: &mut S) -> StdResult<()> {
        Self::write(storage).save(self)
    }
}

pub trait NaiveBucketStorage: Serialize + DeserializeOwned {
    fn read<'a, S: Storage>(storage: &'a S, namespace: &'a [u8]) -> ReadonlyBucket<'a, S, Self> {
        bucket_read(namespace, storage)
    }

    fn load<'a, S: Storage>(storage: &'a S, namespace: &'a [u8], key: &'a [u8]) -> StdResult<Self> {
        Self::read(storage, namespace).load(key)
    }

    fn may_load<'a, S: Storage>(
        storage: &'a S,
        namespace: &'a [u8],
        key: &'a [u8],
    ) -> StdResult<Option<Self>> {
        Self::read(storage, namespace).may_load(key)
    }

    fn write<'a, S: Storage>(storage: &'a mut S, namespace: &'a [u8]) -> Bucket<'a, S, Self> {
        bucket(namespace, storage)
    }

    fn save<'a, S: Storage>(
        &self,
        storage: &'a mut S,
        namespace: &'a [u8],
        key: &'a [u8],
    ) -> StdResult<()> {
        Self::write(storage, namespace).save(key, self)
    }
}

pub trait BucketStorage: Serialize + DeserializeOwned {
    const NAMESPACE: &'static [u8];

    fn read<S: Storage>(storage: &S) -> ReadonlyBucket<S, Self> {
        bucket_read(Self::NAMESPACE, storage)
    }

    fn load<S: Storage>(storage: &S, key: &[u8]) -> StdResult<Self> {
        Self::read(storage).load(key)
    }

    fn may_load<S: Storage>(storage: &S, key: &[u8]) -> StdResult<Option<Self>> {
        Self::read(storage).may_load(key)
    }

    fn write<S: Storage>(storage: &mut S) -> Bucket<S, Self> {
        bucket(Self::NAMESPACE, storage)
    }

    fn save<S: Storage>(&self, storage: &mut S, key: &[u8]) -> StdResult<()> {
        Self::write(storage).save(key, self)
    }
}
