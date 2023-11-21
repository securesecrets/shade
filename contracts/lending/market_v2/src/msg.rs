use cosmwasm_schema::{cw_serde, QueryResponses};
use shade_protocol::c_std::{ContractInfo, Decimal, Timestamp, Uint128};

use lending_utils::{
    interest::Interest,
    {coin::Coin, token::Token},
};

#[cw_serde]
pub struct InstantiateMsg {
    /// Name used to create the cToken name `Lent ${name}`
    pub name: String,
    /// Symbol used to create the cToken `C${symbol}`
    pub symbol: String,
    /// Decimals for cToken
    pub decimals: u8,
    /// CodeId used to create cToken
    pub token_id: u64,
    /// Market token
    pub market_token: Token,
    /// An optional cap on total number of tokens deposited into the market
    pub market_cap: Option<Uint128>,
    /// Interest rate curve
    pub interest_rate: Interest,
    /// Token which would be distributed via created lend contracts
    pub distributed_token: Token,
    /// Define interest's charged period (in seconds)
    pub interest_charge_period: u64,
    /// Common Token denom that comes from Credit Agency (same for all markets)
    pub common_token: Token,
    /// Ratio of how much tokens can be borrowed for one unit, 0 <= x < 1
    pub collateral_ratio: Decimal,
    /// Address of contract to query for price
    pub price_oracle: String,
    /// Defines the portion of borrower interest that is converted into reserves (0 <= x <= 1)
    pub reserve_factor: Decimal,
    /// Maximum percentage of credit_limit that can be borrowed.
    /// This is used to prevent borrowers from being liquidated (almost) immediately after borrowing,
    /// because they maxed out their credit limit.
    pub borrow_limit_ratio: Decimal,
    /// Address of the governance contract that controls this market
    pub gov_contract: String,
    /// Key used for reading data in queries
    pub viewing_key: String,
    // I have no idea what to do with it
    pub ctoken_code_hash: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// This requests to withdraw the amount of C Tokens. More specifically,
    /// the contract will burn amount C Tokens and return that to the lender in base asset.
    Withdraw { amount: Uint128 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}

#[cw_serde]
pub struct InterestResponse {
    pub interest: Decimal,
    pub utilisation: Decimal,
    pub charge_period: Timestamp,
}

#[cw_serde]
pub struct TokensBalanceResponse {
    pub collateral: Coin,
    pub debt: Coin,
}

#[cw_serde]
pub struct TransferableAmountResponse {
    pub transferable: Uint128,
}

#[cw_serde]
pub struct ReserveResponse {
    pub reserve: Uint128,
}

#[cw_serde]
pub enum CreditAgencyExecuteMsg {
    /// Ensures a given account has entered a market. Meant to be called by a specific
    /// market contract - so the sender of the msg would be the market
    EnterMarket { account: String },
}

#[cw_serde]
pub struct ApyResponse {
    /// How much % interest will a borrower have to pay
    pub borrower: Decimal,
    /// How much % interest will a lender earn
    pub lender: Decimal,
}

#[cw_serde]
pub struct TotalDebtResponse {
    /// Total amount of debt in the market, denominated in base asset
    pub total: Uint128,

    /// The current debt multiplier used to convert debt to base assets
    pub multiplier: Decimal,
}