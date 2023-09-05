use crate::shared::{validate_permissions, ADMINS, PERMISSIONS, STATUS, SUPER};
use shade_protocol::admin::errors::{no_permission, unregistered_admin};
use shade_protocol::admin::{AdminAuthStatus, RegistryAction};
use shade_protocol::c_std::{Addr, Api, DepsMut, Response, StdResult, Storage};

/// Performs one registry update. Cannot be run during a shutdown.
pub fn try_update_registry(
    store: &mut dyn Storage,
    api: &dyn Api,
    action: RegistryAction,
) -> StdResult<Response> {
    STATUS.load(store)?.not_shutdown()?;
    let mut admins = ADMINS.load(store)?;
    resolve_registry_action(store, &mut admins, api, action)?;
    ADMINS.save(store, &admins)?;
    Ok(Response::default())
}

/// Performs bulk registry updates. Cannot be run during a shutdown.
pub fn try_update_registry_bulk(
    deps: DepsMut,
    actions: Vec<RegistryAction>,
) -> StdResult<Response> {
    STATUS.load(deps.storage)?.not_shutdown()?;
    let mut admins = ADMINS.load(deps.storage)?;
    for action in actions {
        resolve_registry_action(deps.storage, &mut admins, deps.api, action)?;
    }
    ADMINS.save(deps.storage, &admins)?;
    Ok(Response::default())
}

pub fn try_transfer_super(deps: DepsMut, new_super: String) -> StdResult<Response> {
    let valid_super = deps.api.addr_validate(new_super.as_str())?;
    // If you're trying to transfer the super permissions to someone who hasn't been registered as an admin,
    // it won't work. This is a safeguard.
    let mut admins = ADMINS.load(deps.storage)?;
    if !admins.contains(&valid_super) {
        return Err(unregistered_admin(valid_super.as_str()));
    } else {
        // Update the super and remove them from the admin list.
        SUPER.save(deps.storage, &valid_super)?;
        delete_admin(deps.storage, &mut admins, deps.api, new_super)?;
        ADMINS.save(deps.storage, &admins)?;
    }
    Ok(Response::default())
}

pub fn try_self_destruct(deps: DepsMut) -> StdResult<Response> {
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

pub fn try_toggle_status(deps: DepsMut, new_status: AdminAuthStatus) -> StdResult<Response> {
    STATUS.update(deps.storage, |_| -> StdResult<_> { Ok(new_status) })?;
    Ok(Response::default())
}

fn resolve_registry_action(
    store: &mut dyn Storage,
    admins: &mut Vec<Addr>,
    api: &dyn Api,
    action: RegistryAction,
) -> StdResult<()> {
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

fn register_admin(
    store: &mut dyn Storage,
    admins: &mut Vec<Addr>,
    api: &dyn Api,
    user: String,
) -> StdResult<()> {
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
) -> StdResult<()> {
    let user_addr = api.addr_validate(user.as_str())?;
    if admins.contains(&user_addr) {
        // Delete admin from list.
        admins.retain(|x| x.ne(&user_addr));
        // Delete their permissions.
        PERMISSIONS.remove(store, &user_addr);
    };
    Ok(())
}

fn grant_access(
    store: &mut dyn Storage,
    api: &dyn Api,
    admins: &[Addr],
    mut permissions: Vec<String>,
    user: String,
) -> StdResult<()> {
    let user = api.addr_validate(user.as_str())?;
    validate_permissions(permissions.as_slice())?;
    verify_registered(admins, &user)?;
    PERMISSIONS.update(store, &user, |old_perms| -> StdResult<_> {
        match old_perms {
            Some(mut old_perms) => {
                permissions.retain(|c| !old_perms.contains(c));
                old_perms.append(&mut permissions);
                Ok(old_perms)
            }
            None => Err(no_permission(user.as_str())),
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
) -> StdResult<()> {
    let user = api.addr_validate(user.as_str())?;
    validate_permissions(permissions.as_slice())?;
    verify_registered(admins, &user)?;
    PERMISSIONS.update(store, &user, |old_perms| -> StdResult<_> {
        match old_perms {
            Some(mut old_perms) => {
                old_perms.retain(|c| !permissions.contains(c));
                Ok(old_perms)
            }
            None => Err(no_permission(user.as_str())),
        }
    })?;
    Ok(())
}

fn verify_registered(admins: &[Addr], user: &Addr) -> StdResult<()> {
    if !admins.contains(user) {
        return Err(no_permission(user.as_str()));
    }
    Ok(())
}
