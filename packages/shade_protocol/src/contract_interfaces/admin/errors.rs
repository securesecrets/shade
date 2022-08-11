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
