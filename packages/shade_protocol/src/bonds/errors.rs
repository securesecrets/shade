use crate::impl_into_u8;
use crate::utils::errors::{build_string, CodeType, DetailedError};
use cosmwasm_std::StdError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Debug, JsonSchema)]
#[repr(u8)]
#[serde(rename_all = "snake_case")]
pub enum Error{
    BondEnded,
    BondNotStarted,
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