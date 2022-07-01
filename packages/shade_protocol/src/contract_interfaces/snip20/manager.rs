use cosmwasm_std::{Binary, Env, HumanAddr, StdError, StdResult, Storage};
use query_authentication::viewing_keys::ViewingKey;
use schemars::JsonSchema;
use secret_toolkit::crypto::{Prng, sha_256};
use serde::{Deserialize, Serialize};
use cosmwasm_math_compat::Uint128;
use crate::contract_interfaces::snip20::errors::{allowance_expired, contract_status_level_invalid, insufficient_allowance, no_funds, not_enough_funds};
use crate::impl_into_u8;
#[cfg(feature = "snip20-impl")]
use crate::utils::storage::plus::{ItemStorage, MapStorage, NaiveItemStorage};
#[cfg(feature = "snip20-impl")]
use secret_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatusLevel {
    NormalRun,
    StopAllButRedeems,
    StopAll,
}

#[cfg(feature = "snip20-impl")]
impl ContractStatusLevel {
    pub fn save<S: Storage>(self, storage: &mut S) -> StdResult<()> {
        ContractStatus(self.into()).save(storage)
    }
    pub fn load<S: Storage>(storage: & S) -> StdResult<Self> {
        let i = ContractStatus::load(storage)?.0;
        let item = match i {
            0 => ContractStatusLevel::NormalRun,
            1 => ContractStatusLevel::StopAllButRedeems,
            2 => ContractStatusLevel::StopAll,
            _ => return Err(contract_status_level_invalid(i))
        };
        Ok(item)
    }
}
impl_into_u8!(ContractStatusLevel);

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ContractStatus(pub u8);

#[cfg(feature = "snip20-impl")]
impl ItemStorage for ContractStatus {
    const ITEM: Item<'static, Self> = Item::new("contract-status-level-");
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct CoinInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

#[cfg(feature = "snip20-impl")]
impl ItemStorage for CoinInfo {
    const ITEM: Item<'static, Self> = Item::new("coin-info-");
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Admin(pub HumanAddr);

#[cfg(feature = "snip20-impl")]
impl ItemStorage for Admin {
    const ITEM: Item<'static, Self> = Item::new("admin-");
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct RandSeed(pub Vec<u8>);

#[cfg(feature = "snip20-impl")]
impl ItemStorage for RandSeed {
    const ITEM: Item<'static, Self> = Item::new("rand-seed-");
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Setting(pub bool);

#[cfg(feature = "snip20-impl")]
impl NaiveItemStorage for Setting {}

#[cfg(feature = "snip20-impl")]
const PUBLIC_TOTAL_SUPPLY: Item<'static, Setting> = Item::new("public-total-supply-");
#[cfg(feature = "snip20-impl")]
const ENABLE_DEPOSIT: Item<'static, Setting> = Item::new("enable-deposit-");
#[cfg(feature = "snip20-impl")]
const ENABLE_REDEEM: Item<'static, Setting> = Item::new("enable-redeem-");
#[cfg(feature = "snip20-impl")]
const ENABLE_MINT: Item<'static, Setting> = Item::new("enable-mint-");
#[cfg(feature = "snip20-impl")]
const ENABLE_BURN: Item<'static, Setting> = Item::new("enable-burn-");
#[cfg(feature = "snip20-impl")]
const ENABLE_TRANSFER: Item<'static, Setting> = Item::new("enable-transfer-");

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub public_total_supply: bool,
    pub enable_deposit: bool,
    pub enable_redeem: bool,
    pub enable_mint: bool,
    pub enable_burn: bool,
    pub enable_transfer: bool,
}

#[cfg(feature = "snip20-impl")]
impl Config {
    pub fn save<S: Storage>(&self, storage: &mut S) -> StdResult<()> {
        Self::set_public_total_supply(storage, self.public_total_supply)?;
        Self::set_deposit_enabled(storage, self.enable_deposit)?;
        Self::set_redeem_enabled(storage, self.enable_redeem)?;
        Self::set_mint_enabled(storage, self.enable_mint)?;
        Self::set_burn_enabled(storage, self.enable_burn)?;
        Self::set_transfer_enabled(storage, self.enable_transfer)?;
        Ok(())
    }

