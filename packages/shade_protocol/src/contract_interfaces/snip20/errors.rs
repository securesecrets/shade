use crate::{
    impl_into_u8,
    utils::errors::{build_string, CodeType, DetailedError},
};
use crate::c_std::{Addr, StdError};

use cosmwasm_schema::{cw_serde};
use crate::c_std::Uint128;
use crate::contract_interfaces::snip20::Permission;

#[cw_serde]#[repr(u8)]
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

    // Auth errors
    NotAdmin,
    PermitRevoked,
    PermitNotFound,
    UnauthorisedPermit,
    InvalidViewingKey,

    // Config errors
    TransfersDisabled,
    MintingDisabled,
    NotMinter,
    BurningDisabled,
    RedeemDisabled,
    DepositDisabled,
    NotEnoughTokens,
    NoTokensReceived,
    UnsupportedToken,

    // Run state errors
    ActionDisabled,

    NotAuthenticatedMsg,

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
            Error::NotAdmin => build_string("Only admin is allowed to do this action", context),
            Error::PermitRevoked => build_string("Permit key {} is revoked", context),
            Error::UnauthorisedPermit => build_string("Permit lacks the required authorisation, expecting {}", context),
            Error::PermitNotFound => build_string("No permit was included in this query.", context),
            Error::InvalidViewingKey => build_string("Viewing key is invalid", context),
            Error::TransfersDisabled => build_string("Transfers are disabled", context),
            Error::MintingDisabled => build_string("Minting is disabled", context),
            Error::NotMinter => build_string("{} is not an allowed minter", context),
            Error::BurningDisabled => build_string("Burning is disabled", context),
            Error::RedeemDisabled => build_string("Redemptions are disabled", context),
            Error::DepositDisabled => build_string("Deposits are disabled", context),
            Error::NotEnoughTokens => build_string("Asking to redeem {} when theres only {} held by the reserve", context),
            Error::NoTokensReceived => build_string("Found no tokens to deposit", context),
            Error::UnsupportedToken => build_string("Sent tokens are not supported", context),
            Error::ActionDisabled => build_string("This action has been disabled", context),
            Error::NotAuthenticatedMsg => build_string("Message doesnt require authentication", context),
            Error::ContractStatusLevelInvalidConversion => build_string("Stored enum id {} is greater than total supported enum items", context),
            Error::TxCodeInvalidConversion => build_string("Stored action id {} is greater than total supported enum items", context),
            Error::LegacyCannotConvertFromTx => build_string("Legacy Txs only supports Transfer", context),
        }
    }
}

const TARGET: &str = "snip20";

pub fn invalid_name_format(name: &str) -> StdError {
    DetailedError::from_code(TARGET, Error::InvalidNameFormat, vec![name]).to_error()
}

pub fn invalid_symbol_format(symbol: &str) -> StdError {
    DetailedError::from_code(TARGET, Error::InvalidSymbolFormat, vec![symbol]).to_error()
}

pub fn invalid_decimals() -> StdError {
    DetailedError::from_code(TARGET, Error::InvalidDecimals, vec![]).to_error()
}

pub fn no_funds() -> StdError {
    DetailedError::from_code(TARGET, Error::NoFunds, vec![]).to_error()
}

pub fn not_enough_funds() -> StdError {
    DetailedError::from_code(TARGET, Error::NotEnoughFunds, vec![]).to_error()
}

pub fn allowance_expired(date: u64) -> StdError {
    DetailedError::from_code(TARGET, Error::AllowanceExpired, vec![&date.to_string()]).to_error()
}

pub fn not_admin() -> StdError {
    DetailedError::from_code(TARGET, Error::NotAdmin, vec![]).to_error()
}

pub fn permit_revoked(key: String) -> StdError {
    DetailedError::from_code(TARGET, Error::PermitRevoked, vec![&key]).to_error()
}

pub fn permit_not_found() -> StdError {
    DetailedError::from_code(TARGET, Error::PermitNotFound, vec![]).to_error()
}

pub fn unauthorized_permit(auth: Permission) -> StdError {
    let perm = match auth {
        Permission::Allowance => String::from("allowance"),
        Permission::Balance => String::from("balance"),
        Permission::History => String::from("history"),
        Permission::Owner => String::from("owner"),
    };
    DetailedError::from_code(TARGET, Error::UnauthorisedPermit, vec![&perm]).to_error()
}

pub fn invalid_viewing_key() -> StdError {
    DetailedError::from_code(TARGET, Error::InvalidViewingKey, vec![]).to_error()
}

pub fn transfer_disabled() -> StdError {
    DetailedError::from_code(TARGET, Error::TransfersDisabled, vec![]).to_error()
}

pub fn minting_disabled() -> StdError {
    DetailedError::from_code(TARGET, Error::MintingDisabled, vec![]).to_error()
}

pub fn not_minter(user: &Addr) -> StdError {
    DetailedError::from_code(TARGET, Error::NotMinter, vec![user.as_str()]).to_error()
}

pub fn burning_disabled() -> StdError {
    DetailedError::from_code(TARGET, Error::BurningDisabled, vec![]).to_error()
}

pub fn redeem_disabled() -> StdError {
    DetailedError::from_code(TARGET, Error::RedeemDisabled, vec![]).to_error()
}

pub fn deposit_disabled() -> StdError {
    DetailedError::from_code(TARGET, Error::DepositDisabled, vec![]).to_error()
}

pub fn not_enough_tokens(sent: Uint128, max: Uint128) -> StdError {
    DetailedError::from_code(TARGET, Error::NotEnoughTokens, vec![&sent.to_string(), &max.to_string()]).to_error()
}

pub fn no_tokens_received() -> StdError {
    DetailedError::from_code(TARGET, Error::NoTokensReceived, vec![]).to_error()
}

pub fn unsupported_token() -> StdError {
    DetailedError::from_code(TARGET, Error::UnsupportedToken, vec![]).to_error()
}

pub fn action_disabled() -> StdError {
    DetailedError::from_code(TARGET, Error::ActionDisabled, vec![]).to_error()
}

pub fn not_authenticated_msg() -> StdError {
    DetailedError::from_code(TARGET, Error::NotAuthenticatedMsg, vec![]).to_error()
}

pub fn insufficient_allowance() -> StdError {
    DetailedError::from_code(TARGET, Error::InsufficientAllowance, vec![]).to_error()
}

pub fn contract_status_level_invalid(id: u8) -> StdError {
    DetailedError::from_code(TARGET, Error::ContractStatusLevelInvalidConversion, vec![&id.to_string()]).to_error()
}

pub fn tx_code_invalid_conversion(id: u8) -> StdError {
    DetailedError::from_code(TARGET, Error::TxCodeInvalidConversion, vec![&id.to_string()]).to_error()
}

pub fn legacy_cannot_convert_from_tx() -> StdError {
    DetailedError::from_code(TARGET, Error::LegacyCannotConvertFromTx, vec![]).to_error()
}

