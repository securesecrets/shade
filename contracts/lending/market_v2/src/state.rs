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

