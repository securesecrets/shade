use cosmwasm_std::{
    entry_point, to_binary, Addr, Api, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response,
    StdResult, Storage,
};
use shade_admin::admin::{
    AdminAuthError, AdminAuthResult, AdminAuthStatus, AdminsResponse, ConfigResponse, ExecuteMsg,
    InstantiateMsg, PermissionsResponse, QueryMsg, RegistryAction, ValidateAdminPermissionResponse,
};
use shade_admin::storage::{Item, Map};

/// Maps user to permissions for which they have user.
const PERMISSIONS: Map<&Addr, Vec<String>> = Map::new("permissions");
/// List of all admins.
const ADMINS: Item<Vec<Addr>> = Item::new("admins");
/// Super user.
const SUPER: Item<Addr> = Item::new("super");
/// Whether or not this contract can be consumed.
const STATUS: Item<AdminAuthStatus> = Item::new("is_active");

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
) -> AdminAuthResult<Response> {
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

fn is_valid_permission(permission: &str) -> AdminAuthResult<()> {
    if permission.len() <= 10 {
        return Err(AdminAuthError::InvalidPermissionFormat {
            permission: permission.to_string(),
        });
    }
    let valid_chars = permission.bytes().all(|byte| {
        (b'A'..=b'Z').contains(&byte) || (b'0'..=b'9').contains(&byte) || b'_'.eq(&byte)
    });
    if !valid_chars {
        return Err(AdminAuthError::InvalidPermissionFormat {
            permission: permission.to_string(),
        });
    }
    Ok(())
}

fn resolve_registry_action(
    store: &mut dyn Storage,
    admins: &mut Vec<Addr>,
    api: &dyn Api,
    action: RegistryAction,
) -> AdminAuthResult<()> {
    match action {
        RegistryAction::RegisterAdmin { user } => register_admin(store, admins, api, user),
        RegistryAction::GrantAccess { permissions, user } => {
            grant_access(store, api, admins, permissions, user)
        }
        RegistryAction::RevokeAccess { permissions, user } => {
            revoke_access(store, api, admins, permissions, user)
        }
        RegistryAction::DeleteAdmin { user } => delete_admin(store, admins, api, user),
    }?;
    Ok(())
}

/// Performs one registry update. Cannot be run during a shutdown.
fn try_update_registry(
    store: &mut dyn Storage,
    api: &dyn Api,
    action: RegistryAction,
) -> AdminAuthResult<Response> {
    STATUS.load(store)?.not_shutdown()?;
    let mut admins = ADMINS.load(store)?;
    resolve_registry_action(store, &mut admins, api, action)?;
    ADMINS.save(store, &admins)?;
    Ok(Response::default())
}

/// Performs bulk registry updates. Cannot be run during a shutdown.
fn try_update_registry_bulk(
    deps: DepsMut,
    actions: Vec<RegistryAction>,
) -> AdminAuthResult<Response> {
    STATUS.load(deps.storage)?.not_shutdown()?;
    let mut admins = ADMINS.load(deps.storage)?;
    for action in actions {
        resolve_registry_action(deps.storage, &mut admins, deps.api, action)?;
    }
    ADMINS.save(deps.storage, &admins)?;
    Ok(Response::default())
}

fn register_admin(
    store: &mut dyn Storage,
    admins: &mut Vec<Addr>,
    api: &dyn Api,
    user: String,
) -> AdminAuthResult<()> {
    let user_addr = api.addr_validate(user.as_str())?;
    if !admins.contains(&user_addr) {
        // Create an empty permissions for them and add their address to the registered array.
        admins.push(user_addr.clone());
        PERMISSIONS.save(store, &user_addr, &vec![])?;
    };
    Ok(())
}

fn delete_admin(
    store: &mut dyn Storage,
    admins: &mut Vec<Addr>,
    api: &dyn Api,
    user: String,
) -> AdminAuthResult<()> {
    let user_addr = api.addr_validate(user.as_str())?;
    if admins.contains(&user_addr) {
        // Delete admin from list.
        admins.retain(|x| x.ne(&user_addr));
        // Clear their permissions.
        PERMISSIONS.save(store, &user_addr, &vec![])?;
    };
    Ok(())
}

fn verify_registered(admins: &[Addr], user: &Addr) -> AdminAuthResult<()> {
    if !admins.contains(user) {
        return Err(AdminAuthError::UnregisteredAdmin { user: user.clone() });
    }
    Ok(())
}

fn grant_access(
    store: &mut dyn Storage,
    api: &dyn Api,
    admins: &[Addr],
    mut permissions: Vec<String>,
    user: String,
) -> AdminAuthResult<()> {
    let user = api.addr_validate(user.as_str())?;
    validate_permissions(permissions.as_slice())?;
    verify_registered(admins, &user)?;
    PERMISSIONS.update(store, &user, |old_perms| -> AdminAuthResult<_> {
        match old_perms {
            Some(mut old_perms) => {
                permissions.retain(|c| !old_perms.contains(c));
                old_perms.append(&mut permissions);
                Ok(old_perms)
            }
            None => Err(AdminAuthError::NoPermissions { user: user.clone() }),
        }
    })?;
    Ok(())
}

fn revoke_access(
    store: &mut dyn Storage,
    api: &dyn Api,
    admins: &[Addr],
    permissions: Vec<String>,
    user: String,
) -> AdminAuthResult<()> {
    let user = api.addr_validate(user.as_str())?;
    validate_permissions(permissions.as_slice())?;
    verify_registered(admins, &user)?;
    PERMISSIONS.update(store, &user, |old_perms| -> AdminAuthResult<_> {
        match old_perms {
            Some(mut old_perms) => {
                old_perms.retain(|c| !permissions.contains(c));
                Ok(old_perms)
            }
            None => Err(AdminAuthError::NoPermissions { user: user.clone() }),
        }
    })?;
    Ok(())
}

fn try_transfer_super(deps: DepsMut, new_super: String) -> AdminAuthResult<Response> {
    let valid_super = deps.api.addr_validate(new_super.as_str())?;
    // If you're trying to transfer the super permissions to someone who hasn't been registered as an admin,
    // it won't work. This is a safeguard.
    let mut admins = ADMINS.load(deps.storage)?;
    if !admins.contains(&valid_super) {
        return Err(AdminAuthError::UnregisteredAdmin { user: valid_super });
    } else {
        // Update the super and remove them from the admin list.
        SUPER.save(deps.storage, &valid_super)?;
        delete_admin(deps.storage, &mut admins, deps.api, new_super)?;
        ADMINS.save(deps.storage, &admins)?;
    }
    Ok(Response::default())
}

fn try_self_destruct(deps: DepsMut) -> AdminAuthResult<Response> {
    STATUS.load(deps.storage)?.not_shutdown()?;
    // Clear permissions
    let admins = ADMINS.load(deps.storage)?;
    admins
        .iter()
        .for_each(|admin| PERMISSIONS.remove(deps.storage, admin));
    // Clear admins
    ADMINS.save(deps.storage, &vec![])?;
    // Disable contract
    STATUS.save(deps.storage, &AdminAuthStatus::Shutdown)?;
    Ok(Response::default())
}

fn try_toggle_status(deps: DepsMut, new_status: AdminAuthStatus) -> AdminAuthResult<Response> {
    STATUS.update(deps.storage, |_| -> StdResult<_> { Ok(new_status) })?;
    Ok(Response::default())
}

fn validate_permissions(permissions: &[String]) -> AdminAuthResult<()> {
    for permission in permissions {
        is_valid_permission(permission.as_str())?;
    }
    Ok(())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> AdminAuthResult<QueryResponse> {
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

fn is_super(storage: &dyn Storage, address: &Addr) -> AdminAuthResult<()> {
    let super_admin = SUPER.load(storage)?;
    if super_admin == *address {
        Ok(())
    } else {
        Err(AdminAuthError::UnauthorizedSuper {
            expected_super_admin: super_admin,
        })
    }
}

/// Checks if the user has the requested permission. Permissions are case sensitive.
fn query_validate_permission(
    deps: Deps,
    permission: String,
    user: String,
) -> AdminAuthResult<ValidateAdminPermissionResponse> {
    STATUS
        .load(deps.storage)?
        .not_shutdown()?
        .not_under_maintenance()?;
    is_valid_permission(permission.as_str())?;
    let valid_user = deps.api.addr_validate(user.as_str())?;
    let super_admin = SUPER.load(deps.storage)?;

    let has_permission: bool;

    // Super admin has all permissions. The permissions don't need to have been created and assigned to the super admin beforehand. We do this because we assume that the super admin is secure (like a multi-sig or the main governance contract) so it would be a hassle to whitelist every permission we want them to have.
    if valid_user == super_admin {
        has_permission = true;
    } else {
        let permissions = PERMISSIONS.may_load(deps.storage, &valid_user)?;
        match permissions {
            Some(permissions) => {
                if permissions.iter().any(|perm| permission.eq(perm)) {
                    has_permission = true;
                } else {
                    return Err(AdminAuthError::UnauthorizedAdmin {
                        user: valid_user,
                        permission,
                    });
                }
            }
            // If user has been registered, there should be an empty vector there.
            None => return Err(AdminAuthError::UnregisteredAdmin { user: valid_user }),
        }
    }
    Ok(ValidateAdminPermissionResponse { has_permission })
}

#[cfg(test)]
mod test {
    use super::*;
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
