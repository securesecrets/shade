use crate::impl_into_u8;
use crate::utils::errors::{build_string, CodeType, DetailedError};
use crate::c_std::StdError;

use cosmwasm_schema::{cw_serde};

#[cw_serde]
#[repr(u8)]
pub enum Error {
    NotUtilityAdmin,
    NoContractFound,
    CriticalAdminError,
    NoAddressFound,
    UnderMaintenance
}

impl_into_u8!(Error);

impl CodeType for Error {
    fn to_verbose(&self, context: &Vec<&str>) -> String {
        match self{
            Error::NotUtilityAdmin => {
                build_string("User is not authorized to act as the Utility Router admin", context)
            }
            Error::NoContractFound => {
                build_string("No contract found at name {}", context)
            }
            Error::CriticalAdminError => {
                build_string("Admin contract cannot be found, so no users can be trusted. Contract must be redeployed with admin.", context)
            }
            Error::NoAddressFound => {
                build_string("No address found at name {}", context)
            }
            Error::UnderMaintenance => {
                build_string("Cannot query for information, as router is under maintenance", context)
            }
        }
    }
}

const UTIL_ROUTER_TARGET: &str = "utility";

pub fn not_admin() -> StdError {
    DetailedError::from_code(UTIL_ROUTER_TARGET, Error::NotUtilityAdmin, vec![]).to_error()
}

pub fn no_contract_found(name: String) -> StdError {
    DetailedError::from_code(UTIL_ROUTER_TARGET, Error::NoContractFound, vec![name.as_str()]).to_error()
}

pub fn critical_admin_error() -> StdError {
    DetailedError::from_code(UTIL_ROUTER_TARGET, Error::CriticalAdminError, vec![]).to_error()
}

pub fn no_address_found(name: String) -> StdError {
    DetailedError::from_code(UTIL_ROUTER_TARGET, Error::NoAddressFound, vec![name.as_str()]).to_error()
}

pub fn under_maintenance() -> StdError {
    DetailedError::from_code(UTIL_ROUTER_TARGET, Error::UnderMaintenance, vec![]).to_error()
}