// use base64::{engine::general_purpose, Engine as _};
use cosmwasm_std::{
    entry_point,
    // debug_print,
    to_binary,
    Binary,
    Deps,
    DepsMut,
    Env,
    MessageInfo,
    Response,
    StdError,
    StdResult,
};

use crate::{execute::*, query::*};

use crate::state::{blockinfo_w, contr_conf_r, contr_conf_w, PREFIX_REVOKED_PERMITS};

use shade_protocol::{
    lb_libraries::lb_token::state_structs::ContractConfig,
    liquidity_book::lb_token::{ExecuteMsg, InstantiateMsg, SendAction},
    s_toolkit::{
        crypto::sha_256,
        permit::{validate, Permit, TokenPermissions},
        viewing_key::{ViewingKey, ViewingKeyStore},
    },
};
/////////////////////////////////////////////////////////////////////////////////
// Init
/////////////////////////////////////////////////////////////////////////////////

/// instantiation function. See [InitMsg](crate::msg::InitMsg) for the api
#[entry_point]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // save latest block info. not necessary once we migrate to CosmWasm v1.0
    blockinfo_w(deps.storage).save(&env.block)?;

    // set admin. If `has_admin` == None => no admin.
    // If `has_admin` == true && msg.admin == None => admin is the instantiator
    let admin = match msg.has_admin {
        false => None,
        true => match msg.admin {
            Some(i) => Some(i),
            None => Some(info.sender.clone()),
        },
    };

    // create contract config -- save later
    let prng_seed_hashed = sha_256(msg.entropy.as_bytes());
    let prng_seed = prng_seed_hashed.to_vec();
    // let prng_seed = sha_256(
    //     general_purpose::STANDARD
    //         .encode(msg.entropy.as_str())
    //         .as_bytes(),
    // );

    ViewingKey::set_seed(deps.storage, &prng_seed);

    let mut config = ContractConfig {
        admin,
        curators: msg.curators,
        token_id_list: vec![],
        tx_cnt: 0u64,
        prng_seed: prng_seed.to_vec(),
        contract_address: env.contract.address.clone(),
        lb_pair_info: msg.lb_pair_info,
    };

    // // set initial balances
    for initial_token in msg.initial_tokens {
        exec_curate_token_id(&mut deps, &env, &info, &mut config, initial_token, None)?;
    }

    // save contract config -- where tx_cnt would have increased post initial balances
    contr_conf_w(deps.storage).save(&config)?;
    let response = Response::new().set_data(to_binary(&env.contract.address)?);

    // deps.api
    //     .debug(format!("Contract address {}", env.contract.address).as_str());
    Ok(response)
}

/////////////////////////////////////////////////////////////////////////////////
// Handles
/////////////////////////////////////////////////////////////////////////////////

