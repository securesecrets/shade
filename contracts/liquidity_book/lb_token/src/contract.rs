use std::collections::BTreeSet;

// use base64::{engine::general_purpose, Engine as _};
use cosmwasm_std::{
    entry_point,
    // debug_print,
    to_binary,
    Addr,
    Binary,
    BlockInfo,
    CosmosMsg,
    Deps,
    DepsMut,
    Env,
    MessageInfo,
    Response,
    StdError,
    StdResult,
    Storage,
    Timestamp,
    Uint256,
};
use secret_toolkit::{
    crypto::sha_256,
    permit::RevokedPermits,
    utils::space_pad,
    viewing_key::{ViewingKey, ViewingKeyStore},
};

use crate::state::{
    balances_r, blockinfo_r, contr_conf_r, get_receiver_hash,
    permissions::{list_owner_permission_keys, may_load_any_permission},
    tkn_info_r, tkn_tot_supply_r,
    txhistory::{get_txs, may_get_current_owner},
    PREFIX_REVOKED_PERMITS,
};
use crate::{
    receiver::Snip1155ReceiveMsg,
    state::{
        balances_w, blockinfo_w, contr_conf_w,
        permissions::{new_permission, update_permission},
        set_receiver_hash, tkn_info_w, tkn_tot_supply_w,
        txhistory::{append_new_owner, store_burn, store_mint, store_transfer},
        RESPONSE_BLOCK_SIZE,
    },
};

use secret_toolkit::permit::{validate, Permit, TokenPermissions};
use shade_protocol::lb_libraries::lb_token::{
    expiration::Expiration,
    metadata::Metadata,
    permissions::{Permission, PermissionKey},
    state_structs::{
        ContractConfig, CurateTokenId, OwnerBalance, StoredTokenInfo, TknConfig, TokenAmount,
        TokenInfoMsg,
    },
};
use shade_protocol::liquidity_book::lb_token::{
    ExecuteAnswer, ExecuteMsg, InstantiateMsg, ResponseStatus::Success, SendAction, TransferAction,
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
        } => try_send(
            deps,
            env,
            info,
            SendAction {
                token_id,
                from,
                recipient,
                recipient_code_hash,
                amount,
                msg,
                memo,
            },
        ),
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
        ExecuteMsg::AddCurators {
            add_curators,
            padding: _,
        } => try_add_curators(deps, env, info, add_curators),
        ExecuteMsg::RemoveCurators {
            remove_curators,
            padding: _,
        } => try_remove_curators(deps, env, info, remove_curators),
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

// fn try_curate_token_ids(
//     mut deps: DepsMut,
//     env: Env,
//     info: MessageInfo,
//     initial_tokens: Vec<CurateTokenId>,
//     memo: Option<String>,
// ) -> StdResult<Response> {
//     let mut config = contr_conf_r(deps.storage).load()?;
//     // check if sender is a curator
//     verify_curator(&config, &info)?;

//     // curate new token_ids
//     for initial_token in initial_tokens {
//         exec_curate_token_id(
//             &mut deps,
//             &env,
//             &info,
//             &mut config,
//             initial_token,
//             memo.clone(),
//         )?;
//     }

//     contr_conf_w(deps.storage).save(&config)?;

//     Ok(
//         Response::new().set_data(to_binary(&ExecuteAnswer::CurateTokenIds {
//             status: Success,
//         })?),
//     )
// }

fn try_mint_tokens(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mint_tokens: Vec<TokenAmount>,
    memo: Option<String>,
) -> StdResult<Response> {
    let mut config = contr_conf_r(deps.storage).load()?;
    verify_curator(&config, &info)?;

    // mint tokens
    for mint_token in mint_tokens {
        let token_info_op = tkn_info_r(deps.storage).may_load(mint_token.token_id.as_bytes())?;

        // check if token_id exists
        if token_info_op.is_none() {
            let curate_token = CurateTokenId {
                token_info: TokenInfoMsg {
                    token_id: mint_token.token_id.clone(),
                    name: format!("LP-{}", &config.lb_pair_info.symbol),
                    symbol: format!("LP-{}", &config.lb_pair_info.symbol),
                    token_config: TknConfig::Fungible {
                        minters: Vec::new(), // No need for minter curator will be the minter
                        decimals: config.lb_pair_info.decimals,
                        public_total_supply: true,
                        enable_mint: true,
                        enable_burn: true,
                        minter_may_update_metadata: false,
                    },
                    public_metadata: None,
                    private_metadata: None,
                },
                balances: mint_token.balances,
            };

            exec_curate_token_id(
                &mut deps,
                &env,
                &info,
                &mut config,
                curate_token,
                memo.clone(),
            )?;
            continue;
        }

        // check if enable_mint == true
        // if !token_info_op
        //     .clone()
        //     .unwrap()
        //     .token_config
        //     .flatten()
        //     .enable_mint
        // {
        //     return Err(StdError::generic_err(
        //         "minting is not enabled for this token_id",
        //     ));
        // }

        // check if sender is a minter
        // verify_minter(token_info_op.as_ref().unwrap(), &info)?;
        // add balances

        for add_balance in mint_token.balances {
            exec_change_balance(
                deps.storage,
                &mint_token.token_id,
                None,
                Some(&add_balance.address),
                &add_balance.amount,
                &token_info_op.clone().unwrap(),
            )?;

            // store mint_token
            store_mint(
                deps.storage,
                &mut config,
                &env.block,
                &mint_token.token_id,
                deps.api.addr_canonicalize(info.sender.as_str())?,
                deps.api.addr_canonicalize(add_balance.address.as_str())?,
                add_balance.amount,
                memo.clone(),
            )?;
        }
    }

    contr_conf_w(deps.storage).save(&config)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::MintTokens { status: Success })?))
}

