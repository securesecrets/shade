use shade_protocol::{
    c_std::{Addr, StdError},
    cosmwasm_schema::{cw_serde, QueryResponses},
    thiserror::Error,
    utils::{ExecuteCallback, InstantiateCallback, Query},
};

pub type AdminAuthResult<T> = core::result::Result<T, AdminAuthError>;

#[cw_serde]
pub enum AdminAuthStatus {
    Active,
    Maintenance,
    Shutdown,
}

impl AdminAuthStatus {
    // Throws an error if status is under maintenance
    pub fn not_under_maintenance(&self) -> AdminAuthResult<&Self> {
        if self.eq(&AdminAuthStatus::Maintenance) {
            return Err(AdminAuthError::IsUnderMaintenance);
        }
        Ok(self)
    }

    // Throws an error if status is shutdown
    pub fn not_shutdown(&self) -> AdminAuthResult<&Self> {
        if self.eq(&AdminAuthStatus::Shutdown) {
            return Err(AdminAuthError::IsShutdown);
        }
        Ok(self)
    }
}

#[cw_serde]
pub struct InstantiateMsg {
    pub super_admin: Option<String>,
}

impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateRegistry { action: RegistryAction },
    UpdateRegistryBulk { actions: Vec<RegistryAction> },
    TransferSuper { new_super: String },
    SelfDestruct {},
    ToggleStatus { new_status: AdminAuthStatus },
}

#[cw_serde]
pub enum RegistryAction {
    RegisterAdmin {
        user: String,
    },
    GrantAccess {
        permissions: Vec<String>,
        user: String,
    },
    RevokeAccess {
        permissions: Vec<String>,
        user: String,
    },
    DeleteAdmin {
        user: String,
    },
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    GetConfig {},
    #[returns(AdminsResponse)]
    GetAdmins {},
    #[returns(PermissionsResponse)]
    GetPermissions { user: String },
    #[returns(ValidateAdminPermissionResponse)]
    ValidateAdminPermission { permission: String, user: String },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub struct ConfigResponse {
    pub super_admin: Addr,
    pub status: AdminAuthStatus,
}

#[cw_serde]
pub struct PermissionsResponse {
    pub permissions: Vec<String>,
}

#[cw_serde]
pub struct AdminsResponse {
    pub admins: Vec<Addr>,
}

#[cw_serde]
pub struct ValidateAdminPermissionResponse {
    pub has_permission: bool,
}

#[derive(Error, Debug, PartialEq)]
pub enum AdminAuthError {
    #[error("{0}")]
    // let thiserror implement From<StdError> for you
    Std(#[from] StdError),
    // this is whatever we want
    #[error("Registry error: {user} has not been registered as an admin.")]
    UnregisteredAdmin { user: Addr },
    #[error("Permission denied: {user} does not have this permission - {permission}.")]
    UnauthorizedAdmin { user: Addr, permission: String },
    #[error("Permission denied: {expected_super_admin} is not the authorized super admin.")]
    UnauthorizedSuper { expected_super_admin: Addr },
    #[error("Registry error: there are no permissions for this {user}.")]
    NoPermissions { user: Addr },
    #[error(
        "Contract is currently shutdown. It must be turned on for any changes to be made or any permissions to be validated."
    )]
    IsShutdown,
    #[error(
        "Contract is under maintenance. Only registry updates may be made. Consumers cannot validate permissions at this time."
    )]
    IsUnderMaintenance,
    #[error("{permission} must be > 10 characters and only contains 0-9, A-Z, and underscores.")]
    InvalidPermissionFormat { permission: String },
}