    pub fn public_total_supply<S: Storage>(storage: & S) -> StdResult<bool> {
        Ok(Setting::load(storage, PUBLIC_TOTAL_SUPPLY)?.0)
    }

    pub fn set_public_total_supply<S: Storage>(storage: &mut S, setting: bool) -> StdResult<()> {
        Setting(setting).save(storage, PUBLIC_TOTAL_SUPPLY)?;
        Ok(())
    }

    pub fn deposit_enabled<S: Storage>(storage: & S) -> StdResult<bool> {
        Ok(Setting::load(storage, ENABLE_DEPOSIT)?.0)
    }

    pub fn set_deposit_enabled<S: Storage>(storage: &mut S, setting: bool) -> StdResult<()> {
        Setting(setting).save(storage, ENABLE_DEPOSIT)?;
        Ok(())
    }

    pub fn redeem_enabled<S: Storage>(storage: & S) -> StdResult<bool> {
        Ok(Setting::load(storage, ENABLE_REDEEM)?.0)
    }

    pub fn set_redeem_enabled<S: Storage>(storage: &mut S, setting: bool) -> StdResult<()> {
        Setting(setting).save(storage, ENABLE_REDEEM)?;
        Ok(())
    }

    pub fn mint_enabled<S: Storage>(storage: & S) -> StdResult<bool> {
        Ok(Setting::load(storage, ENABLE_MINT)?.0)
    }

    pub fn set_mint_enabled<S: Storage>(storage: &mut S, setting: bool) -> StdResult<()> {
        Setting(setting).save(storage, ENABLE_MINT)?;
        Ok(())
    }

    pub fn burn_enabled<S: Storage>(storage: & S) -> StdResult<bool> {
        Ok(Setting::load(storage, ENABLE_BURN)?.0)
    }

    pub fn set_burn_enabled<S: Storage>(storage: &mut S, setting: bool) -> StdResult<()> {
        Setting(setting).save(storage, ENABLE_BURN)?;
        Ok(())
    }

    pub fn transfer_enabled<S: Storage>(storage: & S) -> StdResult<bool> {
        Ok(Setting::load(storage, ENABLE_TRANSFER)?.0)
    }

