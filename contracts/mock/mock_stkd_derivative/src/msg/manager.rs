use shade_protocol::c_std::{Addr, BlockInfo, StdResult, Storage, Timestamp};

use shade_protocol::utils::storage::plus::{ItemStorage, MapStorage, NaiveItemStorage};
use shade_protocol::{
    c_std::Uint128,
    impl_into_u8,
    Contract,
};
use crate::msg::errors::{
    allowance_expired,
    contract_status_level_invalid,
    insufficient_allowance,
    no_funds,
    not_enough_funds,
};
use cosmwasm_schema::cw_serde;
use shade_protocol::secret_storage_plus::{Item, Map};

#[cw_serde]
#[repr(u8)]
pub enum ContractStatusLevel {
    NormalRun,
    StopAllButRedeems,
    StopAll,
}

impl ContractStatusLevel {
    pub fn save(self, storage: &mut dyn Storage) -> StdResult<()> {
        ContractStatus(self.into()).save(storage)
    }

    pub fn load(storage: &dyn Storage) -> StdResult<Self> {
        let i = ContractStatus::load(storage)?.0;
        let item = match i {
            0 => ContractStatusLevel::NormalRun,
            1 => ContractStatusLevel::StopAllButRedeems,
            2 => ContractStatusLevel::StopAll,
            _ => return Err(contract_status_level_invalid(i)),
        };
        Ok(item)
    }
}
impl_into_u8!(ContractStatusLevel);

// TODO: group all of these snip20-impl features into its own package

#[cw_serde]
pub struct ContractStatus(pub u8);

impl ItemStorage for ContractStatus {
    const ITEM: Item<'static, Self> = Item::new("contract-status-level-");
}

#[cw_serde]
pub struct CoinInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

impl ItemStorage for CoinInfo {
    const ITEM: Item<'static, Self> = Item::new("coin-info-");
}

#[cw_serde]
pub struct Admin(pub Addr);

impl ItemStorage for Admin {
    const ITEM: Item<'static, Self> = Item::new("admin-");
}

#[cw_serde]
pub struct QueryAuth(pub Contract);

impl ItemStorage for QueryAuth {
    const ITEM: Item<'static, Self> = Item::new("query_auth-");
}

#[cw_serde]
pub struct RandSeed(pub Vec<u8>);

impl ItemStorage for RandSeed {
    const ITEM: Item<'static, Self> = Item::new("rand-seed-");
}

#[cw_serde]
pub struct Setting(pub bool);

impl NaiveItemStorage for Setting {}

const PUBLIC_TOTAL_SUPPLY: Item<'static, Setting> = Item::new("public-total-supply-");
const ENABLE_DEPOSIT: Item<'static, Setting> = Item::new("enable-deposit-");
const ENABLE_REDEEM: Item<'static, Setting> = Item::new("enable-redeem-");
const ENABLE_MINT: Item<'static, Setting> = Item::new("enable-mint-");
const ENABLE_BURN: Item<'static, Setting> = Item::new("enable-burn-");
const ENABLE_TRANSFER: Item<'static, Setting> = Item::new("enable-transfer-");

#[cw_serde]
pub struct Config {
    pub public_total_supply: bool,
    pub enable_deposit: bool,
    pub enable_redeem: bool,
    pub enable_mint: bool,
    pub enable_burn: bool,
    pub enable_transfer: bool,
}

impl Config {
    pub fn save(&self, storage: &mut dyn Storage) -> StdResult<()> {
        Self::set_public_total_supply(storage, self.public_total_supply)?;
        Self::set_deposit_enabled(storage, self.enable_deposit)?;
        Self::set_redeem_enabled(storage, self.enable_redeem)?;
        Self::set_mint_enabled(storage, self.enable_mint)?;
        Self::set_burn_enabled(storage, self.enable_burn)?;
        Self::set_transfer_enabled(storage, self.enable_transfer)?;
        Ok(())
    }

    pub fn public_total_supply(storage: &dyn Storage) -> StdResult<bool> {
        Ok(Setting::load(storage, PUBLIC_TOTAL_SUPPLY)?.0)
    }

    pub fn set_public_total_supply(storage: &mut dyn Storage, setting: bool) -> StdResult<()> {
        Setting(setting).save(storage, PUBLIC_TOTAL_SUPPLY)?;
        Ok(())
    }

    pub fn deposit_enabled(storage: &dyn Storage) -> StdResult<bool> {
        Ok(Setting::load(storage, ENABLE_DEPOSIT)?.0)
    }

    pub fn set_deposit_enabled(storage: &mut dyn Storage, setting: bool) -> StdResult<()> {
        Setting(setting).save(storage, ENABLE_DEPOSIT)?;
        Ok(())
    }

    pub fn redeem_enabled(storage: &dyn Storage) -> StdResult<bool> {
        Ok(Setting::load(storage, ENABLE_REDEEM)?.0)
    }

    pub fn set_redeem_enabled(storage: &mut dyn Storage, setting: bool) -> StdResult<()> {
        Setting(setting).save(storage, ENABLE_REDEEM)?;
        Ok(())
    }

    pub fn mint_enabled(storage: &dyn Storage) -> StdResult<bool> {
        Ok(Setting::load(storage, ENABLE_MINT)?.0)
    }

