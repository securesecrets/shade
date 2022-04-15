use std::fmt;

use snafu::Snafu;

pub use cosmwasm_std::StdError;

impl From<OverflowError> for StdError {
    fn from(err: OverflowError) -> Self {
        Self::generic_err(err.to_string())
    }
}

impl From<ConversionOverflowError> for StdError {
    fn from(err: ConversionOverflowError) -> Self {
        Self::generic_err(err.to_string())
    }
}

impl From<DivideByZeroError> for StdError {
    fn from(err: DivideByZeroError) -> Self {
        Self::generic_err(err.to_string())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum OverflowOperation {
    Add,
    Sub,
    Mul,
    Pow,
    Shr,
    Shl,
}

impl fmt::Display for OverflowOperation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Snafu, Debug, PartialEq, Eq)]
#[snafu(display(
    "Overflow Error: cannot {} with {} and {}",
    operation,
    operand1,
    operand2
))]
pub struct OverflowError {
    pub operation: OverflowOperation,
    pub operand1: String,
    pub operand2: String,
}

impl OverflowError {
    pub fn new(
        operation: OverflowOperation,
        operand1: impl ToString,
        operand2: impl ToString,
    ) -> Self {
        Self {
            operation,
            operand1: operand1.to_string(),
            operand2: operand2.to_string(),
        }
    }
}

/// The error returned by [`TryFrom`] conversions that overflow, for example
/// when converting from [`Uint256`] to [`Uint128`].
///
/// [`TryFrom`]: std::convert::TryFrom
/// [`Uint256`]: crate::Uint256
/// [`Uint128`]: crate::Uint128
#[derive(Snafu, Debug, PartialEq, Eq)]
#[snafu(display(
    "Conversion Overflow Error: cannot convert {} to {} for {}",
    source_type,
    target_type,
    value
))]
pub struct ConversionOverflowError {
    pub source_type: &'static str,
    pub target_type: &'static str,
    pub value: String,
}

impl ConversionOverflowError {
    pub fn new(
        source_type: &'static str,
        target_type: &'static str,
        value: impl Into<String>,
    ) -> Self {
        Self {
            source_type,
            target_type,
            value: value.into(),
        }
    }
}

#[derive(Snafu, Debug, PartialEq, Eq)]
#[snafu(display("Divide By Zero: cannot devide {} by zero", operand))]
pub struct DivideByZeroError {
    pub operand: String,
}

impl DivideByZeroError {
    pub fn new(operand: impl ToString) -> Self {
        Self {
            operand: operand.to_string(),
        }
    }
}