    pub fn set_transfer_enabled<S: Storage>(storage: &mut S, setting: bool) -> StdResult<()> {
        Setting(setting).save(storage, ENABLE_TRANSFER)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct TotalSupply(pub Uint128);

#[cfg(feature = "snip20-impl")]
impl ItemStorage for TotalSupply {
    const ITEM: Item<'static, Self> = Item::new("total-supply-");
}

#[cfg(feature = "snip20-impl")]
impl TotalSupply {
    pub fn set<S: Storage>(storage: &mut S, amount: Uint128) -> StdResult<()> {
        TotalSupply(amount).save(storage)
    }
    pub fn add<S: Storage>(storage: &mut S, amount: Uint128) -> StdResult<Uint128> {
        let supply = TotalSupply::load(storage)?.0.checked_add(amount)?;
        TotalSupply::set(storage, supply)?;
        Ok(supply)
    }
    pub fn sub<S: Storage>(storage: &mut S, amount: Uint128) -> StdResult<Uint128> {
        let supply = TotalSupply::load(storage)?.0.checked_sub(amount)?;
        TotalSupply::set(storage, supply)?;
        Ok(supply)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Balance(pub Uint128);

#[cfg(feature = "snip20-impl")]
impl MapStorage<'static, HumanAddr> for Balance {
    const MAP: Map<'static, HumanAddr, Self> = Map::new("balance-");
}

#[cfg(feature = "snip20-impl")]
impl Balance {
    pub fn set<S: Storage>(storage: &mut S, amount: Uint128, addr: &HumanAddr) -> StdResult<()> {
        Balance(amount).save(storage, addr.clone())
    }
    pub fn add<S: Storage>(storage: &mut S, amount: Uint128, addr: &HumanAddr) -> StdResult<Uint128> {
        let supply = Self::may_load(storage, addr.clone())?
            .unwrap_or(Self(Uint128::zero())).0
            .checked_add(amount)?;

        Balance::set(storage, supply, addr)?;
        Ok(supply)
    }
    pub fn sub<S: Storage>(storage: &mut S, amount: Uint128, addr: &HumanAddr) -> StdResult<Uint128> {
        let subtractee = match Self::load(storage, addr.clone()) {
            Ok(amount) => amount.0,
            Err(_) => return Err(no_funds())
        };
        let supply = match subtractee.checked_sub(amount) {
            Ok(supply) => supply,
            Err(_) => return Err(not_enough_funds())
        };
        Balance::set(storage, supply, addr)?;
        Ok(supply)
    }
    pub fn transfer<S: Storage>(
        storage: &mut S,
        amount: Uint128,
        sender: &HumanAddr,
        recipient: &HumanAddr
    ) -> StdResult<()> {
        Self::sub(storage, amount, sender)?;
        Self::add(storage, amount, recipient)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Minters(pub Vec<HumanAddr>);

#[cfg(feature = "snip20-impl")]
impl ItemStorage for Minters {
    const ITEM: Item<'static, Self> = Item::new("minters-");
}

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct Allowance {
    pub amount: Uint128,
    pub expiration: Option<u64>,
}

impl Default for Allowance {
    fn default() -> Self {
        Self {
            amount: Uint128::zero(),
            expiration: None
        }
    }
}

#[cfg(feature = "snip20-impl")]
impl Allowance {
    pub fn is_expired(&self, block: &cosmwasm_std::BlockInfo) -> bool {
        match self.expiration {
            Some(time) => block.time >= time,
            None => false
        }
    }

    pub fn spend<S: Storage>(
        storage: &mut S,
        owner: &HumanAddr,
        spender: &HumanAddr,
        amount: Uint128,
        block: &cosmwasm_std::BlockInfo
    ) -> StdResult<()> {
        let mut allowance = Allowance::load(storage, (owner.clone(), spender.clone()))?;

        if allowance.is_expired(block) {
            return Err(allowance_expired(allowance.expiration.unwrap()))
        }
        if let Ok(new_allowance) = allowance.amount.checked_sub(amount) {
            allowance.amount = new_allowance;
        } else {
            return Err(insufficient_allowance());
        }

        allowance.save(storage, (owner.clone(), spender.clone()))?;

        Ok(())
    }
}
// (Owner, Spender)
#[cfg(feature = "snip20-impl")]
impl MapStorage<'static, (HumanAddr, HumanAddr)> for Allowance {
    const MAP: Map<'static, (HumanAddr, HumanAddr), Self> = Map::new("allowance-");
}

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, Default, JsonSchema)]
pub struct ReceiverHash(pub String);

#[cfg(feature = "snip20-impl")]
impl MapStorage<'static, HumanAddr> for ReceiverHash {
    const MAP: Map<'static, HumanAddr, Self> = Map::new("receiver-hash-");
}

// Auth
#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, Default, JsonSchema)]
pub struct Key(pub String);

#[cfg(feature = "snip20-impl")]
impl Key {
    // TODO: implement this in query auth instead
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
impl ViewingKey<32> for Key{}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct HashedKey(pub [u8; KEY_SIZE]);

#[cfg(feature = "snip20-impl")]
impl MapStorage<'static, HumanAddr> for HashedKey {
    const MAP: Map<'static, HumanAddr, Self> = Map::new("hashed-viewing-key-");
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PermitKey(pub bool);

#[cfg(feature = "snip20-impl")]
impl MapStorage<'static, (HumanAddr, String)> for PermitKey {
    const MAP: Map<'static, (HumanAddr, String), Self> = Map::new("revoked-permit-");
}

#[cfg(feature = "snip20-impl")]
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