// in the base specifications, this function can be performed by token owner only
fn try_burn_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    burn_tokens: Vec<TokenAmount>,
    memo: Option<String>,
) -> StdResult<Response> {
    let mut config = contr_conf_r(deps.storage).load()?;
    verify_curator(&config, &info)?;
    // burn tokens
    for burn_token in burn_tokens {
        let token_info_op = tkn_info_r(deps.storage).may_load(burn_token.token_id.as_bytes())?;

        if token_info_op.is_none() {
            return Err(StdError::generic_err(
                  "token_id does not exist. Cannot burn non-existent `token_ids`. Use `curate_token_ids` to create tokens on new `token_ids`"
              ));
        }

        let token_info = token_info_op.clone().unwrap();

        if !token_info.token_config.flatten().enable_burn {
            return Err(StdError::generic_err(
                "burning is not enabled for this token_id",
            ));
        }

        // remove balances
        for rem_balance in burn_token.balances {
            // in base specification, burner MUST be the owner
            // if rem_balance.address != info.sender {
            //     return Err(StdError::generic_err(format!(
            //         "you do not have permission to burn {} tokens from address {}",
            //         rem_balance.amount, rem_balance.address
            //     )));
            // }

            exec_change_balance(
                deps.storage,
                &burn_token.token_id,
                Some(&rem_balance.address),
                None,
                &rem_balance.amount,
                &token_info,
            )?;

            // store burn_token
            store_burn(
                deps.storage,
                &mut config,
                &env.block,
                &burn_token.token_id,
                // in base specification, burner MUST be the owner
                None,
                deps.api.addr_canonicalize(rem_balance.address.as_str())?,
                rem_balance.amount,
                memo.clone(),
            )?;
        }
    }

    contr_conf_w(deps.storage).save(&config)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::BurnTokens { status: Success })?))
}

fn try_change_metadata(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
    public_metadata: Option<Metadata>,
    private_metadata: Option<Metadata>,
) -> StdResult<Response> {
    let tkn_info_op = tkn_info_r(deps.storage).may_load(token_id.as_bytes())?;
    let tkn_conf = match tkn_info_op.clone() {
        None => {
            return Err(StdError::generic_err(format!(
                "token_id {} does not exist",
                token_id
            )))
        }
        Some(i) => i.token_config.flatten(),
    };

    // define variables for control flow
    let owner = may_get_current_owner(deps.storage, &token_id)?;
    let is_owner = match owner {
        Some(owner_addr) => owner_addr == info.sender,
        None => false,
    };

    let is_minter = verify_minter(tkn_info_op.as_ref().unwrap(), &info).is_ok();

    // can sender change metadata? based on i) sender is minter or owner, ii) token_id config allows it or not
    let allow_update = is_owner && tkn_conf.owner_may_update_metadata
        || is_minter && tkn_conf.minter_may_update_metadata;

    // control flow based on `allow_update`
    match allow_update {
        false => {
            return Err(StdError::generic_err(format!(
                "unable to change the metadata for token_id {}",
                token_id
            )))
        }
        true => {
            let mut tkn_info = tkn_info_op.unwrap();
            if public_metadata.is_some() {
                tkn_info.public_metadata = public_metadata
            };
            if private_metadata.is_some() {
                tkn_info.private_metadata = private_metadata
            };
            tkn_info_w(deps.storage).save(token_id.as_bytes(), &tkn_info)?;
        }
    }

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::ChangeMetadata {
            status: Success,
        })?),
    )
}

#[allow(clippy::too_many_arguments)]
fn try_transfer(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_id: String,
    from: Addr,
    recipient: Addr,
    amount: Uint256,
    memo: Option<String>,
) -> StdResult<Response> {
    impl_transfer(
        &mut deps, &env, &info, &token_id, &from, &recipient, amount, memo,
    )?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Transfer { status: Success })?))
}

