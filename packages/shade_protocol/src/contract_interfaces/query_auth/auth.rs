use cosmwasm_std::{Env, HumanAddr, StdResult, Storage};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use query_authentication::viewing_keys::ViewingKey;
use secret_storage_plus::Map;
use secret_toolkit::crypto::{Prng, sha_256};
use crate::utils::storage::plus::MapStorage;

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, Default, JsonSchema)]
pub struct Key(pub String);

impl Key {
    pub fn generate(env: &Env, seed: &[u8], entropy: &[u8]) -> Self {
        // 16 here represents the lengths in bytes of the block height and time.
        let entropy_len = 16 + env.message.sender.len() + entropy.len();
        let mut rng_entropy = Vec::with_capacity(entropy_len);
        rng_entropy.extend_from_slice(&env.block.height.to_be_bytes());
        rng_entropy.extend_from_slice(&env.block.time.to_be_bytes());
        rng_entropy.extend_from_slice(&env.message.sender.0.as_bytes());
        rng_entropy.extend_from_slice(entropy);

        let mut rng = Prng::new(seed, &rng_entropy);

        let rand_slice = rng.rand_bytes();

        let key = sha_256(&rand_slice);

        Self(base64::encode(key))
    }

    pub fn verify<S: Storage>(storage: &S, address: HumanAddr, key: String) -> StdResult<bool> {
        Ok(match HashedKey::may_load(storage, address)? {
            None => {
                // Empty compare for security reasons
                Key(key).compare(&[0u8; KEY_SIZE]);
                false
            }
            Some(hashed) => Key(key).compare(&hashed.0)
        })
    }
}

impl ToString for Key {
    fn to_string(&self) -> String {
        self.0.clone()
    }
}
const KEY_SIZE: usize = 32;
impl ViewingKey<KEY_SIZE> for Key{}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HashedKey(pub [u8; KEY_SIZE]);

impl MapStorage<'static, HumanAddr> for HashedKey {
    const MAP: Map<'static, HumanAddr, Self> = Map::new("hashed-viewing-key-");
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PermitKey(pub bool);

impl MapStorage<'static, (HumanAddr, String)> for PermitKey {
    const MAP: Map<'static, (HumanAddr, String), Self> = Map::new("permit-key-");
}

impl PermitKey {
    pub fn revoke<S: Storage>(storage: &mut S, key: String, user: HumanAddr) -> StdResult<()> {
        PermitKey(true).save(storage, (user, key))
    }

    pub fn is_revoked<S: Storage>(storage: &mut S, key: String, user: HumanAddr) -> StdResult<bool> {
        Ok(match PermitKey::may_load(storage, (user, key))? {
            None => false,
            Some(_) => true
        })
    }
}