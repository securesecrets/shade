use crate::{
    handle::{
        allowance::{
            try_batch_send_from,
            try_batch_transfer_from,
            try_decrease_allowance,
            try_increase_allowance,
            try_send_from,
            try_transfer_from,
        },
        burning::{try_batch_burn_from, try_burn, try_burn_from},
        minting::{try_add_minters, try_batch_mint, try_mint, try_remove_minters, try_set_minters},
        transfers::{try_batch_send, try_batch_transfer, try_send, try_transfer},
        try_change_admin,
        try_create_viewing_key,
        try_deposit,
        try_redeem,
        try_register_receive,
        try_revoke_permit,
        try_set_contract_status,
        try_set_viewing_key,
        try_update_query_auth,
    },
    query,
};
use shade_protocol::{
    c_std::{
        shd_entry_point,
        to_binary,
        Addr,
        Binary,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdResult,
    },
    contract_interfaces::snip20::{
        errors::{
            action_disabled,
            invalid_viewing_key,
            not_authenticated_msg,
            permit_revoked,
            unauthorized_permit,
        },
        manager::{ContractStatusLevel, Key, PermitKey},
        ExecuteMsg,
        InstantiateMsg,
        Permission,
        QueryMsg,
        QueryWithPermit,
    },
    query_auth::helpers::{authenticate_permit, authenticate_vk, PermitAuthentication},
    snip20::{errors::permit_not_found, manager::QueryAuth, PermitParams},
    utils::{
        asset::validate_vec,
        pad_handle_result,
        pad_query_result,
        storage::plus::{ItemStorage, MapStorage},
    },
};

// Used to pad up responses for better privacy.
pub const RESPONSE_BLOCK_SIZE: usize = 256;
pub const PREFIX_REVOKED_PERMITS: &str = "revoked_permits";

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    msg.save(deps.storage, deps.api, env, info)?;
    Ok(Response::new())
}

