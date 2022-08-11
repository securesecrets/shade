pub mod errors;
pub mod helpers;

use c_std::{Addr, StdError};
use cosmwasm_schema::{cw_serde, QueryResponses};
use thiserror::Error;
use utils::{ExecuteCallback, InstantiateCallback, Query};

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
