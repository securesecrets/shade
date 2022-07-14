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
    },
    query,
};
use shade_protocol::c_std::{from_binary, to_binary, Api, Binary, Env, DepsMut, Response, Querier, StdError, StdResult, Storage, MessageInfo};
use shade_protocol::utils::{pad_handle_result, pad_query_result};
use shade_protocol::{
    contract_interfaces::snip20::{
        manager::{ContractStatusLevel, Key, PermitKey},
        HandleAnswer,
        ExecuteMsg,
        InstantiateMsg,
        Permission,
        QueryAnswer,
        QueryMsg,
        QueryWithPermit,
    },
    utils::storage::plus::MapStorage,
};
use shade_protocol::contract_interfaces::snip20::errors::{action_disabled, invalid_viewing_key, not_authenticated_msg, permit_revoked, unauthorized_permit};

// Used to pad up responses for better privacy.
pub const RESPONSE_BLOCK_SIZE: usize = 256;
pub const PREFIX_REVOKED_PERMITS: &str = "revoked_permits";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    msg.save(deps.storage, env, info)?;
    Ok(Response::new())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    // Check if transfers are allowed
    let status = ContractStatusLevel::load(&deps.storage)?;
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
            ExecuteMsg::Redeem { amount, denom, .. } => try_redeem(deps, env, amount),

            ExecuteMsg::Deposit { .. } => try_deposit(deps, env),

            ExecuteMsg::Transfer {
                recipient,
                amount,
                memo,
                ..
            } => try_transfer(deps, env, recipient, amount, memo),

            ExecuteMsg::Send {
                recipient,
                recipient_code_hash,
                amount,
                msg,
                memo,
                ..
            } => try_send(deps, env, recipient, recipient_code_hash, amount, memo, msg),

            ExecuteMsg::BatchTransfer { actions, .. } => try_batch_transfer(deps, env, actions),

            ExecuteMsg::BatchSend { actions, .. } => try_batch_send(deps, env, actions),

            ExecuteMsg::Burn { amount, memo, .. } => try_burn(deps, env, amount, memo),

            ExecuteMsg::RegisterReceive { code_hash, .. } => {
                try_register_receive(deps, env, code_hash)
            }

            ExecuteMsg::CreateViewingKey { entropy, .. } => {
                try_create_viewing_key(deps, env, entropy)
            }

            ExecuteMsg::SetViewingKey { key, .. } => try_set_viewing_key(deps, env, key),

            ExecuteMsg::IncreaseAllowance {
                spender,
                amount,
                expiration,
                ..
            } => try_increase_allowance(deps, env, spender, amount, expiration),

            ExecuteMsg::DecreaseAllowance {
                spender,
                amount,
                expiration,
                ..
            } => try_decrease_allowance(deps, env, spender, amount, expiration),

            ExecuteMsg::TransferFrom {
                owner,
                recipient,
                amount,
                memo,
                ..
            } => try_transfer_from(deps, env, owner, recipient, amount, memo),

            ExecuteMsg::SendFrom {
                owner,
                recipient,
                recipient_code_hash,
                amount,
                msg,
                memo,
                ..
            } => try_send_from(
                deps,
                env,
                owner,
                recipient,
                recipient_code_hash,
                amount,
                msg,
                memo,
            ),

            ExecuteMsg::BatchTransferFrom { actions, .. } => {
                try_batch_transfer_from(deps, env, actions)
            }

            ExecuteMsg::BatchSendFrom { actions, .. } => try_batch_send_from(deps, env, actions),

            ExecuteMsg::BurnFrom {
                owner,
                amount,
                memo,
                ..
            } => try_burn_from(deps, env, owner, amount, memo),

            ExecuteMsg::BatchBurnFrom { actions, .. } => try_batch_burn_from(deps, env, actions),

            ExecuteMsg::Mint {
                recipient,
                amount,
                memo,
                ..
            } => try_mint(deps, env, recipient, amount, memo),

            ExecuteMsg::BatchMint { actions, .. } => try_batch_mint(deps, env, actions),

            ExecuteMsg::AddMinters { minters, .. } => try_add_minters(deps, env, minters),

            ExecuteMsg::RemoveMinters { minters, .. } => try_remove_minters(deps, env, minters),

            ExecuteMsg::SetMinters { minters, .. } => try_set_minters(deps, env, minters),

            ExecuteMsg::ChangeAdmin { address, .. } => try_change_admin(deps, env, address),

            ExecuteMsg::SetContractStatus { level, .. } => try_set_contract_status(deps, env, level),

            ExecuteMsg::RevokePermit { permit_name, .. } => {
                try_revoke_permit(deps, env, permit_name)
            }
        },
        RESPONSE_BLOCK_SIZE,
    )
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
    pad_query_result(
        to_binary(&match msg {
            QueryMsg::TokenInfo {} => query::token_info(deps)?,
            QueryMsg::TokenConfig {} => query::token_config(deps)?,
            QueryMsg::ContractStatus {} => query::contract_status(deps)?,
            QueryMsg::ExchangeRate {} => query::exchange_rate(deps)?,
            QueryMsg::Minters {} => query::minters(deps)?,

            QueryMsg::WithPermit { permit, query } => {
                // Validate permit and get account
                let account = permit.validate(&deps.api, None)?.as_Addr(None)?;

                // Check that permit is not revoked
                if PermitKey::may_load(
                    &deps.storage,
                    (account.clone(), permit.params.permit_name.clone()),
                )?
                .is_some()
                {
                    return Err(permit_revoked(permit.params.permit_name));
                }

                match query {
                    QueryWithPermit::Allowance { owner, spender, .. } => {
                        if !permit.params.contains(Permission::Allowance) {
                            return Err(unauthorized_permit(Permission::Allowance));
                        }

                        if owner != account && spender != account {
                            return Err(unauthorized_permit(Permission::Allowance));
                        }

                        query::allowance(deps, owner, spender)?
                    }
                    QueryWithPermit::Balance {} => {
                        if !permit.params.contains(Permission::Balance) {
                            return Err(unauthorized_permit(Permission::Balance));
                        }

                        query::balance(deps, account.clone())?
                    }
                    QueryWithPermit::TransferHistory { page, page_size } => {
                        if !permit.params.contains(Permission::History) {
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
                        if !permit.params.contains(Permission::History) {
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
                    if Key::verify(&deps.storage, owner.clone(), key.clone())?
                        || Key::verify(&deps.storage, spender.clone(), key)?
                    {
                        query::allowance(deps, owner, spender)?
                    } else {
                        return Err(invalid_viewing_key());
                    }
                }
                QueryMsg::Balance { address, key } => {
                    if Key::verify(&deps.storage, address.clone(), key.clone())? {
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
                    if Key::verify(&deps.storage, address.clone(), key.clone())? {
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
                    if Key::verify(&deps.storage, address.clone(), key.clone())? {
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
