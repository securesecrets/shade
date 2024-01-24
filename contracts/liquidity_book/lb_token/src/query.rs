use std::collections::BTreeSet;

use cosmwasm_std::{
    entry_point,
    to_binary,
    Addr,
    Binary,
    BlockInfo,
    Deps,
    Env,
    // debug_print,
    StdError,
    StdResult,
    Timestamp,
    Uint256,
};

use crate::state::{
    balances_r,
    blockinfo_r,
    contr_conf_r,
    get_receiver_hash,
    permissions::{list_owner_permission_keys, may_load_any_permission},
    tkn_info_r,
    tkn_tot_supply_r,
    txhistory::{get_txs, may_get_current_owner},
    PREFIX_REVOKED_PERMITS,
};

use shade_protocol::{
    lb_libraries::lb_token::{
        permissions::{Permission, PermissionKey},
        state_structs::OwnerBalance,
    },
    liquidity_book::lb_token::{QueryAnswer, QueryMsg, QueryWithPermit},
    s_toolkit::{
        permit::{validate, Permit, TokenPermissions},
        viewing_key::{ViewingKey, ViewingKeyStore},
    },
};
/////////////////////////////////////////////////////////////////////////////////
// Queries
/////////////////////////////////////////////////////////////////////////////////
pub fn query_contract_info(deps: Deps) -> StdResult<Binary> {
    let contr_conf = contr_conf_r(deps.storage).load()?;
    let response = QueryAnswer::TokenContractInfo {
        admin: contr_conf.admin,
        curators: contr_conf.curators,
        all_token_ids: contr_conf.token_id_list,
    };
    to_binary(&response)
}

pub fn query_id_balance(deps: Deps, token_id: String) -> StdResult<Binary> {
    let id_balance_raw = tkn_tot_supply_r(deps.storage).load(token_id.as_bytes());

    let mut id_balance = Uint256::zero();

    if id_balance_raw.is_ok() {
        id_balance = id_balance_raw?;
    }

    let response = QueryAnswer::IdTotalBalance { amount: id_balance };
    to_binary(&response)
}

pub fn query_token_id_public_info(deps: Deps, token_id: String) -> StdResult<Binary> {
    let tkn_info_op = tkn_info_r(deps.storage).may_load(token_id.as_bytes())?;
    match tkn_info_op {
        None => Err(StdError::generic_err(format!(
            "token_id {} does not exist",
            token_id
        ))),
        Some(mut tkn_info) => {
            // add owner if owner_is_public == true
            let owner: Option<Addr> = if tkn_info.token_config.flatten().owner_is_public {
                may_get_current_owner(deps.storage, &token_id)?
            } else {
                None
            };

            // add public supply if public_total_supply == true
            let total_supply: Option<Uint256> =
                if tkn_info.token_config.flatten().public_total_supply {
                    Some(tkn_tot_supply_r(deps.storage).load(token_id.as_bytes())?)
                } else {
                    None
                };

            // private_metadata always == None for public info query
            tkn_info.private_metadata = None;
            let response = QueryAnswer::TokenIdPublicInfo {
                token_id_info: tkn_info,
                total_supply,
                owner,
            };
            to_binary(&response)
        }
    }
}

pub fn query_token_id_private_info(
    deps: Deps,
    viewer: &Addr,
    token_id: String,
) -> StdResult<Binary> {
    let tkn_info_op = tkn_info_r(deps.storage).may_load(token_id.as_bytes())?;
    if tkn_info_op.is_none() {
        return Err(StdError::generic_err(format!(
            "token_id {} does not exist",
            token_id
        )));
    }

    let mut tkn_info = tkn_info_op.unwrap();

    // add owner if owner_is_public == true
    let owner: Option<Addr> = if tkn_info.token_config.flatten().owner_is_public {
        may_get_current_owner(deps.storage, &token_id)?
    } else {
        None
    };

    // private metadata is viewable if viewer owns at least 1 token
    let viewer_owns_some_tokens =
        match balances_r(deps.storage, &token_id).may_load(to_binary(&viewer)?.as_slice())? {
            None => false,
            Some(i) if i == Uint256::from(0_u64) => false,
            Some(i) if i > Uint256::from(0_u64) => true,
            Some(_) => unreachable!("should not reach here"),
        };

    // If request owns at least 1 token, can view `private_metadata`. Otherwise check viewership permissions (permission only applicable to nfts, as
    // fungible tokens have no current `owner`).
    if !viewer_owns_some_tokens {
        let permission_op = may_load_any_permission(
            deps.storage,
            // if no owner, = "" ie blank string => will not have any permission
            owner.as_ref().unwrap_or(&Addr::unchecked("".to_string())),
            &token_id,
            viewer,
        )?;
        match permission_op {
            None => {
                return Err(StdError::generic_err(
                    "you do have have permission to view private token info",
                ));
            }
            Some(perm) => {
                let block: BlockInfo =
                    blockinfo_r(deps.storage)
                        .may_load()?
                        .unwrap_or_else(|| BlockInfo {
                            height: 1,
                            time: Timestamp::from_seconds(1),
                            chain_id: "not used".to_string(),
                            random: None,
                        });
                if !perm.check_view_pr_metadata_perm(&block) {
                    tkn_info.private_metadata = None
                };
            }
        }
    }

    // add public supply if public_total_supply == true
    let total_supply: Option<Uint256> = if tkn_info.token_config.flatten().public_total_supply {
        Some(tkn_tot_supply_r(deps.storage).load(token_id.as_bytes())?)
    } else {
        None
    };

    let response = QueryAnswer::TokenIdPrivateInfo {
        token_id_info: tkn_info,
        total_supply,
        owner,
    };
    to_binary(&response)
}

