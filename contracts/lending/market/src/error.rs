use cosmwasm_std::{OverflowError, StdError, Uint128};
use lend_utils::interest::InterestError;
use thiserror::Error;

use lend_utils::credit_line::InvalidCommonTokenDenom;
use lend_utils::price::PriceError;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Price(#[from] PriceError),

    #[error("{0}")]
    Overflow(#[from] OverflowError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Unrecognised reply id: {0}")]
    UnrecognisedReply(u64),

    #[error("Unrecognised token: {0}")]
    UnrecognisedToken(String),

    #[error("Invalid reply from submessage {id}, {err}")]
    ReplyParseFailure { id: u64, err: String },

    #[error("No funds sent")]
    NoFundsSent {},

    #[error("Sent too many denoms, must deposit only '{0}' in the lending pool")]
    ExtraDenoms(String),

    #[error("Sent unsupported token, must deposit '{0}' in the lending pool")]
    InvalidDenom(String),

    #[error("Expected the cw20 token {0}, but got native tokens instead")]
    Cw20Expected(String),

    #[error("Fatal: market token price is zero")]
    ZeroPrice {},

    #[error("Fatal: market collateral ratio is zero")]
    ZeroCollateralRatio {},

    #[error("Liquidation price cannot be zero")]
    ZeroLiquidationPrice {},

    #[error("Cannot borrow amount {amount} for {account}")]
    CannotBorrow { amount: Uint128, account: String },

    #[error("Address {account} cannot withdraw {amount}")]
    CannotWithdraw { account: String, amount: Uint128 },

    #[error("Insufficient amount of debt on account {account}: {debt} to liquidate debt")]
    LiquidationInsufficientDebt { account: String, debt: Uint128 },

    #[error("Unauthorized - requires sender to be a Market's Credit Agency")]
    RequiresCreditAgency {},

    #[error("{0}")]
    InvalidCommonTokenDenom(#[from] InvalidCommonTokenDenom),

    #[error("{0}")]
    InterestError(#[from] InterestError),

    #[error("Cannot deposit {attempted_deposit} tokens - market cap is {cap} and there are already {ctoken_base_supply} tokens present")]
    DepositOverCap {
        attempted_deposit: Uint128,
        ctoken_base_supply: Uint128,
        cap: Uint128,
    },

    #[error("Cw20 tokens are not supported yet")]
    Cw20TokensNotSupported,

    #[error(
        "WYND DEX returned SwapAmount::Out in response for estimate - something went wrong, abort"
    )]
    IncorrectSwapAmountResponse {},

    #[error("Exactly one coin have to be sent.")]
    RequiresExactlyOneCoin {},

    #[error("Estimated required amount [{estimate}] is higher than sell limit [{sell_limit}].")]
    EstimateHigherThanLimit {
        estimate: Uint128,
        sell_limit: Uint128,
    },

    #[error("Estimate multiplier must be bigger or equal to 1.0")]
    InvalidEstimateMultiplier {},
}
