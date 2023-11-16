use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Decimal, Timestamp, Uint128};

use lending_utils::interest::Interest;
use lending_utils::{coin::Coin, token::Token};

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
}

#[cw_serde]
pub enum ExecuteMsg {
    /// X market_token must be sent along with this message. If it matches, X c_token is minted of the sender address.
    /// The underlying market_token is stored in this Market contract
    Deposit {},
    /// This requests to withdraw the amount of C Tokens. More specifically,
    /// the contract will burn amount C Tokens and return that to the lender in base asset.
    Withdraw {
        amount: Uint128,
    },
    /// If sent tokens' denom matches market_token, burns tokens from sender's address
    Repay {},
    /// Increases the sender's debt and dispatches a message to send amount base asset to the sender
    Borrow {
        amount: Uint128,
    },
    /// Helper to allow repay of debt on given account.
    /// Sender must be a Credit Agency
    RepayTo {
        account: String,
    },
    /// Helper to allow transfering Ctokens from account source to account destination.
    /// Sender must be a Credit Agency
    TransferFrom {
        source: String,
        destination: String,
        amount: Uint128,
        liquidation_price: Decimal,
    },
    AdjustCommonToken {
        new_token: Token,
    },
    /// Withdraw some base asset, by burning C Tokens and swapping it for `buy` amount.
    /// The bought tokens are transferred to the sender.
    /// Only callable by the credit agency. Skips the credit line check.
    SwapWithdrawFrom {
        account: String,
        buy: Coin,
        sell_limit: Uint128,
        /// Selling assets for `buy` amount is simulated and uses the
        /// simulation's result as input for the swap. To be ahead of ever
        /// changing prices, add an estimate multiplicator to the output of
        /// simulate swap query.
        /// Have to be more then 1.0, not recommended to be above 1.01
        estimate_multiplier: Decimal,
    },
    /// Sender must be the Governance Contract
    AdjustCollateralRatio {
        new_ratio: Decimal,
    },
    /// Sender must be the Governance Contract
    AdjustReserveFactor {
        new_factor: Decimal,
    },
    /// Sender must be the Governance Contract
    AdjustPriceOracle {
        new_oracle: String,
    },
    /// Sender must be the Governance Contract
    AdjustMarketCap {
        new_cap: Option<Uint128>,
    },
    /// Sender must be the Governance Contract
    AdjustInterestRates {
        new_interest_rates: Interest,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns current configuration
    #[returns(crate::state::Config)]
    Configuration {},
    /// Returns TokensBalanceResponse
    #[returns(TokensBalanceResponse)]
    TokensBalance { account: String },
    /// Returns TransferableAmountResponse
    #[returns(TransferableAmountResponse)]
    TransferableAmount {
        /// WyndLend contract address that calls "CanTransfer"
        token: String,
        /// Address that wishes to transfer
        account: String,
    },
    /// Returns the amount that the given account can withdraw
    #[returns(Coin)]
    Withdrawable { account: String },
    /// Returns the amount that the given account can borrow
    #[returns(Coin)]
    Borrowable { account: String },
    /// Returns current utilisation and interest rates
    #[returns(InterestResponse)]
    Interest {},
    /// Returns PriceRate, structure representing sell/buy ratio for local(market)/common denoms
    #[returns(lending_utils::price::PriceRate)]
    PriceMarketLocalPerCommon {},
    /// Returns CreditLineResponse
    #[returns(lending_utils::credit_line::CreditLineResponse)]
    CreditLine { account: String },
    /// Returns ReserveResponse
    #[returns(ReserveResponse)]
    Reserve {},
    /// APY Query
    #[returns(ApyResponse)]
    Apy {},
    /// Returns the total amount of debt in the market in base asset
    /// Return type: `TokenInfoResponse`.
    #[returns(TotalDebtResponse)]
    TotalDebt {},
}

#[cw_serde]
pub struct MigrateMsg {
    pub lend_token_id: Option<u64>,
}

#[cw_serde]
pub enum QueryTotalCreditLine {
    TotalCreditLine { account: String },
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

// TODO: should this be defined elsewhere?
// This is here so we can call CA entrypoints without adding credit agency as a dependency.
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
