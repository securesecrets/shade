use lending_utils::interest::InterestError;
use shade_protocol::c_std::{OverflowError, StdError, Uint128};
use thiserror::Error;

use lending_utils::credit_line::InvalidCommonTokenDenom;
use lending_utils::price::PriceError;

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
}
