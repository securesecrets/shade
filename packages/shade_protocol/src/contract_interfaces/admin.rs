use cosmwasm_std::{QuerierWrapper, StdError, StdResult};
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
    ValidateAdminPermission { contract: String, user: String },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

pub fn validate_admin(
    querier: &QuerierWrapper,
    contract: String,
    user: String,
    admin_auth: &Contract,
) -> StdResult<()> {
    let admin_resp: StdResult<ValidateAdminPermissionResponse> = QueryMsg::ValidateAdminPermission {
        contract,
        user,
    }.query(querier, admin_auth);

    match admin_resp {
        Ok(resp) => match resp.is_admin {
            true => Ok(()),
            false => Err(StdError::generic_err("Unexpected response.")),
        },
        Err(err) => Err(err),
    }
}