use shade_protocol::{
    admin::{
        errors::unauthorized_super, AdminAuthStatus, AdminsResponse, ConfigResponse, ExecuteMsg,
        InstantiateMsg, PermissionsResponse, QueryMsg,
    },
    c_std::{
        shd_entry_point, to_binary, Addr, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
        StdResult, Storage,
    },
    utils::pad_handle_result,
};

use crate::{
    execute::{
        try_self_destruct, try_toggle_status, try_transfer_super, try_update_registry,
        try_update_registry_bulk,
    },
    query::query_validate_permission,
    shared::{ADMINS, PERMISSIONS, STATUS, SUPER},
};

pub const RESPONSE_BLOCK_SIZE: usize = 256;

#[cfg_attr(not(feature = "library"), shd_entry_point)]
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

#[cfg_attr(not(feature = "library"), shd_entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    // Only the super user can execute anything on this contract.
    is_super(deps.storage, &info.sender)?;
    // Super user is assumed to have been verified by this point.
    pad_handle_result(
        match msg {
            ExecuteMsg::UpdateRegistry { action } => {
                try_update_registry(deps.storage, deps.api, action)
            }
            ExecuteMsg::UpdateRegistryBulk { actions } => try_update_registry_bulk(deps, actions),
            ExecuteMsg::TransferSuper { new_super } => try_transfer_super(deps, new_super),
            ExecuteMsg::SelfDestruct {} => try_self_destruct(deps),
            ExecuteMsg::ToggleStatus { new_status } => try_toggle_status(deps, new_status),
        },
        RESPONSE_BLOCK_SIZE,
    )
}

fn is_super(storage: &dyn Storage, address: &Addr) -> StdResult<()> {
    let super_admin = SUPER.load(storage)?;
    if super_admin == *address {
        Ok(())
    } else {
        Err(unauthorized_super(address.as_str()))
    }
}

#[cfg_attr(not(feature = "library"), shd_entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    Ok(match msg {
        QueryMsg::GetConfig {} => to_binary(&ConfigResponse {
            super_admin: SUPER.load(deps.storage)?,
            status: STATUS.load(deps.storage)?,
        }),
        QueryMsg::ValidateAdminPermission { permission, user } => {
            to_binary(&query_validate_permission(deps, permission, user)?)
        }
        QueryMsg::GetAdmins {} => {
            STATUS
                .load(deps.storage)?
                .not_shutdown()?
                .not_under_maintenance()?;
            to_binary(&AdminsResponse {
                admins: ADMINS.load(deps.storage)?,
            })
        }
        QueryMsg::GetPermissions { user } => {
            STATUS
                .load(deps.storage)?
                .not_shutdown()?
                .not_under_maintenance()?;
            let validated_user = deps.api.addr_validate(user.as_str())?;
            to_binary(&PermissionsResponse {
                permissions: PERMISSIONS.load(deps.storage, &validated_user)?,
            })
        }
    }?)
}