fn try_batch_transfer(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    actions: Vec<TransferAction>,
) -> StdResult<Response> {
    for action in actions {
        let from = deps.api.addr_validate(action.from.as_str())?;
        let recipient = deps.api.addr_validate(action.recipient.as_str())?;
        impl_transfer(
            &mut deps,
            &env,
            &info,
            &action.token_id,
            &from,
            &recipient,
            action.amount,
            action.memo,
        )?;
    }

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::BatchTransfer {
            status: Success,
        })?),
    )
}

fn try_send(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    action: SendAction,
) -> StdResult<Response> {
    // set up cosmos messages
    let mut messages = vec![];

    impl_send(&mut deps, &env, &info, &mut messages, action)?;

    let data = to_binary(&ExecuteAnswer::Send { status: Success })?;
    let res = Response::new().add_messages(messages).set_data(data);
    Ok(res)
}

fn try_batch_send(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    actions: Vec<SendAction>,
) -> StdResult<Response> {
    // declare vector for cosmos messages
    let mut messages = vec![];

    for action in actions {
        impl_send(&mut deps, &env, &info, &mut messages, action)?;
    }

    let data = to_binary(&ExecuteAnswer::BatchSend { status: Success })?;
    let res = Response::new().add_messages(messages).set_data(data);
    Ok(res)
}

/// does not check if `token_id` exists so attacker cannot easily figure out if
/// a `token_id` has been created
#[allow(clippy::too_many_arguments)]
fn try_give_permission(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    allowed_address: Addr,
    token_id: String,
    view_balance: Option<bool>,
    view_balance_expiry: Option<Expiration>,
    view_private_metadata: Option<bool>,
    view_private_metadata_expiry: Option<Expiration>,
    transfer: Option<Uint256>,
    transfer_expiry: Option<Expiration>,
) -> StdResult<Response> {
    // may_load current permission
    let permission_op =
        may_load_any_permission(deps.storage, &info.sender, &token_id, &allowed_address)?;

    let action = |old_perm: Permission,
                  view_balance: Option<bool>,
                  view_balance_expiry: Option<Expiration>,
                  view_private_metadata: Option<bool>,
                  view_private_metadata_expiry: Option<Expiration>,
                  transfer: Option<Uint256>,
                  transfer_expiry: Option<Expiration>|
     -> Permission {
        Permission {
            view_balance_perm: match view_balance {
                Some(i) => i,
                None => old_perm.view_balance_perm,
            },
            view_balance_exp: match view_balance_expiry {
                Some(i) => i,
                None => old_perm.view_balance_exp,
            },
            view_pr_metadata_perm: match view_private_metadata {
                Some(i) => i,
                None => old_perm.view_pr_metadata_perm,
            },
            view_pr_metadata_exp: match view_private_metadata_expiry {
                Some(i) => i,
                None => old_perm.view_pr_metadata_exp,
            },
            trfer_allowance_perm: match transfer {
                Some(i) => i,
                None => old_perm.trfer_allowance_perm,
            },
            trfer_allowance_exp: match transfer_expiry {
                Some(i) => i,
                None => old_perm.trfer_allowance_exp,
            },
        }
    };

    // create new permission if not created yet, otherwise update existing permission
    match permission_op {
        Some(old_perm) => {
            let updated_permission = action(
                old_perm,
                view_balance,
                view_balance_expiry,
                view_private_metadata,
                view_private_metadata_expiry,
                transfer,
                transfer_expiry,
            );
            update_permission(
                deps.storage,
                &info.sender,
                &token_id,
                &allowed_address,
                &updated_permission,
            )?;
        }
        None => {
            let default_permission = Permission::default();
            let updated_permission = action(
                default_permission,
                view_balance,
                view_balance_expiry,
                view_private_metadata,
                view_private_metadata_expiry,
                transfer,
                transfer_expiry,
            );
            new_permission(
                deps.storage,
                &info.sender,
                &token_id,
                &allowed_address,
                &updated_permission,
            )?;
        }
    };

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::GivePermission {
            status: Success,
        })?),
    )
}

/// changes an existing permission entry to default (ie: revoke all permissions granted). Does not remove
/// entry in storage, because it is unecessarily in most use cases, but will require also removing
/// owner-specific PermissionKeys, which introduces complexity and increases gas cost.
/// If permission does not exist, message will return an error.
fn try_revoke_permission(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
    owner: Addr,
    allowed_addr: Addr,
) -> StdResult<Response> {
    // either owner or allowed_address can remove permission
    if info.sender != owner && info.sender != allowed_addr {
        return Err(StdError::generic_err(
            "only the owner or address with permission can remove permission",
        ));
    }

    update_permission(
        deps.storage,
        &owner,
        &token_id,
        &allowed_addr,
        &Permission::default(),
    )?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::RevokePermission {
            status: Success,
        })?),
    )
}