    pub fn set_mint_enabled(storage: &mut dyn Storage, setting: bool) -> StdResult<()> {
        Setting(setting).save(storage, ENABLE_MINT)?;
        Ok(())
    }

    pub fn burn_enabled(storage: &dyn Storage) -> StdResult<bool> {
        Ok(Setting::load(storage, ENABLE_BURN)?.0)
    }

    pub fn set_burn_enabled(storage: &mut dyn Storage, setting: bool) -> StdResult<()> {
        Setting(setting).save(storage, ENABLE_BURN)?;
        Ok(())
    }

    pub fn transfer_enabled(storage: &dyn Storage) -> StdResult<bool> {
        Ok(Setting::load(storage, ENABLE_TRANSFER)?.0)
    }

    pub fn set_transfer_enabled(storage: &mut dyn Storage, setting: bool) -> StdResult<()> {
        Setting(setting).save(storage, ENABLE_TRANSFER)?;
        Ok(())
    }
}

#[cw_serde]
pub struct TotalSupply(pub Uint128);

impl ItemStorage for TotalSupply {
    const ITEM: Item<'static, Self> = Item::new("total-supply-");
}

impl TotalSupply {
    pub fn set(storage: &mut dyn Storage, amount: Uint128) -> StdResult<()> {
        TotalSupply(amount).save(storage)
    }

    pub fn add(storage: &mut dyn Storage, amount: Uint128) -> StdResult<Uint128> {
        let supply = TotalSupply::load(storage)?.0.checked_add(amount)?;
        TotalSupply::set(storage, supply)?;
        Ok(supply)
    }

    pub fn sub(storage: &mut dyn Storage, amount: Uint128) -> StdResult<Uint128> {
        let supply = TotalSupply::load(storage)?.0.checked_sub(amount)?;
        TotalSupply::set(storage, supply)?;
        Ok(supply)
    }
}

#[cw_serde]
pub struct Balance(pub Uint128);

impl MapStorage<'static, Addr> for Balance {
    const MAP: Map<'static, Addr, Self> = Map::new("balance-");
}

impl Balance {
    pub fn set(storage: &mut dyn Storage, amount: Uint128, addr: &Addr) -> StdResult<()> {
        Balance(amount).save(storage, addr.clone())
    }

    pub fn add(storage: &mut dyn Storage, amount: Uint128, addr: &Addr) -> StdResult<Uint128> {
        let supply = Self::may_load(storage, addr.clone())?
            .unwrap_or(Self(Uint128::zero()))
            .0
            .checked_add(amount)?;

        Balance::set(storage, supply, addr)?;
        Ok(supply)
    }

    pub fn sub(storage: &mut dyn Storage, amount: Uint128, addr: &Addr) -> StdResult<Uint128> {
        let subtractee = match Self::load(storage, addr.clone()) {
            Ok(amount) => amount.0,
            Err(_) => return Err(no_funds()),
        };
        let supply = match subtractee.checked_sub(amount) {
            Ok(supply) => supply,
            Err(_) => return Err(not_enough_funds()),
        };
        Balance::set(storage, supply, addr)?;
        Ok(supply)
    }

    pub fn transfer(
        storage: &mut dyn Storage,
        amount: Uint128,
        sender: &Addr,
        recipient: &Addr,
    ) -> StdResult<()> {
        Self::sub(storage, amount, sender)?;
        Self::add(storage, amount, recipient)?;
        Ok(())
    }
}

#[cw_serde]
pub struct Minters(pub Vec<Addr>);

impl ItemStorage for Minters {
    const ITEM: Item<'static, Self> = Item::new("minters-");
}

#[cw_serde]
pub struct AllowanceResponse {
    pub spender: Addr,
    pub owner: Addr,
    pub amount: Uint128,
    pub expiration: Option<u64>,
}

#[cw_serde]
pub struct Allowance {
    pub amount: Uint128,
    pub expiration: Option<u64>,
}

impl Default for Allowance {
    fn default() -> Self {
        Self {
            amount: Uint128::zero(),
            expiration: None,
        }
    }
}

impl Allowance {
    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        match self.expiration {
            Some(time) => block.time >= Timestamp::from_seconds(time),
            None => false,
        }
    }

    pub fn spend(
        storage: &mut dyn Storage,
        owner: &Addr,
        spender: &Addr,
        amount: Uint128,
        block: &BlockInfo,
    ) -> StdResult<()> {
        let mut allowance = Allowance::load(storage, (owner.clone(), spender.clone()))?;

        if allowance.is_expired(block) {
            return Err(allowance_expired(allowance.expiration.unwrap()));
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
impl MapStorage<'static, (Addr, Addr)> for Allowance {
    const MAP: Map<'static, (Addr, Addr), Self> = Map::new("allowance-");
}

#[cw_serde]
pub struct ReceiverHash(pub String);

impl MapStorage<'static, Addr> for ReceiverHash {
    const MAP: Map<'static, Addr, Self> = Map::new("receiver-hash-");
}

// Auth
pub use shade_protocol::contract_interfaces::query_auth::auth::{HashedKey, Key, PermitKey};
