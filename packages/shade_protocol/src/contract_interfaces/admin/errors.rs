use crate::utils::errors::{build_string, CodeType};

#[cw_serde]
#[repr(u8)]
pub enum Error {
    UnregisteredAdmin,
    UnauthorizedAdmin,
    UnauthorizedSuper,
    NoPermissions,
    IsShutdown,
    IsUnderMaintenance,
    InvalidPermissionFormat,
}

impl CodeType for Error {
    fn to_verbose(&self, context: &Vec<&str>) -> String {
        build_string(
            match self {
                Error::UnregisteredAdmin => "{} has not been registered as an admin",
                Error::UnauthorizedAdmin => "{} does not have this permissions - {}",
                Error::UnauthorizedSuper => "{} is not the authorized super admin",
                Error::NoPermissions => "There are not permissions for {}",
                Error::IsShutdown => {
                    "Contract is currently shutdown. It must be turned on for any changes to be made or any permissions to be validates"
                }
                Error::IsUnderMaintenance => {
                    "Contract is under maintenance. Oly registry updated may be made. Permission validation is disabled."
                }
                Error::InvalidPermissionFormat => {
                    "{} must be > 10 characters and only contains 0-9, A-Z, and underscores"
                }
            },
            context,
        )
    }
}

const ADMIN_TARGET: &str = "airdrop";

pub fn unregistered_admin(address: String) -> StdError {
    DetailedError::from_code(ADMIN_TARGET, Error::UnregisteredAdmin, vec![address]).to_error()
}

pub fn unauthorized_admin(address: String, permission: String) -> StdError {
    DetailedError::from_code(ADMIN_TARGET, Error::UnauthorizedAdmin, vec![
        address, permission,
    ])
    .to_error()
}
pub fn unauthorized_super(super_admin: String) -> StdError {
    DetailedError::from_code(ADMIN_TARGET, Error::UnauthorizedSuper, vec![super_admin]).to_error()
}
pub fn no_permission(user: String) -> StdError {
    DetailedError::from_code(ADMIN_TARGET, Error::NoPermissions, vec![user]).to_error()
}
pub fn is_shutdown() -> StdError {
    DetailedError::from_code(ADMIN_TARGET, Error::IsShutdown, vec![]).to_error()
}
pub fn is_under_maintenance() -> StdError {
    DetailedError::from_code(ADMIN_TARGET, Error::IsUnderMaintenance, vec![]).to_error()
}
pub fn invalid_permission_format(permission: String) -> StdError {
    DetailedError::from_code(ADMIN_TARGET, Error::InvalidPermissionFormat, vec![
        permission,
    ])
    .to_error()
}
