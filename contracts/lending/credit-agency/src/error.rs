use shade_protocol::{
    c_std::{Addr, StdError, Uint128},
    contract_interfaces::snip20::Snip20ReceiveMsg,
};

use lending_utils::{
    coin::{Coin, CoinError},
    credit_line::InvalidCommonTokenDenom,
    price::PriceError,
};

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Price(#[from] PriceError),

    #[error("{0}")]
    Coin(#[from] CoinError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Creating Market failure - collateral ratio must be lower than liquidation price")]
    MarketCfgCollateralFailure {},

    #[error("Unrecognised reply id: {0}")]
    UnrecognisedReply(u64),

    #[error("Invalid reply from submessage {id}, {err}")]
    ReplyParseFailure { id: u64, err: String },

    #[error("No market set up for base asset {0}")]
    NoMarket(String),

    #[error("A market for base asset {0} is still being created")]
    MarketCreating(String),

    #[error("A market for base asset {0} already exists")]
    MarketAlreadyExists(String),

    #[error("Account cannot be liquidated as it does not have more debt then credit line")]
    LiquidationNotAllowed {},

    #[error("Only one denom can be sent for liquidation")]
    LiquidationOnlyOneDenomRequired {},

    #[error("{0}")]
    InvalidCommonTokenDenom(#[from] InvalidCommonTokenDenom),

    #[error("{market}: Market either does not exist or is not active yet")]
    MarketSearchError { market: String },

    #[error("{address} is not on a market {market}")]
    NotOnMarket { address: Addr, market: Addr },

    #[error("{address} has dept on market {market} of {debt:?}")]
    DebtOnMarket {
        address: Addr,
        market: Addr,
        debt: Coin,
    },

    #[error("Not enough credit line left after operation, total dept: {debt}, total credit line: credit_line, total collateral: {collateral}")]
    NotEnoughCollat {
        debt: Uint128,
        credit_line: Uint128,
        collateral: Uint128,
    },

    #[error("Cw20 tokens are not supported yet")]
    Cw20TokensNotSupported,

    #[error("Repaying loan using collateral failed - your debt is bigger then your credit line")]
    RepayingLoanUsingCollateralFailed {},

    #[error("Estimate multiplier must be bigger or equal to 1.0")]
    InvalidEstimateMultiplier {},

    #[error("Invalid liquidation price threshold - must be between 0% and 5%")]
    InvalidLiquidationThreshold {},
}
