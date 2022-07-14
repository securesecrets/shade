use crate::c_std::{StdResult, Storage};
use crate::storage::{
    bucket,
    bucket_read,
    singleton,
    singleton_read,
    Bucket,
    ReadonlyBucket,
    ReadonlySingleton,
    Singleton,
};
use crate::serde::{de::DeserializeOwned, Serialize};

pub trait NaiveSingletonStorage: Serialize + DeserializeOwned {
    fn read<'a>(storage: &'a dyn Storage, namespace: &'a [u8]) -> ReadonlySingleton<'a, Self> {
        singleton_read(storage, namespace)
    }

    fn load<'a>(storage: &'a dyn Storage, namespace: &'a [u8]) -> StdResult<Self> {
        Self::read(storage, namespace).load()
    }

    fn may_load<'a>(storage: &'a dyn Storage, namespace: &'a [u8]) -> StdResult<Option<Self>> {
        Self::read(storage, namespace).may_load()
    }

    fn write<'a>(storage: &'a mut dyn Storage, namespace: &'a [u8]) -> Singleton<'a, Self> {
        singleton(storage, namespace)
    }

    fn save<'a>(&self, storage: &mut dyn Storage, namespace: &'a [u8]) -> StdResult<()> {
        Self::write(storage, namespace).save(self)
    }
}

pub trait SingletonStorage: Serialize + DeserializeOwned {
    const NAMESPACE: &'static [u8];

    fn read(storage: &dyn Storage) -> ReadonlySingleton<Self> {
        singleton_read(storage, Self::NAMESPACE)
    }

    fn load(storage: &dyn Storage) -> StdResult<Self> {
        Self::read(storage).load()
    }

    fn may_load(storage: &dyn Storage) -> StdResult<Option<Self>> {
        Self::read(storage).may_load()
    }

    fn write(storage: &mut dyn Storage) -> Singleton<Self> {
        singleton(storage, Self::NAMESPACE)
    }

    fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        Self::write(storage).save(self)
    }
}

pub trait NaiveBucketStorage: Serialize + DeserializeOwned {
    fn read<'a>(storage: &'a dyn Storage, namespace: &'a [u8]) -> ReadonlyBucket<'a, Self> {
        bucket_read(storage, namespace)
    }

    fn load<'a>(storage: &'a dyn Storage, namespace: &'a [u8], key: &'a [u8]) -> StdResult<Self> {
        Self::read(storage, namespace).load(key)
    }

    fn may_load<'a>(
        storage: &'a dyn Storage,
        namespace: &'a [u8],
        key: &'a [u8],
    ) -> StdResult<Option<Self>> {
        Self::read(storage, namespace).may_load(key)
    }

    fn write<'a>(storage: &'a mut dyn Storage, namespace: &'a [u8]) -> Bucket<'a, Self> {
        bucket(storage, namespace)
    }

    fn save<'a>(
        &self,
        storage: &mut dyn Storage,
        namespace: &'a [u8],
        key: &'a [u8],
    ) -> StdResult<()> {
        Self::write(storage, namespace).save(key, self)
    }
}

pub trait BucketStorage: Serialize + DeserializeOwned {
    const NAMESPACE: &'static [u8];

    fn read(storage: &dyn Storage) -> ReadonlyBucket<Self> {
        bucket_read(storage, Self::NAMESPACE)
    }

    fn load(storage: &dyn Storage, key: &[u8]) -> StdResult<Self> {
        Self::read(storage).load(key)
    }

    fn may_load(storage: &dyn Storage, key: &[u8]) -> StdResult<Option<Self>> {
        Self::read(storage).may_load(key)
    }

    fn write(storage: &mut dyn Storage) -> Bucket<Self> {
        bucket(storage, Self::NAMESPACE)
    }

    fn save(&self, storage: &mut dyn Storage, key: &[u8]) -> StdResult<()> {
        Self::write(storage).save(key, self)
    }
}
