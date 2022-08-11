use shade_protocol::admin::errors::unauthorized_super;
use shade_protocol::c_std::{
    entry_point, to_binary, Addr, Api, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
    StdResult, Storage,
};
use shade_protocol::admin::{
    AdminAuthStatus, AdminsResponse, ConfigResponse, ExecuteMsg,
    InstantiateMsg, PermissionsResponse, QueryMsg, RegistryAction, ValidateAdminPermissionResponse,
};

use crate::execute::{try_update_registry, try_update_registry_bulk, try_transfer_super, try_self_destruct, try_toggle_status};
use crate::query::query_validate_permission;
use crate::shared::{SUPER, ADMINS, STATUS, PERMISSIONS};


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let super_admin = msg.super_admin.unwrap_or_else(|| info.sender.to_string());
    let super_admin_addr = deps.api.addr_validate(super_admin.as_str())?;
    SUPER.save(deps.storage, &super_admin_addr)?;

    ADMINS.save(deps.storage, &Vec::new())?;
    STATUS.save(deps.storage, &AdminAuthStatus::Active)?;

    let res = Response::new()
        .add_attribute("action", "initialized")
        .add_attribute("superadmin", &info.sender);
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    // Only the super user can execute anything on this contract.
    is_super(deps.storage, &info.sender)?;
    // Super user is assumed to have been verified by this point.
    match msg {
        ExecuteMsg::UpdateRegistry { action } => {
            try_update_registry(deps.storage, deps.api, action)
        }
        ExecuteMsg::UpdateRegistryBulk { actions } => try_update_registry_bulk(deps, actions),
        ExecuteMsg::TransferSuper { new_super } => try_transfer_super(deps, new_super),
        ExecuteMsg::SelfDestruct {} => try_self_destruct(deps),
        ExecuteMsg::ToggleStatus { new_status } => try_toggle_status(deps, new_status),
    }
}

fn is_super(storage: &dyn Storage, address: &Addr) -> StdResult<()> {
    let super_admin = SUPER.load(storage)?;
    if super_admin == *address {
        Ok(())
    } else {
        Err(unauthorized_super(super_admin.as_str()))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::GetConfig {} => Ok(to_binary(&ConfigResponse {
            super_admin: SUPER.load(deps.storage)?,
            status: STATUS.load(deps.storage)?,
        })?),
        QueryMsg::ValidateAdminPermission { permission, user } => Ok(to_binary(
            &query_validate_permission(deps, permission, user)?,
        )?),
        QueryMsg::GetAdmins {} => {
            STATUS
                .load(deps.storage)?
                .not_shutdown()?
                .not_under_maintenance()?;
            Ok(to_binary(&AdminsResponse {
                admins: ADMINS.load(deps.storage)?,
            })?)
        }
        QueryMsg::GetPermissions { user } => {
            STATUS
                .load(deps.storage)?
                .not_shutdown()?
                .not_under_maintenance()?;
            let validated_user = deps.api.addr_validate(user.as_str())?;
            Ok(to_binary(&PermissionsResponse {
                permissions: PERMISSIONS.load(deps.storage, &validated_user)?,
            })?)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::shared::is_valid_permission;
    use rstest::*;

    #[rstest]
    #[case("test", false)]
    #[case("VAULT_", false)]
    #[case("VAULT_TARGET", true)]
    #[case("VAULT_TARG3T_2", true)]
    #[case("", false)]
    #[case("*@#$*!*#!#!#****", false)]
    #[case("VAULT_TARGET_addr", false)]
    fn test_is_valid_permission(#[case] permission: String, #[case] is_valid: bool) {
        let resp = is_valid_permission(permission.as_str());
        if is_valid {
            assert!(resp.is_ok());
        } else {
            assert!(resp.is_err());
        }
    }
}