fn try_create_viewing_key(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    entropy: String,
) -> StdResult<Response> {
    let key = ViewingKey::create(
        deps.storage,
        &info,
        &env,
        info.sender.as_str(),
        entropy.as_ref(),
    );

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::CreateViewingKey { key })?))
}

fn try_set_viewing_key(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    key: String,
) -> StdResult<Response> {
    ViewingKey::set(deps.storage, info.sender.as_str(), key.as_str());
    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::SetViewingKey {
            status: Success,
        })?),
    )
}

fn try_revoke_permit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    permit_name: String,
) -> StdResult<Response> {
    RevokedPermits::revoke_permit(
        deps.storage,
        PREFIX_REVOKED_PERMITS,
        info.sender.as_ref(),
        &permit_name,
    );

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::RevokePermit { status: Success })?))
}

fn try_add_curators(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    add_curators: Vec<Addr>,
) -> StdResult<Response> {
    let mut config = contr_conf_r(deps.storage).load()?;

    // verify admin
    verify_admin(&config, &info)?;

    // add curators
    for curator in add_curators {
        config.curators.push(curator);
    }
    contr_conf_w(deps.storage).save(&config)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::AddCurators { status: Success })?))
}

fn try_remove_curators(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    remove_curators: Vec<Addr>,
) -> StdResult<Response> {
    let mut config = contr_conf_r(deps.storage).load()?;

    // verify admin
    verify_admin(&config, &info)?;

    // remove curators
    for curator in remove_curators {
        config.curators.retain(|x| x != &curator);
    }
    contr_conf_w(deps.storage).save(&config)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::RemoveCurators {
            status: Success,
        })?),
    )
}

// fn try_add_minters(
//     deps: DepsMut,
//     _env: Env,
//     info: MessageInfo,
//     token_id: String,
//     add_minters: Vec<Addr>,
// ) -> StdResult<Response> {
//     let contract_config = contr_conf_r(deps.storage).load()?;
//     let token_info_op = tkn_info_r(deps.storage).may_load(token_id.as_bytes())?;
//     if token_info_op.is_none() {
//         return Err(StdError::generic_err(format!(
//             "token_id {} does not exist",
//             token_id
//         )));
//     };
//     let mut token_info = token_info_op.unwrap();

//     // check if either admin
//     let admin_result = verify_admin(&contract_config, &info);
//     // let curator_result = verify_curator_of_token_id(&token_info, &env); Not part of base specifications.

//     let verified = admin_result.is_ok(); // || curator_result.is_ok();
//     if !verified {
//         return Err(StdError::generic_err(
//             "You need to be the admin to add or remove minters",
//         ));
//     }

//     // add minters
//     let mut flattened_token_config = token_info.token_config.flatten();
//     for minter in add_minters {
//         flattened_token_config.minters.push(minter)
//     }

//     // save token info with new minters
//     token_info.token_config = flattened_token_config.to_enum();
//     tkn_info_w(deps.storage).save(token_id.as_bytes(), &token_info)?;

//     Ok(Response::new().set_data(to_binary(&ExecuteAnswer::AddMinters { status: Success })?))
// }

// fn try_remove_minters(
//     deps: DepsMut,
//     _env: Env,
//     info: MessageInfo,
//     token_id: String,
//     remove_minters: Vec<Addr>,
// ) -> StdResult<Response> {
//     let contract_config = contr_conf_r(deps.storage).load()?;
//     let token_info_op = tkn_info_r(deps.storage).may_load(token_id.as_bytes())?;
//     if token_info_op.is_none() {
//         return Err(StdError::generic_err(format!(
//             "token_id {} does not exist",
//             token_id
//         )));
//     };
//     let mut token_info = token_info_op.unwrap();

//     // check if either admin or curator
//     let admin_result = verify_admin(&contract_config, &info);
//     // let curator_result = verify_curator_of_token_id(&token_info, &env); Not part of base specifications.

//     let verified = admin_result.is_ok(); // || curator_result.is_ok();
//     if !verified {
//         return Err(StdError::generic_err(
//             "You need to be the admin to add or remove minters",
//         ));
//     }

//     // remove minters
//     let mut flattened_token_config = token_info.token_config.flatten();
//     for minter in remove_minters {
//         flattened_token_config.minters.retain(|x| x != &minter);
//     }

//     // save token info with new minters
//     token_info.token_config = flattened_token_config.to_enum();
//     tkn_info_w(deps.storage).save(token_id.as_bytes(), &token_info)?;

//     Ok(
//         Response::new().set_data(to_binary(&ExecuteAnswer::RemoveMinters {
//             status: Success,
//         })?),
//     )
// }