/// contract handle function. See [ExecuteMsg](crate::msg::ExecuteMsg) and
/// [ExecuteAnswer](crate::msg::ExecuteAnswer) for the api
#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    // allows approx latest block info to be available for queries. Important to enforce
    // allowance expiration. Remove this after BlockInfo becomes available to queries
    blockinfo_w(deps.storage).save(&env.block)?;

    let response = match msg {
        // ExecuteMsg::CurateTokenIds {
        //     initial_tokens,
        //     memo,
        //     padding: _,
        // } => try_curate_token_ids(deps, env, info, initial_tokens, memo),
        ExecuteMsg::MintTokens {
            mint_tokens,
            memo,
            padding: _,
        } => try_mint_tokens(deps, env, info, mint_tokens, memo),
        ExecuteMsg::BurnTokens {
            burn_tokens,
            memo,
            padding: _,
        } => try_burn_tokens(deps, env, info, burn_tokens, memo),
        ExecuteMsg::ChangeMetadata {
            token_id,
            public_metadata,
            private_metadata,
        } => try_change_metadata(
            deps,
            env,
            info,
            token_id,
            *public_metadata,
            *private_metadata,
        ),
        ExecuteMsg::Transfer {
            token_id,
            from,
            recipient,
            amount,
            memo,
            padding: _,
        } => try_transfer(deps, env, info, token_id, from, recipient, amount, memo),
        ExecuteMsg::BatchTransfer {
            actions,
            padding: _,
        } => try_batch_transfer(deps, env, info, actions),
        ExecuteMsg::Send {
            token_id,
            from,
            recipient,
            recipient_code_hash,
            amount,
            msg,
            memo,
            padding: _,
        } => try_send(deps, env, info, SendAction {
            token_id,
            from,
            recipient,
            recipient_code_hash,
            amount,
            msg,
            memo,
        }),
        ExecuteMsg::BatchSend {
            actions,
            padding: _,
        } => try_batch_send(deps, env, info, actions),
        ExecuteMsg::GivePermission {
            allowed_address,
            token_id,
            view_balance,
            view_balance_expiry,
            view_private_metadata,
            view_private_metadata_expiry,
            transfer,
            transfer_expiry,
            padding: _,
        } => try_give_permission(
            deps,
            env,
            info,
            allowed_address,
            token_id,
            view_balance,
            view_balance_expiry,
            view_private_metadata,
            view_private_metadata_expiry,
            transfer,
            transfer_expiry,
        ),
        ExecuteMsg::RevokePermission {
            token_id,
            owner,
            allowed_address,
            padding: _,
        } => try_revoke_permission(deps, env, info, token_id, owner, allowed_address),
        ExecuteMsg::CreateViewingKey {
            entropy,
            padding: _,
        } => try_create_viewing_key(deps, env, info, entropy),
        ExecuteMsg::SetViewingKey { key, padding: _ } => try_set_viewing_key(deps, env, info, key),
        ExecuteMsg::RevokePermit {
            permit_name,
            padding: _,
        } => try_revoke_permit(deps, env, info, permit_name),
        // ExecuteMsg::AddCurators {
        //     add_curators,
        //     padding: _,
        // } => try_add_curators(deps, env, info, add_curators),
        // ExecuteMsg::RemoveCurators {
        //     remove_curators,
        //     padding: _,
        // } => try_remove_curators(deps, env, info, remove_curators),
        // ExecuteMsg::AddMinters {
        //     token_id,
        //     add_minters,
        //     padding: _,
        // } => try_add_minters(deps, env, info, token_id, add_minters),
        // ExecuteMsg::RemoveMinters {
        //     token_id,
        //     remove_minters,
        //     padding: _,
        // } => try_remove_minters(deps, env, info, token_id, remove_minters),
        ExecuteMsg::ChangeAdmin {
            new_admin,
            padding: _,
        } => try_change_admin(deps, env, info, new_admin),
        ExecuteMsg::RemoveAdmin {
            current_admin,
            contract_address,
            padding: _,
        } => try_remove_admin(deps, env, info, current_admin, contract_address),
        ExecuteMsg::RegisterReceive {
            code_hash,
            padding: _,
        } => try_register_receive(deps, env, info, code_hash),
    };
    pad_response(response)
}

use shade_protocol::liquidity_book::lb_token::{QueryAnswer, QueryMsg, QueryWithPermit};
/////////////////////////////////////////////////////////////////////////////////
// Queries
/////////////////////////////////////////////////////////////////////////////////

/// contract query function. See [QueryMsg](crate::msg::QueryMsg) and
/// [QueryAnswer](crate::msg::QueryAnswer) for the api
#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::IdTotalBalance { id } => query_id_balance(deps, id),
        QueryMsg::ContractInfo {} => query_contract_info(deps),
        QueryMsg::TokenIdPublicInfo { token_id } => query_token_id_public_info(deps, token_id),
        QueryMsg::RegisteredCodeHash { contract } => query_registered_code_hash(deps, contract),
        QueryMsg::WithPermit { permit, query } => permit_queries(deps, permit, query),
        QueryMsg::Balance { .. }
        | QueryMsg::AllBalances { .. }
        | QueryMsg::TransactionHistory { .. }
        | QueryMsg::Permission { .. }
        | QueryMsg::AllPermissions { .. }
        | QueryMsg::TokenIdPrivateInfo { .. } => viewing_keys_queries(deps, msg),
    }
}

