use crate::impl_into_u8;
use crate::utils::errors::{build_string, CodeType, DetailedError};
use crate::c_std::Uint128;
use crate::c_std::{Addr, StdError};

use cosmwasm_schema::{cw_serde};

#[cw_serde]
#[repr(u8)]
pub enum Error {
    NotUtilityAdmin,
}

impl_into_u8!(Error);

impl CodeType for Error {
    fn to_verbose(&self, context: &Vec<&str>) -> String {
        match self{
            Error::NotUtilityAdmin => {
                build_string("User is not authorized to act as the Utility Router admin", context)
            }
        }
    }
}

const UTIL_ROUTER_TARGET: &str = "utility";

pub fn not_admin() -> StdError {
    DetailedError::from_code(UTIL_ROUTER_TARGET, Error::NotUtilityAdmin, vec![]).to_error()
}