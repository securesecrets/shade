use crate::{
    admin::{QueryMsg, ValidateAdminPermissionResponse},
    utils::Query,
    Contract,
};
use cosmwasm_std::{QuerierWrapper, StdError, StdResult};

// TODO: move relevant stuff from admin.rs over to here and delete that file
pub fn validate_admin<T: Into<String>>(
    querier: &QuerierWrapper,
    permission: AdminPermissions,
    user: T,
    contract: T,
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
}

// NOTE: SHADE_{CONTRACT_NAME}_{CONTRACT_ROLE}_{POTENTIAL IDs}

impl AdminPermissions {
    pub fn into_string(self) -> String {
        match self {
            AdminPermissions::QueryAuthAdmin => "SHADE_QUERY_AUTH_ADMIN",
            AdminPermissions::ScrtStakingAdmin => "SHADE_SCRT_STAKING_ADMIN",
            AdminPermissions::TreasuryManager => "SHADE_TREASURY_MANAGER",
            AdminPermissions::TreasuryAdmin => "SHADE_TREASURY_ADMIN",
        }
        .to_string()
    }
}
