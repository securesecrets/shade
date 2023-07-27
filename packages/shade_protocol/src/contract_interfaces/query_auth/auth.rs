use cosmwasm_std::MessageInfo;
use crate::c_std::{Env, Addr, StdResult, Storage};
use cosmwasm_schema::{cw_serde};

use crate::query_authentication::viewing_keys::ViewingKey;
use secret_storage_plus::Map;
use crate::utils::crypto::{Prng, sha_256};
use crate::utils::storage::plus::MapStorage;

#[cw_serde]
pub struct Key(pub String);

impl Key {
    pub fn generate(info: &MessageInfo, env: &Env, seed: &[u8], entropy: &[u8]) -> Self {
        // 16 here represents the lengths in bytes of the block height and time.
        let entropy_len = 16 + info.sender.as_str().len() + entropy.len();
        let mut rng_entropy = Vec::with_capacity(entropy_len);
        rng_entropy.extend_from_slice(&env.block.height.to_be_bytes());
        rng_entropy.extend_from_slice(&env.block.time.seconds().to_be_bytes());
        rng_entropy.extend_from_slice(&info.sender.as_bytes());
        rng_entropy.extend_from_slice(entropy);

        let mut rng = Prng::new(seed, &rng_entropy);

        let rand_slice = rng.rand_bytes();

        let key = sha_256(&rand_slice);

        Self(base64::encode(key))
    }

    pub fn verify(storage: &dyn Storage, address: Addr, key: String) -> StdResult<bool> {
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

#[cw_serde]
pub struct HashedKey(pub [u8; KEY_SIZE]);

impl MapStorage<'static, Addr> for HashedKey {
    const MAP: Map<'static, Addr, Self> = Map::new("hashed-viewing-key-");
}


#[cw_serde]
pub struct PermitKey(pub bool);

impl MapStorage<'static, (Addr, String)> for PermitKey {
    const MAP: Map<'static, (Addr, String), Self> = Map::new("permit-key-");
}

impl PermitKey {
    pub fn revoke(storage: &mut dyn Storage, key: String, user: Addr) -> StdResult<()> {
        PermitKey(true).save(storage, (user, key))
    }

    pub fn is_revoked(storage: &mut dyn Storage, key: String, user: Addr) -> StdResult<bool> {
        Ok(match PermitKey::may_load(storage, (user, key))? {
            None => false,
            Some(_) => true
        })
    }
}