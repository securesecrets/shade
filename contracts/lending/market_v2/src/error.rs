use shade_protocol::{
    c_std::{OverflowError, StdError, Uint128},
    lending_utils::{
        credit_line::InvalidCommonTokenDenom, interest::InterestError, price::PriceError,
    },
};
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Price(#[from] PriceError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("Invalid reply from submessage {id}, {err}")]
    ReplyParseFailure { id: u64, err: String },

    #[error("Unrecognised reply id: {0}")]
    UnrecognisedReply(u64),

    #[error("{0}")]
    InterestError(#[from] InterestError),

    #[error("Unrecognised token: {0}")]
    UnrecognisedToken(String),

    #[error("Fatal: market collateral ratio is zero")]
    ZeroCollateralRatio {},

    #[error("{0}")]
    InvalidCommonTokenDenom(#[from] InvalidCommonTokenDenom),

    #[error("Fatal: market token price is zero")]
    ZeroPrice {},

    #[error("Sent unsupported token, must deposit '{0}' in the lending pool")]
    InvalidDenom(String),

    #[error("Cannot deposit {attempted_deposit} tokens - market cap is {cap} and there are already {ctoken_base_supply} tokens present")]
    DepositOverCap {
        attempted_deposit: Uint128,
        ctoken_base_supply: Uint128,
        cap: Uint128,
    },

    #[error("Address {account} cannot withdraw {amount}")]
    CannotWithdraw { account: String, amount: Uint128 },

    #[error("Cannot borrow amount {amount} for {account}")]
    CannotBorrow { amount: Uint128, account: String },

    #[error("Insufficient amount of debt on account {account}: {debt} to liquidate debt")]
    LiquidationInsufficientDebt { account: String, debt: Uint128 },

    #[error("Unauthorized - requires sender to be a Market's Credit Agency")]
    RequiresCreditAgency {},

    #[error("Unauthorized")]
    Unauthorized {},
}
