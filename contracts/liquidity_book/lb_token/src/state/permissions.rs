use super::*;

use cosmwasm_std::{to_binary, Addr, StdError, StdResult, Storage};

use lb_libraries::lb_token::permissions::PermissionKey;
use shade_protocol::s_toolkit::storage::AppendStore;

pub static PERMISSION_ID_STORE: AppendStore<PermissionKey> = AppendStore::new(PREFIX_PERMISSION_ID);

/////////////////////////////////////////////////////////////////////////////////
// Permissions
/////////////////////////////////////////////////////////////////////////////////

/// saves new permission entry and adds identifier to the list of permissions the owner address has
pub fn new_permission(
    storage: &mut dyn Storage,
    owner: &Addr,
    token_id: &str,
    allowed_addr: &Addr,
    // permission_key: &PermissionKey,
    permission: &Permission,
) -> StdResult<()> {
    // store permission
    permission_w(storage, owner, token_id).save(to_binary(allowed_addr)?.as_slice(), permission)?;

    // add permission to list of permissions for a given owner
    append_permission_for_addr(storage, owner, token_id, allowed_addr)?;

    Ok(())
}

// /// updates an existing permission entry. Does not check that existing entry exists, so
// /// riskier to use this. But saves gas from potentially loading permission twice
// pub fn update_permission_unchecked(
//     storage: &mut dyn Storage,
//     owner: &Addr,
//     token_id: &str,
//     allowed_addr: &Addr,
//     permission: &Permission,
// ) -> StdResult<()> {
//     permission_w(storage, owner, token_id).save(
//         to_binary(allowed_addr)?.as_slice(),
//         permission
//     )?;

//     Ok(())
// }

/// updates an existing permission entry. Returns error if permission entry does not aleady exist
pub fn update_permission(
    storage: &mut dyn Storage,
    owner: &Addr,
    token_id: &str,
    allowed_addr: &Addr,
    permission: &Permission, // update_action: A,
) -> StdResult<()>
// where
    // S: Storage, 
    // A: FnOnce(Option<Permission>) -> StdResult<Permission>
{
    let update_action = |perm: Option<Permission>| -> StdResult<Permission> {
        match perm {
            Some(_) => Ok(permission.clone()),
            None => Err(StdError::generic_err(
                "cannot update or revoke a non-existent permission entry",
            )),
        }
    };

    permission_w(storage, owner, token_id)
        .update(to_binary(allowed_addr)?.as_slice(), update_action)?;

    Ok(())
}

/// returns StdResult<Option<Permission>> for a given [`owner`, `token_id`, `allowed_addr`] combination.
/// Returns "dormant" permissions we well, ie: where owner doesn't currently own tokens.
/// If permission does not exist -> returns StdResult<None>
pub fn may_load_any_permission(
    storage: &dyn Storage,
    owner: &Addr,
    token_id: &str,
    allowed_addr: &Addr,
) -> StdResult<Option<Permission>> {
    permission_r(storage, owner, token_id).may_load(to_binary(allowed_addr)?.as_slice())
}

// /// returns StdResult<Option<Permission>> for a given [`owner`, `token_id`, `allowed_addr`] combination.
// /// If (permission does not exist) || (owner no longer owns tokens) () -> returns StdResult<None>
// pub fn may_load_active_permission(
//     storage: &dyn Storage,
//     owner: &Addr,
//     token_id: &str,
//     allowed_addr: &Addr,
// ) -> StdResult<Option<Permission>> {
//     let permission = permission_r(storage, owner, token_id).may_load(to_binary(allowed_addr)?.as_slice())?;
//     let owner_amount = balances_r(storage, token_id).may_load(to_binary(owner)?.as_slice())?;
//     match owner_amount {
//         None =>  return Ok(None),
//         Some(i) if i == Uint256(0) => return Ok(None),
//         Some(i) if i > Uint256(0) => return Ok(permission),
//         Some(_) => unreachable!("may_load_permission: this should be unreachable")
//     }
// }

/// Return (Vec<`PermissionKey { token_id, allowed_addr }`>, u64)
/// returns a list and total number of PermissionKeys for a given owner. The PermissionKeys represents (part of)
/// the keys to retrieve all permissions an `owner` has currently granted
pub fn list_owner_permission_keys(
    storage: &dyn Storage,
    owner: &Addr,
    page: u32,
    page_size: u32,
) -> StdResult<(Vec<PermissionKey>, u64)> {
    let owner_store = PERMISSION_ID_STORE.add_suffix(to_binary(owner)?.as_slice());

    // let store = ReadonlyPrefixedStorage::multilevel(&[PREFIX_PERMISSION_ID, to_binary(owner)?.as_slice()], storage);

    // Try to access the storage of PermissionKeys for the account.
    // If it doesn't exist yet, return an empty list of transfers.
    // let store = AppendStore::<PermissionKey, _, _>::attach(&store);
    // let store = if let Some(result) = store {
    //     result?
    // } else {
    //     return Ok((vec![], 0));
    // };

    // Take `page_size` starting from the latest entry, potentially skipping `page * page_size`
    // entries from the start.
    let pkeys_iter = owner_store
        .iter(storage)?
        .rev()
        .skip((page * page_size) as _)
        .take(page_size as _);

    // Transform iterator to a `Vec<PermissionKey>`
    let pkeys: StdResult<Vec<PermissionKey>> = pkeys_iter
        // .map(|pkey| pkey)
        .collect();
    // return `(Vec<PermissionKey> , total_permission)`
    pkeys.map(|pkeys| {
        (
            pkeys,
            owner_store.get_len(storage).unwrap_or_default() as u64,
        )
    })
}

/// stores a `PermissionKey {token_id: String, allowed_addr: String]` for a given `owner`. Note that
/// permission key is [`owner`, `token_id`, `allowed_addr`]. This function does not enforce that the
/// list of PermissionKey stored is unique; while this doesn't really matter, the ref implementation's
/// functions aim to ensure each entry is unique, for storage efficiency.
fn append_permission_for_addr(
    storage: &mut dyn Storage,
    owner: &Addr,
    token_id: &str,
    allowed_addr: &Addr,
) -> StdResult<()> {
    let permission_key = PermissionKey {
        token_id: token_id.to_string(),
        allowed_addr: allowed_addr.clone(),
    };
    let owner_store = PERMISSION_ID_STORE.add_suffix(to_binary(owner)?.as_slice());
    owner_store.push(storage, &permission_key)
}
