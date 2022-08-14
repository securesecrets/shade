use crate::{utils::Query, Contract};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{QuerierWrapper, StdError, StdResult};
use shade_admin::admin::{
    AdminsResponse,
    ConfigResponse,
    PermissionsResponse,
    ValidateAdminPermissionResponse,
};

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
    ValidateAdminPermission {
        permission: String,
        user: String,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

pub fn validate_admin<T: Into<String>>(
    querier: &QuerierWrapper,
    permission: AdminPermissions,
    user: T,
    admin_auth: &Contract,
) -> StdResult<()> {
    let admin_resp: StdResult<ValidateAdminPermissionResponse> =
        QueryMsg::ValidateAdminPermission {
            permission: permission.into_string(),
            user: user.into(),
        }
        .query(querier, admin_auth);

    match admin_resp {
        Ok(resp) => match resp.has_permission {
            true => Ok(()),
            false => Err(StdError::generic_err("Unexpected response.")),
        },
        Err(err) => Err(err),
    }
}

pub enum AdminPermissions {
    QueryAuthAdmin,
    ScrtStakingAdmin,
    TreasuryManager,
    TreasuryAdmin,
    StabilityAdmin,
}

// NOTE: SHADE_{CONTRACT_NAME}_{CONTRACT_ROLE}_{POTENTIAL IDs}

impl AdminPermissions {
    pub fn into_string(self) -> String {
        match self {
            AdminPermissions::QueryAuthAdmin => "SHADE_QUERY_AUTH_ADMIN",
            AdminPermissions::ScrtStakingAdmin => "SHADE_SCRT_STAKING_ADMIN",
            AdminPermissions::TreasuryManager => "SHADE_TREASURY_MANAGER",
            AdminPermissions::TreasuryAdmin => "SHADE_TREASURY_ADMIN",
            AdminPermissions::StabilityAdmin => "SHADE_STABILITY_ADMIN",
        }
        .to_string()
    }
}
