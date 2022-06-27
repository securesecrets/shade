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
use cosmwasm_std::{
    from_binary,
    to_binary,
    Api,
    Binary,
    Env,
    Extern,
    HandleResponse,
    HandleResult,
    InitResponse,
    Querier,
    QueryResult,
    StdError,
    StdResult,
    Storage,
};
use secret_toolkit::utils::{pad_handle_result, pad_query_result};
use shade_protocol::{
    contract_interfaces::snip20::{
        manager::{ContractStatusLevel, Key, PermitKey},
        HandleAnswer,
        HandleMsg,
        InitMsg,
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
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    msg.save(&mut deps.storage, env)?;
    Ok(InitResponse {
        messages: vec![],
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    // Check if transfers are allowed
    let status = ContractStatusLevel::load(&deps.storage)?;
    match status {
        // Ignore if normal run
        ContractStatusLevel::NormalRun => {}
        // Allow only status level updates or redeeming
        ContractStatusLevel::StopAllButRedeems | ContractStatusLevel::StopAll => match msg {
            HandleMsg::Redeem { .. } => {
                if status != ContractStatusLevel::StopAllButRedeems {
                    return Err(action_disabled());
                }
            }
            HandleMsg::SetContractStatus { .. } => {}
            _ => return Err(action_disabled()),
        },
    }

    pad_handle_result(
        match msg {
            HandleMsg::Redeem { amount, denom, .. } => try_redeem(deps, env, amount),

            HandleMsg::Deposit { .. } => try_deposit(deps, env),

            HandleMsg::Transfer {
                recipient,
                amount,
                memo,
                ..
            } => try_transfer(deps, env, recipient, amount, memo),

            HandleMsg::Send {
                recipient,
                recipient_code_hash,
                amount,
                msg,
                memo,
                ..
            } => try_send(deps, env, recipient, recipient_code_hash, amount, memo, msg),

            HandleMsg::BatchTransfer { actions, .. } => try_batch_transfer(deps, env, actions),

            HandleMsg::BatchSend { actions, .. } => try_batch_send(deps, env, actions),

            HandleMsg::Burn { amount, memo, .. } => try_burn(deps, env, amount, memo),

            HandleMsg::RegisterReceive { code_hash, .. } => {
                try_register_receive(deps, env, code_hash)
            }

            HandleMsg::CreateViewingKey { entropy, .. } => {
                try_create_viewing_key(deps, env, entropy)
            }

            HandleMsg::SetViewingKey { key, .. } => try_set_viewing_key(deps, env, key),

            HandleMsg::IncreaseAllowance {
                spender,
                amount,
                expiration,
                ..
            } => try_increase_allowance(deps, env, spender, amount, expiration),

            HandleMsg::DecreaseAllowance {
                spender,
                amount,
                expiration,
                ..
            } => try_decrease_allowance(deps, env, spender, amount, expiration),

            HandleMsg::TransferFrom {
                owner,
                recipient,
                amount,
                memo,
                ..
            } => try_transfer_from(deps, env, owner, recipient, amount, memo),

            HandleMsg::SendFrom {
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

            HandleMsg::BatchTransferFrom { actions, .. } => {
                try_batch_transfer_from(deps, env, actions)
            }

            HandleMsg::BatchSendFrom { actions, .. } => try_batch_send_from(deps, env, actions),

            HandleMsg::BurnFrom {
                owner,
                amount,
                memo,
                ..
            } => try_burn_from(deps, env, owner, amount, memo),

            HandleMsg::BatchBurnFrom { actions, .. } => try_batch_burn_from(deps, env, actions),

            HandleMsg::Mint {
                recipient,
                amount,
                memo,
                ..
            } => try_mint(deps, env, recipient, amount, memo),

            HandleMsg::BatchMint { actions, .. } => try_batch_mint(deps, env, actions),

            HandleMsg::AddMinters { minters, .. } => try_add_minters(deps, env, minters),

            HandleMsg::RemoveMinters { minters, .. } => try_remove_minters(deps, env, minters),

            HandleMsg::SetMinters { minters, .. } => try_set_minters(deps, env, minters),

            HandleMsg::ChangeAdmin { address, .. } => try_change_admin(deps, env, address),

            HandleMsg::SetContractStatus { level, .. } => try_set_contract_status(deps, env, level),

            HandleMsg::RevokePermit { permit_name, .. } => {
                try_revoke_permit(deps, env, permit_name)
            }
        },
        RESPONSE_BLOCK_SIZE,
    )
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    pad_query_result(
        to_binary(&match msg {
            QueryMsg::TokenInfo {} => query::token_info(deps)?,
            QueryMsg::TokenConfig {} => query::token_config(deps)?,
            QueryMsg::ContractStatus {} => query::contract_status(deps)?,
            QueryMsg::ExchangeRate {} => query::exchange_rate(deps)?,
            QueryMsg::Minters {} => query::minters(deps)?,

            QueryMsg::WithPermit { permit, query } => {
                // Validate permit and get account
                let account = permit.validate(&deps.api, None)?.as_humanaddr(None)?;

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
