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
}