fn try_change_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    new_admin: Addr,
) -> StdResult<Response> {
    let mut config = contr_conf_r(deps.storage).load()?;

    // verify admin
    verify_admin(&config, &info)?;

    // change admin
    config.admin = Some(new_admin);
    contr_conf_w(deps.storage).save(&config)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::ChangeAdmin { status: Success })?))
}

fn try_remove_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    current_admin: Addr,
    contract_address: Addr,
) -> StdResult<Response> {
    let mut config = contr_conf_r(deps.storage).load()?;

    // verify admin
    verify_admin(&config, &info)?;

    // checks on redundancy inputs, designed to reduce chances of accidentally
    // calling this function
    if current_admin != config.admin.unwrap() || contract_address != config.contract_address {
        return Err(StdError::generic_err(
            "your inputs are incorrect to perform this function",
        ));
    }

    // remove admin
    config.admin = None;
    contr_conf_w(deps.storage).save(&config)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::RemoveAdmin { status: Success })?))
}

fn try_register_receive(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    code_hash: String,
) -> StdResult<Response> {
    set_receiver_hash(deps.storage, &info.sender, code_hash);

    let data = to_binary(&ExecuteAnswer::RegisterReceive { status: Success })?;
    Ok(Response::new()
        .add_attribute("register_status", "success")
        .set_data(data))
}

/////////////////////////////////////////////////////////////////////////////////
// Private functions
/////////////////////////////////////////////////////////////////////////////////

fn pad_response(response: StdResult<Response>) -> StdResult<Response> {
    response.map(|mut response| {
        response.data = response.data.map(|mut data| {
            space_pad(&mut data.0, RESPONSE_BLOCK_SIZE);
            data
        });
        response
    })
}

fn is_valid_name(name: &str) -> bool {
    let len = name.len();
    (3..=30).contains(&len)
}

fn is_valid_symbol(symbol: &str) -> bool {
    let len = symbol.len();
    (3..=30).contains(&len)
    // let len = symbol.len();
    // let len_is_valid = (3..=50).contains(&len);

    // // len_is_valid && symbol.bytes().all(|byte| (b'A'..=b'Z').contains(&byte))
    // len_is_valid && symbol.bytes().all(|byte| byte.is_ascii_uppercase())
}

fn verify_admin(contract_config: &ContractConfig, info: &MessageInfo) -> StdResult<()> {
    let admin_op = &contract_config.admin;
    match admin_op {
        Some(admin) => {
            if admin != &info.sender {
                return Err(StdError::generic_err("This is an admin function"));
            }
        }
        None => return Err(StdError::generic_err("This contract has no admin")),
    }

    Ok(())
}

/// verifies if sender is a curator
fn verify_curator(contract_config: &ContractConfig, info: &MessageInfo) -> StdResult<()> {
    let curators = &contract_config.curators;
    if !curators.contains(&info.sender) {
        return Err(StdError::generic_err(
            "Only curators are allowed to curate token_ids",
        ));
    }
    Ok(())
}

// /// verifies if sender is the address that curated the token_id.
// /// Not part of base specifications, but function left here for potential use.
// /// If this additional feature is implemented, it is important to ensure that the instantiator
// /// still has the ability to set initial balances without later being able to change minters.
// fn verify_curator_of_token_id(
//     token_info: &StoredTokenInfo,
//     env: &Env
// ) -> StdResult<()> {
//     let curator = &token_info.curator;
//     if curator != &env.message.sender {
//         return Err(StdError::generic_err(
//             "You are not the curator of this token_id",
//         ));
//     }
//     Ok(())
// }

/// verifies if sender is a minter of the specific token_id
fn verify_minter(token_info: &StoredTokenInfo, info: &MessageInfo) -> StdResult<()> {
    let minters = &token_info.token_config.flatten().minters;
    if !minters.contains(&info.sender) {
        return Err(StdError::generic_err(format!(
            "Only minters are allowed to mint additional tokens for token_id {}",
            token_info.token_id
        )));
    }
    Ok(())
}

