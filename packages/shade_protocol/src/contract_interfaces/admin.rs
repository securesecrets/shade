//! Refers to this https://github.com/securesecrets/shadeadmin
use cosmwasm_std::{QuerierWrapper, StdError, StdResult, Addr};
use shade_admin::admin::{ConfigResponse, AdminsResponse, PermissionsResponse, ValidateAdminPermissionResponse};
use cosmwasm_schema::{cw_serde, QueryResponses};
use crate::Contract;
use crate::utils::Query;


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

/// Returns an error if the user does not have the passed permission.
pub fn validate_permission(
    querier: &QuerierWrapper,
    permission: &str,
    user: &Addr,
    admin_auth: &Contract,
) -> StdResult<()> {
    let admin_resp: StdResult<ValidateAdminPermissionResponse> = QueryMsg::ValidateAdminPermission {
        permission: permission.to_string(),
        user: user.to_string().clone(),
    }.query(querier, admin_auth);

    match admin_resp {
        Ok(resp) => match resp.has_permission {
            true => Ok(()),
            false => Err(StdError::generic_err("Unexpected response.")),
        },
        Err(err) => Err(err),
    }
}