#[shd_entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    // Check if transfers are allowed
    let status = ContractStatusLevel::load(deps.storage)?;
    match status {
        // Ignore if normal run
        ContractStatusLevel::NormalRun => {}
        // Allow only status level updates or redeeming
        ContractStatusLevel::StopAllButRedeems | ContractStatusLevel::StopAll => match msg {
            ExecuteMsg::Redeem { .. } => {
                if status != ContractStatusLevel::StopAllButRedeems {
                    return Err(action_disabled());
                }
            }
            ExecuteMsg::SetContractStatus { .. } => {}
            _ => return Err(action_disabled()),
        },
    }

    pad_handle_result(
        match msg {
            ExecuteMsg::Redeem {
                amount, denom: _, ..
            } => try_redeem(deps, env, info, amount),

            ExecuteMsg::Deposit { .. } => try_deposit(deps, env, info),

            ExecuteMsg::Transfer {
                recipient,
                amount,
                memo,
                ..
            } => {
                let recipient = deps.api.addr_validate(recipient.as_str())?;
                try_transfer(deps, env, info, recipient, amount, memo)
            }

            ExecuteMsg::Send {
                recipient,
                recipient_code_hash,
                amount,
                msg,
                memo,
                ..
            } => {
                let recipient = deps.api.addr_validate(recipient.as_str())?;
                try_send(
                    deps,
                    env,
                    info,
                    recipient,
                    recipient_code_hash,
                    amount,
                    memo,
                    msg,
                )
            }

            ExecuteMsg::BatchTransfer { actions, .. } => {
                try_batch_transfer(deps, env, info, actions)
            }

            ExecuteMsg::BatchSend { actions, .. } => try_batch_send(deps, env, info, actions),

            ExecuteMsg::Burn { amount, memo, .. } => try_burn(deps, env, info, amount, memo),

            ExecuteMsg::RegisterReceive { code_hash, .. } => {
                try_register_receive(deps, env, info, code_hash)
            }

            ExecuteMsg::CreateViewingKey { entropy, .. } => {
                try_create_viewing_key(deps, env, info, entropy)
            }

            ExecuteMsg::SetViewingKey { key, .. } => try_set_viewing_key(deps, env, info, key),

            ExecuteMsg::IncreaseAllowance {
                spender,
                amount,
                expiration,
                ..
            } => {
                let spender = deps.api.addr_validate(spender.as_str())?;
                try_increase_allowance(deps, env, info, spender, amount, expiration)
            }
            ExecuteMsg::DecreaseAllowance {
                spender,
                amount,
                expiration,
                ..
            } => {
                let spender = deps.api.addr_validate(spender.as_str())?;
                try_decrease_allowance(deps, env, info, spender, amount, expiration)
            }
            ExecuteMsg::TransferFrom {
                owner,
                recipient,
                amount,
                memo,
                ..
            } => {
                let owner = deps.api.addr_validate(owner.as_str())?;
                let recipient = deps.api.addr_validate(recipient.as_str())?;
                try_transfer_from(deps, env, info, owner, recipient, amount, memo)
            }
            ExecuteMsg::SendFrom {
                owner,
                recipient,
                recipient_code_hash,
                amount,
                msg,
                memo,
                ..
            } => {
                let owner = deps.api.addr_validate(owner.as_str())?;
                let recipient = deps.api.addr_validate(recipient.as_str())?;
                try_send_from(
                    deps,
                    env,
                    info,
                    owner,
                    recipient,
                    recipient_code_hash,
                    amount,
                    msg,
                    memo,
                )
            }
            ExecuteMsg::BatchTransferFrom { actions, .. } => {
                try_batch_transfer_from(deps, env, info, actions)
            }

            ExecuteMsg::BatchSendFrom { actions, .. } => {
                try_batch_send_from(deps, env, info, actions)
            }

            ExecuteMsg::BurnFrom {
                owner,
                amount,
                memo,
                ..
            } => {
                let owner = deps.api.addr_validate(owner.as_str())?;
                try_burn_from(deps, env, info, owner, amount, memo)
            }
            ExecuteMsg::BatchBurnFrom { actions, .. } => {
                try_batch_burn_from(deps, env, info, actions)
            }
            ExecuteMsg::Mint {
                recipient,
                amount,
                memo,
                ..
            } => {
                let recipient = deps.api.addr_validate(recipient.as_str())?;
                try_mint(deps, env, info, recipient, amount, memo)
            }
            ExecuteMsg::BatchMint { actions, .. } => try_batch_mint(deps, env, info, actions),
            ExecuteMsg::AddMinters { minters, .. } => {
                let minters = validate_vec(deps.api, minters)?;
                try_add_minters(deps, env, info, minters)
            }
            ExecuteMsg::RemoveMinters { minters, .. } => {
                let minters = validate_vec(deps.api, minters)?;
                try_remove_minters(deps, env, info, minters)
            }
            ExecuteMsg::SetMinters { minters, .. } => {
                let minters = validate_vec(deps.api, minters)?;
                try_set_minters(deps, env, info, minters)
            }
            ExecuteMsg::ChangeAdmin { address, .. } => {
                let address = deps.api.addr_validate(address.as_str())?;
                try_change_admin(deps, env, info, address)
            }
            ExecuteMsg::UpdateQueryAuth { auth } => try_update_query_auth(deps, env, info, auth),
            ExecuteMsg::SetContractStatus { level, .. } => {
                try_set_contract_status(deps, env, info, level)
            }

            ExecuteMsg::RevokePermit { permit_name, .. } => {
                try_revoke_permit(deps, env, info, permit_name)
            }
        },
        RESPONSE_BLOCK_SIZE,
    )
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    pad_query_result(
        to_binary(&match msg {
            QueryMsg::TokenInfo {} => query::token_info(deps)?,
            QueryMsg::TokenConfig {} => query::token_config(deps)?,
            QueryMsg::ContractStatus {} => query::contract_status(deps)?,
            QueryMsg::ExchangeRate {} => query::exchange_rate(deps)?,
            QueryMsg::Minters {} => query::minters(deps)?,

            QueryMsg::WithPermit {
                permit,
                auth_permit,
                query,
            } => {
                // Verify which authentication setting is set
                let account: Addr;
                let params: PermitParams;

                match QueryAuth::may_load(deps.storage)? {
                    None => {
                        if let Some(permit) = permit {
                            // Validate permit and get account
                            account = permit.validate(deps.api, None)?.as_addr(None)?;

                            // Check that permit is not revoked
                            if PermitKey::may_load(
                                deps.storage,
                                (account.clone(), permit.params.permit_name.clone()),
                            )?
                            .is_some()
                            {
                                return Err(permit_revoked(permit.params.permit_name));
                            }

                            params = permit.params;
                        } else {
                            return Err(permit_not_found());
                        }
                    }
                    Some(authenticator) => {
                        if let Some(permit) = auth_permit {
                            let res: PermitAuthentication<PermitParams> =
                                authenticate_permit(permit, &deps.querier, authenticator.0)?;

                            if res.revoked {
                                return Err(permit_revoked(res.data.permit_name));
                            }

                            account = res.sender;
                            params = res.data;
                        } else {
                            return Err(permit_not_found());
                        }
                    }
                };

                match query {
                    QueryWithPermit::Allowance { owner, spender, .. } => {
                        let owner = deps.api.addr_validate(&owner)?;
                        let spender = deps.api.addr_validate(&spender)?;

                        if !params.contains(Permission::Allowance) {
                            return Err(unauthorized_permit(Permission::Allowance));
                        }

                        if owner != account && spender != account {
                            return Err(unauthorized_permit(Permission::Allowance));
                        }

                        query::allowance(deps, owner, spender)?
                    }
                    QueryWithPermit::Balance {} => {
                        if !params.contains(Permission::Balance) {
                            return Err(unauthorized_permit(Permission::Balance));
                        }

                        query::balance(deps, account.clone())?
                    }
                    QueryWithPermit::TransferHistory { page, page_size } => {
                        if !params.contains(Permission::History) {
                            return Err(unauthorized_permit(Permission::History));
                        }

                        query::transfer_history(
                            deps,
                            account.clone(),
                            page.unwrap_or(0),
                            page_size,
                        )?
                    }
                    QueryWithPermit::TransactionHistory { page, page_size } => {
                        if !params.contains(Permission::History) {
                            return Err(unauthorized_permit(Permission::History));
                        }

                        query::transaction_history(
                            deps,
                            account.clone(),
                            page.unwrap_or(0),
                            page_size,
                        )?
                    }
                }
            }

            _ => match msg {
                QueryMsg::Allowance {
                    owner,
                    spender,
                    key,
                } => {
                    let owner = deps.api.addr_validate(&owner)?;
                    let spender = deps.api.addr_validate(&spender)?;
                    if try_authenticate_vk(&deps, owner.clone(), key.clone())?
                        || try_authenticate_vk(&deps, spender.clone(), key)?
                    {
                        query::allowance(deps, owner, spender)?
                    } else {
                        return Err(invalid_viewing_key());
                    }
                }
                QueryMsg::Balance { address, key } => {
                    let address = deps.api.addr_validate(&address)?;
                    if try_authenticate_vk(&deps, address.clone(), key.clone())? {
                        query::balance(deps, address.clone())?
                    } else {
                        return Err(invalid_viewing_key());
                    }
                }
                QueryMsg::TransferHistory {
                    address,
                    key,
                    page,
                    page_size,
                } => {
                    let address = deps.api.addr_validate(&address)?;
                    if try_authenticate_vk(&deps, address.clone(), key.clone())? {
                        query::transfer_history(
                            deps,
                            address.clone(),
                            page.unwrap_or(0),
                            page_size,
                        )?
                    } else {
                        return Err(invalid_viewing_key());
                    }
                }
                QueryMsg::TransactionHistory {
                    address,
                    key,
                    page,
                    page_size,
                } => {
                    let address = deps.api.addr_validate(&address)?;
                    if try_authenticate_vk(&deps, address.clone(), key.clone())? {
                        query::transaction_history(
                            deps,
                            address.clone(),
                            page.unwrap_or(0),
                            page_size,
                        )?
                    } else {
                        return Err(invalid_viewing_key());
                    }
                }
                _ => return Err(not_authenticated_msg()),
            },
        }),
        RESPONSE_BLOCK_SIZE,
    )
}

fn try_authenticate_vk(deps: &Deps, address: Addr, key: String) -> StdResult<bool> {
    match QueryAuth::may_load(deps.storage)? {
        None => Key::verify(deps.storage, address, key),
        Some(authenticator) => authenticate_vk(address, key, &deps.querier, &authenticator.0),
    }
}