/// checks if `token_id` is available (ie: not yet created), then creates new `token_id` and initial balances
fn exec_curate_token_id(
    deps: &mut DepsMut,
    env: &Env,
    info: &MessageInfo,
    config: &mut ContractConfig,
    initial_token: CurateTokenId,
    memo: Option<String>,
) -> StdResult<()> {
    // check: token_id has not been created yet
    if tkn_info_r(deps.storage)
        .may_load(initial_token.token_info.token_id.as_bytes())?
        .is_some()
    {
        return Err(StdError::generic_err(
            "token_id already exists. Try a different id String",
        ));
    }

    // check: token_id is an NFT => cannot create more than one
    if initial_token.token_info.token_config.flatten().is_nft {
        if initial_token.balances.len() > 1 {
            return Err(StdError::generic_err(format!(
                  "token_id {} is an NFT; there can only be one NFT. Balances should only have one address",
                  initial_token.token_info.token_id
              )));
        } else if initial_token.balances[0].amount != Uint256::from(1_u64) {
            return Err(StdError::generic_err(format!(
                "token_id {} is an NFT; there can only be one NFT. Balances.amount must == 1",
                initial_token.token_info.token_id
            )));
        }
    }

    // Check name, symbol, decimals
    if !is_valid_name(&initial_token.token_info.name) {
        return Err(StdError::generic_err(
            "Name is not in the expected format (3-30 UTF-8 bytes)",
        ));
    }

    if !is_valid_symbol(&initial_token.token_info.symbol) {
        return Err(StdError::generic_err(
            "Ticker symbol is not in expected format [A-Z]{3,6}",
        ));
    }

    if initial_token.token_info.token_config.flatten().decimals > 18 {
        return Err(StdError::generic_err("Decimals must not exceed 18"));
    }

    // create and save new token info
    tkn_info_w(deps.storage).save(
        initial_token.token_info.token_id.as_bytes(),
        &initial_token.token_info.to_store(&info.sender),
    )?;

    // set initial balances and store mint history
    for balance in initial_token.balances {
        // save new balances
        balances_w(deps.storage, &initial_token.token_info.token_id)
            .save(to_binary(&balance.address)?.as_slice(), &balance.amount)?;
        // if is_nft == true, store owner of NFT
        if initial_token.token_info.token_config.flatten().is_nft {
            append_new_owner(
                deps.storage,
                &initial_token.token_info.token_id,
                &balance.address,
            )?;
        }
        // initiate total token supply
        tkn_tot_supply_w(deps.storage).save(
            initial_token.token_info.token_id.as_bytes(),
            &balance.amount,
        )?;

        // store mint_token_id
        store_mint(
            deps.storage,
            config,
            &env.block,
            &initial_token.token_info.token_id,
            deps.api.addr_canonicalize(info.sender.as_str())?,
            deps.api.addr_canonicalize(balance.address.as_str())?,
            balance.amount,
            memo.clone(),
        )?;
    }

    // push token_id to token_id_list
    config.token_id_list.push(initial_token.token_info.token_id);

    Ok(())
}

/// Implements a single `Send` function. Transfers Uint256 amount of a single `token_id`,
/// saves transfer history, may register-receive, and creates callback message.
fn impl_send(
    deps: &mut DepsMut,
    env: &Env,
    info: &MessageInfo,
    messages: &mut Vec<CosmosMsg>,
    action: SendAction,
) -> StdResult<()> {
    // action variables from SendAction
    let token_id = action.token_id;
    let from = action.from;
    let amount = action.amount;
    let recipient = action.recipient;
    let recipient_code_hash = action.recipient_code_hash;
    let msg = action.msg;
    let memo = action.memo;

    // implements transfer of tokens
    impl_transfer(
        deps,
        env,
        info,
        &token_id,
        &from,
        &recipient,
        amount,
        memo.clone(),
    )?;

    // create cosmos message
    try_add_receiver_api_callback(
        deps.storage,
        messages,
        recipient,
        recipient_code_hash,
        msg,
        info.sender.clone(),
        token_id,
        from.to_owned(),
        amount,
        memo,
    )?;

    Ok(())
}

/// Implements a single `Transfer` function. Transfers a Uint256 amount of a
/// single `token_id` and saves the transfer history. Used by `Transfer` and
/// `Send` (via `impl_send`) messages
#[allow(clippy::too_many_arguments)]
fn impl_transfer(
    deps: &mut DepsMut,
    env: &Env,
    info: &MessageInfo,
    token_id: &str,
    from: &Addr,
    recipient: &Addr,
    amount: Uint256,
    memo: Option<String>,
) -> StdResult<()> {
    // check if `from` == message sender || has enough allowance to send tokens
    // perform allowance check, and may reduce allowance
    let mut throw_err = false;
    if from != &info.sender {
        // may_load_active_permission() or may_load_any_permission() both work. The former performs redundancy checks, which are
        // more relevant for authenticated queries (because transfer simply won't work if there is no balance)
        let permission_op = may_load_any_permission(deps.storage, from, token_id, &info.sender)?;

        match permission_op {
            // no permission given
            None => throw_err = true,
            // allowance has expired
            Some(perm) if perm.trfer_allowance_exp.is_expired(&env.block) => {
                return Err(StdError::generic_err(format!(
                    "Allowance has expired: {}",
                    perm.trfer_allowance_exp
                )))
            }
            // not enough allowance to transfer amount
            Some(perm) if perm.trfer_allowance_perm < amount => {
                return Err(StdError::generic_err(format!(
                    "Insufficient transfer allowance: {}",
                    perm.trfer_allowance_perm
                )))
            }
            // success, so need to reduce allowance
            Some(mut perm) if perm.trfer_allowance_perm >= amount => {
                let new_allowance = Uint256::from(
                    perm.trfer_allowance_perm
                        .checked_sub(amount)
                        .expect("something strange happened"),
                );
                perm.trfer_allowance_perm = new_allowance;
                update_permission(deps.storage, from, token_id, &info.sender, &perm)?;
            }
            Some(_) => unreachable!("impl_transfer permission check: this should not be reachable"),
        }
    }

    // check that token_id exists
    let token_info_op = tkn_info_r(deps.storage).may_load(token_id.as_bytes())?;
    if token_info_op.is_none() {
        throw_err = true
    }

    // combined error message for no token_id or no permission given in one place to make it harder to identify if token_id already exists
    match throw_err {
        true => {
            return Err(StdError::generic_err(
                "These tokens do not exist or you have no permission to transfer",
            ))
        }
        false => (),
    }

    // transfer tokens
    exec_change_balance(
        deps.storage,
        token_id,
        Some(from),
        Some(recipient),
        &amount,
        &token_info_op.unwrap(),
    )?;

    // store transaction
    let mut config = contr_conf_r(deps.storage).load()?;
    store_transfer(
        deps.storage,
        &mut config,
        &env.block,
        token_id,
        deps.api.addr_canonicalize(from.as_str())?,
        None,
        deps.api.addr_canonicalize(recipient.as_str())?,
        amount,
        memo,
    )?;
    contr_conf_w(deps.storage).save(&config)?;

    Ok(())
}

