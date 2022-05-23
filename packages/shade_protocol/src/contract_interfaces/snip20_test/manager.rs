use cosmwasm_std::{Binary, HumanAddr, StdError, StdResult, Storage};
use schemars::JsonSchema;
use secret_storage_plus::{Item, Map};
use serde::{Deserialize, Serialize};
use cosmwasm_math_compat::Uint128;
use crate::impl_into_u8;
use crate::utils::storage::plus::{ItemStorage, MapStorage, NaiveItemStorage};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatusLevel {
    NormalRun,
    StopAllButRedeems,
    StopAll,
}
impl ContractStatusLevel {
    pub fn save<S: Storage>(self, storage: &mut S) -> StdResult<()> {
        ContractStatus(self.into()).save(storage)
    }
    pub fn load<S: Storage>(storage: &mut S) -> StdResult<Self> {
        let i = ContractStatus::load(storage)?.0;
        let item = match i {
            1 => ContractStatusLevel::NormalRun,
            2 => ContractStatusLevel::StopAllButRedeems,
            3 => ContractStatusLevel::StopAll,
            _ => return Err(StdError::generic_err("Stored enum u8 is greater than enum"))
        };
        Ok(item)
    }
}
impl_into_u8!(ContractStatusLevel);

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct ContractStatus(pub u8);
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

impl ItemStorage for CoinInfo {
    const ITEM: Item<'static, Self> = Item::new("coin-info-");
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Admin(pub HumanAddr);

impl ItemStorage for Admin {
    const ITEM: Item<'static, Self> = Item::new("admin-");
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct RandSeed(pub Vec<u8>);

impl ItemStorage for RandSeed {
    const ITEM: Item<'static, Self> = Item::new("rand-seed-");
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Setting(pub bool);

impl NaiveItemStorage for Setting {}

const PUBLIC_TOTAL_SUPPLY: Item<'static, Setting> = Item::new("public-total-supply-");
const ENABLE_DEPOSIT: Item<'static, Setting> = Item::new("enable-deposit-");
const ENABLE_REDEEM: Item<'static, Setting> = Item::new("enable-redeem-");
const ENABLE_MINT: Item<'static, Setting> = Item::new("enable-mint-");
const ENABLE_BURN: Item<'static, Setting> = Item::new("enable-burn-");
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
impl ItemStorage for TotalSupply {
    const ITEM: Item<'static, Self> = Item::new("total-supply-");
}
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
impl MapStorage<'static, HumanAddr> for Balance {
    const MAP: Map<'static, HumanAddr, Self> = Map::new("balance-");
}
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
            // TODO: impl error
            Err(_) => return Err(StdError::generic_err("Account has no funds"))
        };
        let supply = match subtractee.checked_sub(amount) {
            Ok(supply) => supply,
            // TODO: impl error
            Err(_) => return Err(StdError::generic_err("Account doesnt have enough funds"))
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
impl ItemStorage for Minters {
    const ITEM: Item<'static, Self> = Item::new("minters-");
}

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, Default, JsonSchema)]
pub struct Allowance {
    pub amount: Uint128,
    pub expiration: Option<u64>,
}

#[derive(Serialize, Debug, Deserialize, Clone, PartialEq, Default, JsonSchema)]
pub struct Allowances(pub Vec<Allowance>);
// (Owner, Spender)
impl MapStorage<'static, (HumanAddr, HumanAddr)> for Allowances {
    const MAP: Map<'static, (HumanAddr, HumanAddr), Self> = Map::new("allowances-");
}