fn permit_queries(deps: Deps, permit: Permit, query: QueryWithPermit) -> Result<Binary, StdError> {
    // Validate permit content
    let contract_address = contr_conf_r(deps.storage).load()?.contract_address;

    let account_str = validate(
        deps,
        PREFIX_REVOKED_PERMITS,
        &permit,
        contract_address.to_string(),
        None,
    )?;
    let account = deps.api.addr_validate(&account_str)?;

    if !permit.check_permission(&TokenPermissions::Owner) {
        return Err(StdError::generic_err(format!(
            "`Owner` permit required for SNIP1155 permit queries, got permissions {:?}",
            permit.params.permissions
        )));
    }

    // Permit validated! We can now execute the query.
    match query {
        QueryWithPermit::Balance { owner, token_id } => {
            query_balance(deps, &owner, &account, token_id)
        }
        QueryWithPermit::AllBalances {
            tx_history_page,
            tx_history_page_size,
        } => query_all_balances(deps, &account, tx_history_page, tx_history_page_size),
        QueryWithPermit::TransactionHistory { page, page_size } => {
            query_transactions(deps, &account, page.unwrap_or(0), page_size)
        }
        QueryWithPermit::Permission {
            owner,
            allowed_address,
            token_id,
        } => {
            if account != owner.as_str() && account != allowed_address.as_str() {
                return Err(StdError::generic_err(format!(
                    "Cannot query permission. Requires permit for either owner {:?} or viewer||spender {:?}, got permit for {:?}",
                    owner.as_str(),
                    allowed_address.as_str(),
                    account.as_str()
                )));
            }

            query_permission(deps, token_id, owner, allowed_address)
        }
        QueryWithPermit::AllPermissions { page, page_size } => {
            query_all_permissions(deps, &account, page.unwrap_or(0), page_size)
        }
        QueryWithPermit::TokenIdPrivateInfo { token_id } => {
            query_token_id_private_info(deps, &account, token_id)
        }
    }
}

fn viewing_keys_queries(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
    let (addresses, key) = msg.get_validation_params()?;

    for address in addresses {
        let result = ViewingKey::check(deps.storage, address.as_str(), key.as_str());
        if result.is_ok() {
            return match msg {
                QueryMsg::IdTotalBalance { id } => query_id_balance(deps, id),
                QueryMsg::Balance {
                    owner,
                    viewer,
                    token_id,
                    ..
                } => query_balance(deps, &owner, &viewer, token_id),
                QueryMsg::AllBalances {
                    tx_history_page,
                    tx_history_page_size,
                    ..
                } => query_all_balances(deps, address, tx_history_page, tx_history_page_size),
                QueryMsg::TransactionHistory {
                    page, page_size, ..
                } => query_transactions(deps, address, page.unwrap_or(0), page_size),
                QueryMsg::Permission {
                    owner,
                    allowed_address,
                    token_id,
                    ..
                } => query_permission(deps, token_id, owner, allowed_address),
                QueryMsg::AllPermissions {
                    page, page_size, ..
                } => query_all_permissions(deps, address, page.unwrap_or(0), page_size),
                QueryMsg::TokenIdPrivateInfo {
                    address, token_id, ..
                } => query_token_id_private_info(deps, &address, token_id),
                QueryMsg::ContractInfo {}
                | QueryMsg::TokenIdPublicInfo { .. }
                | QueryMsg::RegisteredCodeHash { .. }
                | QueryMsg::WithPermit { .. } => {
                    unreachable!("This query type does not require viewing key authentication")
                }
            };
        }
    }

    to_binary(&QueryAnswer::ViewingKeyError {
        msg: "Wrong viewing key for this address or viewing key not set".to_string(),
    })
}
