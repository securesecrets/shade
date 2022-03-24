use crate::impl_into_u8;
use crate::utils::errors::{build_string, CodeType, DetailedError};
use cosmwasm_std::{StdError, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug, JsonSchema)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum Error{
    BondEnded,
    BondNotStarted,
    LimitReached,
    MintExceedsLimit,
}

impl_into_u8!(Error);

impl CodeType for Error {
    fn to_verbose(&self, context: &Vec<&str>) -> String{
        match self{
            Error::BondEnded => {
                build_string("Bond ended on {}, it is currently {}", context)
            }
            Error::BondNotStarted => {
                build_string("Bond begins on {}, it is currently {}", context)
            }
            Error::LimitReached => {
                build_string("Bond issuance limit of {} has been reached", context)
            }
            Error::MintExceedsLimit => {
                build_string("Mint amount of {} exceeds available mint of {}", context)
            }
        }
    }
}

const BOND_TARGET: &str = "bond";


pub fn bond_not_started(start: &str, current: &str) -> StdError {
    DetailedError::from_code(
        BOND_TARGET,
        Error::BondNotStarted,
        vec![start, current],
    )
    .to_error()
}

pub fn bond_ended(end: &str, current: &str) -> StdError {
    DetailedError::from_code(BOND_TARGET, Error::BondEnded, vec![end, current]).to_error()
}

pub fn limit_reached(limit: Uint128) -> StdError {
    let limit_string: String = limit.into();
    let limit_str: &str = limit_string.as_str();
    DetailedError::from_code(BOND_TARGET, Error::LimitReached, vec![limit_str]).to_error()
}

pub fn mint_exceeds_limit(mint_amount: Uint128, available: Uint128) -> StdError{
    let mint_string: String = mint_amount.into();
    let mint_str= mint_string.as_str();
    let available_string: String = available.into();
    let available_str: &str = available_string.as_str();
    DetailedError::from_code(BOND_TARGET, Error::MintExceedsLimit, vec![mint_str, available_str]).to_error()
}