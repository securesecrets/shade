// use base64::{engine::general_purpose, Engine as _};
use cosmwasm_std::{
    entry_point,
    // debug_print,
    to_binary,
    Addr,
    Binary,
    CosmosMsg,
    DepsMut,
    Env,
    MessageInfo,
    Response,
    StdError,
    StdResult,
    Storage,
    Uint256,
};

use crate::state::{
    balances_r,
    balances_w,
    blockinfo_w,
    contr_conf_r,
    contr_conf_w,
    get_receiver_hash,
    permissions::{may_load_any_permission, new_permission, update_permission},
    set_receiver_hash,
    tkn_info_r,
    tkn_info_w,
    tkn_tot_supply_r,
    tkn_tot_supply_w,
    txhistory::{append_new_owner, may_get_current_owner, store_burn, store_mint, store_transfer},
    PREFIX_REVOKED_PERMITS,
    RESPONSE_BLOCK_SIZE,
};

use shade_protocol::{
    lb_libraries::lb_token::{
        expiration::Expiration,
        metadata::Metadata,
        permissions::Permission,
        state_structs::{
            ContractConfig,
            CurateTokenId,
            StoredTokenInfo,
            TknConfig,
            TokenAmount,
            TokenInfoMsg,
        },
    },
    liquidity_book::lb_token::{
        ExecuteAnswer,
        ExecuteMsg,
        InstantiateMsg,
        ResponseStatus::Success,
        SendAction,
        Snip1155ReceiveMsg,
        TransferAction,
    },
    s_toolkit::{
        crypto::sha_256,
        permit::RevokedPermits,
        utils::space_pad,
        viewing_key::{ViewingKey, ViewingKeyStore},
    },
};
/////////////////////////////////////////////////////////////////////////////////
// Init
/////////////////////////////////////////////////////////////////////////////////

// pub fn try_curate_token_ids(
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

pub fn try_mint_tokens(
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
pub fn try_burn_tokens(
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
                "token_id does not exist. Cannot burn non-existent `token_ids`. Use `curate_token_ids` to create tokens on new `token_ids`",
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

pub fn try_change_metadata(
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
            )));
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
            )));
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
pub fn try_transfer(
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

pub fn try_batch_transfer(
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

pub fn try_send(
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

pub fn try_batch_send(
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
pub fn try_give_permission(
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
pub fn try_revoke_permission(
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

pub fn try_create_viewing_key(
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

pub fn try_set_viewing_key(
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

pub fn try_revoke_permit(
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

// pub fn try_add_curators(
//     deps: DepsMut,
//     _env: Env,
//     info: MessageInfo,
//     add_curators: Vec<Addr>,
// ) -> StdResult<Response> {
//     let mut config = contr_conf_r(deps.storage).load()?;

//     // verify admin
//     verify_admin(&config, &info)?;

//     // add curators
//     for curator in add_curators {
//         config.curators.push(curator);
//     }
//     contr_conf_w(deps.storage).save(&config)?;

//     Ok(Response::new().set_data(to_binary(&ExecuteAnswer::AddCurators { status: Success })?))
// }

// pub fn try_remove_curators(
//     deps: DepsMut,
//     _env: Env,
//     info: MessageInfo,
//     remove_curators: Vec<Addr>,
// ) -> StdResult<Response> {
//     let mut config = contr_conf_r(deps.storage).load()?;

//     // verify admin
//     verify_admin(&config, &info)?;

//     // remove curators
//     for curator in remove_curators {
//         config.curators.retain(|x| x != &curator);
//     }
//     contr_conf_w(deps.storage).save(&config)?;

//     Ok(
//         Response::new().set_data(to_binary(&ExecuteAnswer::RemoveCurators {
//             status: Success,
//         })?),
//     )
// }

// pub fn try_add_minters(
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

// pub fn try_remove_minters(
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

//No need to change admin cause we're using admin_auth

pub fn try_change_admin(
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

pub fn try_remove_admin(
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

pub fn try_register_receive(
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

pub fn pad_response(response: StdResult<Response>) -> StdResult<Response> {
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
pub fn exec_curate_token_id(
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
                )));
            }
            // not enough allowance to transfer amount
            Some(perm) if perm.trfer_allowance_perm < amount => {
                return Err(StdError::generic_err(format!(
                    "Insufficient transfer allowance: {}",
                    perm.trfer_allowance_perm
                )));
            }
            // success, so need to reduce allowance
            Some(mut perm) if perm.trfer_allowance_perm >= amount => {
                let new_allowance = perm
                    .trfer_allowance_perm
                    .checked_sub(amount)
                    .expect("something strange happened");
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
            ));
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
        balances_w(storage, token_id)
            .save(to_binary(&from)?.as_slice(), &from_new_amount_op.unwrap())?;

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

        // println!("to_existing_bal {:?}", to_existing_bal);

        // println!("amount {:?}", amount);

        let to_new_amount_op = to_existing_bal.checked_add(*amount);
        if to_new_amount_op.is_err() {
            return Err(StdError::generic_err(
                "recipient will become too rich. Total tokens exceeds 2^128",
            ));
        }

        // save new balances
        balances_w(storage, token_id)
            .save(to_binary(&to)?.as_slice(), &to_new_amount_op.unwrap())?;

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
                Ok(i) => i,
                Err(_e) => {
                    return Err(StdError::generic_err(
                        "total supply exceeds max allowed of 2^128",
                    ));
                }
            };
            tkn_tot_supply_w(storage).save(token_info.token_id.as_bytes(), &new_amount)?;
        }
        (Some(_), None) => {
            let old_amount = tkn_tot_supply_r(storage).load(token_info.token_id.as_bytes())?;
            let new_amount_op = old_amount.checked_sub(*amount);
            let new_amount = match new_amount_op {
                Ok(i) => i,
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
