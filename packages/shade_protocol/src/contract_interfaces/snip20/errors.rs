use crate::{
    impl_into_u8,
    utils::errors::{build_string, CodeType, DetailedError},
};
use cosmwasm_std::StdError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug, JsonSchema)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    // Init Errors
    InvalidNameFormat,
    InvalidSymbolFormat,
    InvalidDecimals,

    // User errors
    NoFunds,
    NotEnoughFunds,
    AllowanceExpired,
    InsufficientAllowance,

    // Errors that shouldnt happen
    ContractStatusLevelInvalidConversion,
    TxCodeInvalidConversion,
    LegacyCannotConvertFromTx,
}

impl_into_u8!(Error);

impl CodeType for Error {
    fn to_verbose(&self, context: &Vec<&str>) -> String {
        match self {
            Error::InvalidNameFormat => build_string("{} is not in the expected name format (3-30 UTF-8 bytes)", context),
            Error::InvalidSymbolFormat => build_string("{} is not in the expected symbol format [A-Z]{3,6}", context),
            Error::InvalidDecimals => build_string("Decimals must not exceed 18", context),
            Error::NoFunds => build_string("Account has no funds", context),
            Error::NotEnoughFunds => build_string("Account doesnt have enough funds", context),
            Error::AllowanceExpired => build_string("Allowance expired on {}", context),
            Error::InsufficientAllowance => build_string("Insufficient allowance", context),
            Error::ContractStatusLevelInvalidConversion => build_string("Stored enum id {} is greater than total supported enum items", context),
            Error::TxCodeInvalidConversion => build_string("Stored action id {} is greater than total supported enum items", context),
            Error::LegacyCannotConvertFromTx => build_string("Legacy Txs only supports Transfer", context),
        }
    }
}

const target: &str = "snip20";

pub fn invalid_name_format(name: &str) -> StdError {
    DetailedError::from_code(target, Error::InvalidNameFormat, vec![name]).to_error()
}

pub fn invalid_symbol_format(symbol: &str) -> StdError {
    DetailedError::from_code(target, Error::InvalidSymbolFormat, vec![symbol]).to_error()
}

pub fn invalid_decimals() -> StdError {
    DetailedError::from_code(target, Error::InvalidDecimals, vec![]).to_error()
}

pub fn no_funds() -> StdError {
    DetailedError::from_code(target, Error::NoFunds, vec![]).to_error()
}

pub fn not_enough_funds() -> StdError {
    DetailedError::from_code(target, Error::NotEnoughFunds, vec![]).to_error()
}

pub fn allowance_expired(date: u64) -> StdError {
    DetailedError::from_code(target, Error::AllowanceExpired, vec![&date.to_string()]).to_error()
}

pub fn insufficient_allowance() -> StdError {
    DetailedError::from_code(target, Error::InsufficientAllowance, vec![]).to_error()
}

pub fn contract_status_level_invalid(id: u8) -> StdError {
    DetailedError::from_code(target, Error::ContractStatusLevelInvalidConversion, vec![&id.to_string()]).to_error()
}

pub fn tx_code_invalid_conversion(id: u8) -> StdError {
    DetailedError::from_code(target, Error::TxCodeInvalidConversion, vec![&id.to_string()]).to_error()
}

pub fn legacy_cannot_convert_from_tx() -> StdError {
    DetailedError::from_code(target, Error::LegacyCannotConvertFromTx, vec![]).to_error()
}

