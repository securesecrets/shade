use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use shade_protocol::{
    c_std::{Addr, Decimal, Uint128, ContractInfo},
    secret_storage_plus::Item,
};

use lending_utils::{interest::ValidatedInterest, token::Token};

pub const SECONDS_IN_YEAR: u128 = 365 * 24 * 3600;

#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, JsonSchema, Debug)]
pub struct Config {
    pub ctoken_contract: ContractInfo,
    /// The contract that controls this contract and is allowed to adjust its parameters
    pub governance_contract: Addr,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub token_id: u64,
    /// Denom for current market
    pub market_token: Token,
    /// An optional cap on total number of tokens deposited into the market
    pub market_cap: Option<Uint128>,
    /// Interest rate calculation
    pub rates: ValidatedInterest,
    pub interest_charge_period: u64,
    pub last_charged: u64,
    /// Denom common amongst markets within same Credit Agency
    pub common_token: Token,
    pub collateral_ratio: Decimal,
    /// Maximum percentage of credit_limit that can be borrowed.
    /// This is used to prevent borrowers from being liquidated (almost) immediately after borrowing,
    /// because they maxed out their credit limit.
    pub borrow_limit_ratio: Decimal,
    /// Address of Oracle's contract
    pub price_oracle: String,
    /// Address of Credit Agency
    pub credit_agency: Addr,
    pub reserve_factor: Decimal,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const VIEWING_KEY: Item<String> = Item::new("viewing_key");

pub mod debt {
    use super::*;

    use crate::ContractError;
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{StdResult, Storage};
    use cw_storage_plus::Map;
    use lending_utils::amount::{base_to_token, token_to_base};

    #[cw_serde]
    struct DebtInfo {
        /// Total amount of debt
        pub total_points: Uint128,
        /// The multiplier used to convert the `total_points` to base tokens
        pub multiplier: Decimal,
    }

    /// Total amount of debt per user
    const DEBT: Map<&Addr, Uint128> = Map::new("debt");
    const DEBT_INFO: Item<DebtInfo> = Item::new("debt_info");

    pub fn init(storage: &mut dyn Storage) -> StdResult<()> {
        DEBT_INFO.save(
            storage,
            &DebtInfo {
                total_points: Uint128::zero(),
                multiplier: Decimal::from_ratio(1u128, 100_000u128),
            },
        )
    }

    pub fn multiplier(storage: &dyn Storage) -> StdResult<Decimal> {
        Ok(DEBT_INFO.load(storage)?.multiplier)
    }

    /// Returns the total amount of debt in base tokens, as well as the multiplier
    pub fn total(storage: &dyn Storage) -> StdResult<(Uint128, Decimal)> {
        let info = DEBT_INFO.load(storage)?;
        Ok((
            token_to_base(info.total_points, info.multiplier),
            info.multiplier,
        ))
    }

    /// Returns the amount of debt in base tokens for the given address
    pub fn of(storage: &dyn Storage, address: &Addr) -> Result<Uint128, ContractError> {
        let raw_balance = DEBT.may_load(storage, address)?.unwrap_or_default();
        let multiplier = multiplier(storage)?;
        Ok(token_to_base(raw_balance, multiplier))
    }

    /// Changes the multiplier by the given ratio
    pub fn rebase(storage: &mut dyn Storage, ratio: Decimal) -> Result<(), ContractError> {
        DEBT_INFO.update(storage, |mut info| -> StdResult<_> {
            info.multiplier *= ratio;
            Ok(info)
        })?;
        Ok(())
    }

    /// Increases the debt by the given amount of base tokens
    pub fn increase(
        storage: &mut dyn Storage,
        recipient: &Addr,
        base_amount: Uint128,
    ) -> Result<(), ContractError> {
        if base_amount.is_zero() {
            return Ok(());
        }

        change_amount(storage, recipient, |old_amount, multiplier| {
            old_amount + base_to_token(base_amount, multiplier)
        })
    }

    /// Decrease the debt by the given amount of base tokens.
    /// Returns the leftover amount of base tokens
    pub fn decrease(
        storage: &mut dyn Storage,
        from: &Addr,
        base_amount: Uint128,
    ) -> Result<Uint128, ContractError> {
        if base_amount.is_zero() {
            return Ok(Uint128::zero());
        }

        // If there are more tokens sent then there are to repay, burn only desired
        // amount and return the difference
        let mut surplus = Uint128::zero();
        change_amount(storage, from, |old_amount, multiplier| {
            // convert from base currency to equivalent token amount to reduce rounding error
            let points = base_to_token(base_amount, multiplier);
            if points >= old_amount {
                surplus = token_to_base(points - old_amount, multiplier);
                Uint128::zero()
            } else {
                old_amount - points
            }
        })?;
        Ok(surplus)
    }

    fn change_amount(
        storage: &mut dyn Storage,
        account: &Addr,
        mut change: impl FnMut(Uint128, Decimal) -> Uint128,
    ) -> Result<(), ContractError> {
        let mut info = DEBT_INFO.load(storage)?;
        info.total_points = change(info.total_points, info.multiplier);
        DEBT_INFO.save(storage, &info)?;

        DEBT.update(storage, account, |debt| -> StdResult<_> {
            let new_debt = change(debt.unwrap_or_default(), info.multiplier);
            Ok(new_debt)
        })?;

        Ok(())
    }
}