pub fn query_registered_code_hash(deps: Deps, contract: Addr) -> StdResult<Binary> {
    let may_hash_res = get_receiver_hash(deps.storage, &contract);
    let response: QueryAnswer = match may_hash_res {
        Some(hash_res) => QueryAnswer::RegisteredCodeHash {
            code_hash: Some(hash_res?),
        },
        None => QueryAnswer::RegisteredCodeHash { code_hash: None },
    };

    to_binary(&response)
}

pub fn query_balance(
    deps: Deps,
    owner: &Addr,
    viewer: &Addr,
    token_id: String,
) -> StdResult<Binary> {
    if owner != viewer {
        let permission_op = may_load_any_permission(deps.storage, owner, &token_id, viewer)?;
        match permission_op {
            None => {
                return Err(StdError::generic_err(
                    "you do have have permission to view balance",
                ));
            }
            Some(perm) => {
                let block: BlockInfo =
                    blockinfo_r(deps.storage)
                        .may_load()?
                        .unwrap_or_else(|| BlockInfo {
                            height: 1,
                            time: Timestamp::from_seconds(1),
                            chain_id: "not used".to_string(),
                            random: None,
                        });
                if !perm.check_view_balance_perm(&block) {
                    return Err(StdError::generic_err(
                        "you do have have permission to view balance",
                    ));
                } else {
                }
            }
        }
    }

    let owner_canon = deps.api.addr_canonicalize(owner.as_str())?;
    let amount_op = balances_r(deps.storage, &token_id)
        .may_load(to_binary(&deps.api.addr_humanize(&owner_canon)?)?.as_slice())?;
    let amount = match amount_op {
        Some(i) => i,
        None => Uint256::from(0_u64),
    };
    let response = QueryAnswer::Balance { amount };
    to_binary(&response)
}

pub fn query_all_balances(
    deps: Deps,
    account: &Addr,
    tx_history_page: Option<u32>,
    tx_history_page_size: Option<u32>,
) -> StdResult<Binary> {
    let address = deps.api.addr_canonicalize(account.as_str())?;
    let (txs, _total) = get_txs(
        deps.api,
        deps.storage,
        &address,
        tx_history_page.unwrap_or(0u32),
        tx_history_page_size.unwrap_or(u32::MAX),
    )?;

    // create unique list of token_ids that owner has potentially owned. BtreeSet used (rather than Hashset) to have a predictable order
    let token_ids = txs
        .into_iter()
        .map(|tx| tx.token_id)
        .collect::<BTreeSet<_>>();

    // get balances for this list of token_ids, only if balance == Some(_), ie: user has had some balance before
    let mut balances: Vec<OwnerBalance> = vec![];
    for token_id in token_ids.into_iter() {
        let amount = balances_r(deps.storage, &token_id)
            .may_load(to_binary(account).unwrap().as_slice())
            .unwrap();
        if let Some(i) = amount {
            // LB change
            if !i.is_zero() {
                balances.push(OwnerBalance {
                    token_id,
                    amount: i,
                })
            }
        }
    }

    let response = QueryAnswer::AllBalances(balances);
    to_binary(&response)
}

pub fn query_transactions(
    deps: Deps,
    account: &Addr,
    page: u32,
    page_size: u32,
) -> StdResult<Binary> {
    let address = deps.api.addr_canonicalize(account.as_str())?;
    let (txs, total) = get_txs(deps.api, deps.storage, &address, page, page_size)?;

    let response = QueryAnswer::TransactionHistory { txs, total };
    to_binary(&response)
}

pub fn query_permission(
    deps: Deps,
    token_id: String,
    owner: Addr,
    allowed_addr: Addr,
) -> StdResult<Binary> {
    let permission = may_load_any_permission(deps.storage, &owner, &token_id, &allowed_addr)?;

    let response = QueryAnswer::Permission(permission);
    to_binary(&response)
}

pub fn query_all_permissions(
    deps: Deps,
    account: &Addr,
    page: u32,
    page_size: u32,
) -> StdResult<Binary> {
    let (permission_keys, total) =
        list_owner_permission_keys(deps.storage, account, page, page_size)?;
    let mut permissions: Vec<Permission> = vec![];
    let mut valid_pkeys: Vec<PermissionKey> = vec![];
    for pkey in permission_keys {
        let permission =
            may_load_any_permission(deps.storage, account, &pkey.token_id, &pkey.allowed_addr)?;
        if let Some(i) = permission {
            permissions.push(i);
            valid_pkeys.push(pkey);
        };
    }

    let response = QueryAnswer::AllPermissions {
        permission_keys: valid_pkeys,
        permissions,
        total,
    };
    to_binary(&response)
}
