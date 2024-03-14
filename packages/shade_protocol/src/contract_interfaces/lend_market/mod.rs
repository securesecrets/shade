use crate::{
    c_std::{Addr, ContractInfo, Decimal, Timestamp, Uint128},
    lending_utils::{coin::Coin, interest::Interest, token::Token, Authentication},
    utils::{asset::Contract, ExecuteCallback, InstantiateCallback, Query},
};
use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub lend_market_id: u64,
    pub lend_market_code_hash: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// This requests to withdraw the amount of C Tokens. More specifically,
    /// the contract will burn amount C Tokens and return that to the lender in base asset.
    Withdraw { amount: Uint128 },
    /// Increases the sender's debt and dispatches a message to send amount base asset to the sender
    Borrow { amount: Uint128 },
    /// Helper to allow transfering Ctokens from account source to account destination.
    /// Sender must be a Credit Agency
    TransferFrom {
        source: Addr,
        destination: Addr,
        amount: Uint128,
        liquidation_price: Decimal,
    },
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}
impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ReceiveMsg {
    /// X market_token must be sent along with this message. If it matches, X c_token is minted of the sender address.
    /// The underlying market_token is stored in this Market contract
    Deposit {},
    /// If sent tokens' denom matches market_token, burns tokens from sender's address
    Repay {},
    /// Helper to allow repay of debt on given account.
    /// Sender must be a Credit Agency
    RepayTo { account: String },
}

/*
#[cw_serde]
pub enum CreditAgencyExecuteMsg {
    /// Ensures a given account has entered a market. Meant to be called by a specific
    /// market contract - so the sender of the msg would be the market
    EnterMarket { account: String },
}
*/

#[cw_serde]
pub struct AuthPermit {}

#[cw_serde]
pub enum QueryMsg {
    /// Returns current configuration
    Configuration {},
    /// Returns current utilisation and interest rates
    Interest {},
    /// Returns PriceRate, structure representing sell/buy ratio for local(market)/common denoms
    PriceMarketLocalPerCommon {},
    /// Returns TransferableAmountResponse
    TransferableAmount {
        /// Lend contract address that calls "CanTransfer"
        token: ContractInfo,
        /// Address that wishes to transfer
        account: String,
    },
    Reserve {},
    /// APY Query
    Apy {},
    /// Returns the total amount of debt in the market in base asset
    /// Return type: `TokenInfoResponse`.
    TotalDebt {},
    /// Returns TokensBalanceResponse
    TokensBalance {
        account: Addr,
        authentication: Authentication,
    },
    /// Returns the amount that the given account can withdraw
    Withdrawable {
        account: Addr,
        authentication: Authentication,
    },
    /// Returns the amount that the given account can borrow
    Borrowable {
        account: Addr,
        authentication: Authentication,
    },
    /// Returns CreditLineResponse
    CreditLine {
        account: Addr,
        authentication: Authentication,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum QueryTotalCreditLine {
    TotalCreditLine { account: String },
}

impl Query for QueryTotalCreditLine {
    const BLOCK_SIZE: usize = 256;
}

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