/// change token balance of an existing `token_id`.
///
/// Should check that `token_id` already exists before calling this function, which is not done
/// explicitly in this function. Although token_info is an argument in this function, so it is
/// likely that the calling function would have had to check.
/// * If `remove_from` == None: minted new tokens.
/// * If `add_to` == None: burn tokens.
/// * If is_nft == true, then `remove_from` MUST be Some(_).
/// * If is_nft == true, stores new owner of NFT
fn exec_change_balance(
    storage: &mut dyn Storage,
    token_id: &str,
    remove_from: Option<&Addr>,
    add_to: Option<&Addr>,
    amount: &Uint256,
    token_info: &StoredTokenInfo,
) -> StdResult<()> {
    // check whether token_id is an NFT => cannot mint. This should not be reachable in standard implementation,
    // as the calling function would have checked that enable_mint == false, which needs to be true for NFTs.
    // This is a redundancy check to make sure
    if token_info.token_config.flatten().is_nft && remove_from.is_none() {
        return Err(StdError::generic_err(
            "NFTs can only be minted once using `mint_token_ids`",
        ));
    }

    // check whether token_id is an NFT => assert!(amount == 1).
    if token_info.token_config.flatten().is_nft && amount != Uint256::from(1_u64) {
        return Err(StdError::generic_err("NFT amount must == 1"));
    }

    // remove balance
    if let Some(from) = remove_from {
        let from_existing_bal = balances_r(storage, token_id).load(to_binary(&from)?.as_slice())?;
        let from_new_amount_op = from_existing_bal.checked_sub(*amount);
        if from_new_amount_op.is_err() {
            return Err(StdError::generic_err("insufficient funds"));
        }
        balances_w(storage, token_id).save(
            to_binary(&from)?.as_slice(),
            &Uint256::from(from_new_amount_op.unwrap()),
        )?;

        // NOTE: if nft, the ownership history remains in storage. Any existing viewing permissions of last owner
        // will remain too
    }

    // add balance
    if let Some(to) = add_to {
        let to_existing_bal_op =
            balances_r(storage, token_id).may_load(to_binary(&to)?.as_slice())?;
        let to_existing_bal = match to_existing_bal_op {
            Some(i) => i,
            // if `to` address has no balance yet, initiate zero balance
            None => Uint256::from(0_u64),
        };
        let to_new_amount_op = to_existing_bal.checked_add(*amount);
        if to_new_amount_op.is_err() {
            return Err(StdError::generic_err(
                "recipient will become too rich. Total tokens exceeds 2^128",
            ));
        }

        // save new balances
        balances_w(storage, token_id).save(
            to_binary(&to)?.as_slice(),
            &Uint256::from(to_new_amount_op.unwrap()),
        )?;

        // if is_nft == true, store new owner of NFT
        if token_info.token_config.flatten().is_nft {
            append_new_owner(storage, &token_info.token_id, to)?;
        }
    }

    // may change total token supply
    match (remove_from, add_to) {
        (None, None) => (),
        (Some(_), Some(_)) => (),
        (None, Some(_)) => {
            let old_amount = tkn_tot_supply_r(storage).load(token_info.token_id.as_bytes())?;
            let new_amount_op = old_amount.checked_add(*amount);
            let new_amount = match new_amount_op {
                Ok(i) => Uint256::from(i),
                Err(_e) => {
                    return Err(StdError::generic_err(
                        "total supply exceeds max allowed of 2^128",
                    ))
                }
            };
            tkn_tot_supply_w(storage).save(token_info.token_id.as_bytes(), &new_amount)?;
        }
        (Some(_), None) => {
            let old_amount = tkn_tot_supply_r(storage).load(token_info.token_id.as_bytes())?;
            let new_amount_op = old_amount.checked_sub(*amount);
            let new_amount = match new_amount_op {
                Ok(i) => Uint256::from(i),
                Err(_e) => return Err(StdError::generic_err("total supply drops below zero")),
            };
            tkn_tot_supply_w(storage).save(token_info.token_id.as_bytes(), &new_amount)?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn try_add_receiver_api_callback(
    storage: &dyn Storage,
    messages: &mut Vec<CosmosMsg>,
    recipient: Addr,
    recipient_code_hash: Option<String>,
    msg: Option<Binary>,
    sender: Addr,
    token_id: String,
    from: Addr,
    amount: Uint256,
    memo: Option<String>,
) -> StdResult<()> {
    if let Some(receiver_hash) = recipient_code_hash {
        let receiver_msg = Snip1155ReceiveMsg::new(sender, token_id, from, amount, memo, msg);
        let callback_msg = receiver_msg.into_cosmos_msg(receiver_hash, recipient)?;

        messages.push(callback_msg);
        return Ok(());
    }

    let receiver_hash = get_receiver_hash(storage, &recipient);
    if let Some(receiver_hash) = receiver_hash {
        let receiver_hash = receiver_hash?;
        let receiver_msg = Snip1155ReceiveMsg::new(sender, token_id, from, amount, memo, msg);
        let callback_msg = receiver_msg.into_cosmos_msg(receiver_hash, recipient)?;

        messages.push(callback_msg);
    }

    Ok(())
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
        QueryMsg::TokenContractInfo {} => query_contract_info(deps),
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
                    owner.as_str(), allowed_address.as_str(), account.as_str()
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
                QueryMsg::TokenContractInfo {}
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

fn query_contract_info(deps: Deps) -> StdResult<Binary> {
    let contr_conf = contr_conf_r(deps.storage).load()?;
    let response = QueryAnswer::TokenContractInfo {
        admin: contr_conf.admin,
        curators: contr_conf.curators,
        all_token_ids: contr_conf.token_id_list,
    };
    to_binary(&response)
}

fn query_id_balance(deps: Deps, token_id: String) -> StdResult<Binary> {
    let id_balance_raw = tkn_tot_supply_r(deps.storage).load(token_id.as_bytes());

    let mut id_balance = Uint256::zero();

    if id_balance_raw.is_ok() {
        id_balance = id_balance_raw?;
    }

    let response = QueryAnswer::IdTotalBalance { amount: id_balance };
    to_binary(&response)
}

fn query_token_id_public_info(deps: Deps, token_id: String) -> StdResult<Binary> {
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

fn query_token_id_private_info(deps: Deps, viewer: &Addr, token_id: String) -> StdResult<Binary> {
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
                ))
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

fn query_registered_code_hash(deps: Deps, contract: Addr) -> StdResult<Binary> {
    let may_hash_res = get_receiver_hash(deps.storage, &contract);
    let response: QueryAnswer = match may_hash_res {
        Some(hash_res) => QueryAnswer::RegisteredCodeHash {
            code_hash: Some(hash_res?),
        },
        None => QueryAnswer::RegisteredCodeHash { code_hash: None },
    };

    to_binary(&response)
}

fn query_balance(deps: Deps, owner: &Addr, viewer: &Addr, token_id: String) -> StdResult<Binary> {
    if owner != viewer {
        let permission_op = may_load_any_permission(deps.storage, owner, &token_id, viewer)?;
        match permission_op {
            None => {
                return Err(StdError::generic_err(
                    "you do have have permission to view balance",
                ))
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

fn query_all_balances(
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
            balances.push(OwnerBalance {
                token_id,
                amount: i,
            })
        }
    }

    let response = QueryAnswer::AllBalances(balances);
    to_binary(&response)
}

fn query_transactions(deps: Deps, account: &Addr, page: u32, page_size: u32) -> StdResult<Binary> {
    let address = deps.api.addr_canonicalize(account.as_str())?;
    let (txs, total) = get_txs(deps.api, deps.storage, &address, page, page_size)?;

    let response = QueryAnswer::TransactionHistory { txs, total };
    to_binary(&response)
}

fn query_permission(
    deps: Deps,
    token_id: String,
    owner: Addr,
    allowed_addr: Addr,
) -> StdResult<Binary> {
    let permission = may_load_any_permission(deps.storage, &owner, &token_id, &allowed_addr)?;

    let response = QueryAnswer::Permission(permission);
    to_binary(&response)
}

fn query_all_permissions(
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
