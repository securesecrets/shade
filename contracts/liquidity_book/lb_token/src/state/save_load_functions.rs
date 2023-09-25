use std::{
    any::type_name,
    // collections::HashSet,
};

use serde::{de::DeserializeOwned, Serialize};

use cosmwasm_std::{
    Storage, 
    StdResult, StdError,
};

use secret_toolkit::{
    serialization::{Json, Serde}, //Bincode2 
};


// /////////////////////////////////////////////////////////////////////////////////
// // Save and load functions
// /////////////////////////////////////////////////////////////////////////////////

/// Returns StdResult<()> resulting from saving an item to storage using Json (de)serialization
/// because bincode2 annoyingly uses a float op when deserializing an enum
///
/// # Arguments
///
/// * `storage` - a mutable reference to the storage this item should go to
/// * `key` - a byte slice representing the key to access the stored item
/// * `value` - a reference to the item to store
pub fn json_save<T: Serialize>(
    storage: &mut dyn Storage,
    key: &[u8],
    value: &T,
) -> StdResult<()> {
    storage.set(key, &Json::serialize(value)?);
    Ok(())
}

/// Returns StdResult<T> from retrieving the item with the specified key using Json
/// (de)serialization because bincode2 annoyingly uses a float op when deserializing an enum.  
/// Returns a StdError::NotFound if there is no item with that key
///
/// # Arguments
///
/// * `storage` - a reference to the storage this item is in
/// * `key` - a byte slice representing the key that accesses the stored item
pub fn json_load<T: DeserializeOwned>(storage: &dyn Storage, key: &[u8]) -> StdResult<T> {
    Json::deserialize(
        &storage
            .get(key)
            .ok_or_else(|| StdError::not_found(type_name::<T>()))?,
    )
}

// /// Returns StdResult<Option<T>> from retrieving the item with the specified key using Json
// /// (de)serialization because bincode2 annoyingly uses a float op when deserializing an enum.
// /// Returns Ok(None) if there is no item with that key
// ///
// /// # Arguments
// ///
// /// * `storage` - a reference to the storage this item is in
// /// * `key` - a byte slice representing the key that accesses the stored item
// pub fn json_may_load<T: DeserializeOwned, S: ReadonlyStorage>(
//     storage: &S,
//     key: &[u8],
// ) -> StdResult<Option<T>> {
//     match storage.get(key) {
//         Some(value) => Json::deserialize(&value).map(Some),
//         None => Ok(None),
//     }
// }

