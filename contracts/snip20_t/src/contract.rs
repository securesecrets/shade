use cosmwasm_std::{Api, Binary, Env, Extern, from_binary, HandleResponse, HandleResult, InitResponse, Querier, StdError, StdResult, Storage, to_binary};
use secret_toolkit::utils::{pad_handle_result, pad_query_result};
use shade_protocol::contract_interfaces::snip20_test::{InitMsg, HandleMsg, HandleAnswer, QueryMsg, QueryAnswer, Permission, QueryWithPermit};
use shade_protocol::contract_interfaces::snip20_test::manager::{Key, PermitKey};
use shade_protocol::utils::storage::plus::MapStorage;
use crate::query;
use crate::handle::transfers::{try_batch_send, try_batch_transfer, try_send, try_transfer};
use crate::handle::{try_change_admin, try_create_viewing_key, try_deposit, try_redeem, try_register_receive, try_revoke_permit, try_set_contract_status, try_set_viewing_key};
use crate::handle::allowance::{try_batch_send_from, try_batch_transfer_from, try_decrease_allowance, try_increase_allowance, try_send_from, try_transfer_from};
use crate::handle::burning::{try_batch_burn_from, try_burn, try_burn_from};
use crate::handle::minting::{try_add_minters, try_batch_mint, try_mint, try_remove_minters, try_set_minters};

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
    // TODO: implement contract status
    pad_handle_result(
        match msg {
            HandleMsg::Redeem { amount, denom, ..
            } => try_redeem(deps, env, amount),

            HandleMsg::Deposit { ..
            } => try_deposit(deps, env),

            HandleMsg::Transfer { recipient, amount, memo, ..
            } => try_transfer(deps, env, recipient, amount, memo),

            HandleMsg::Send { recipient, recipient_code_hash, amount, msg, memo, ..
            } => try_send(deps, env, recipient, recipient_code_hash, amount, memo, msg),

            HandleMsg::BatchTransfer { actions, ..
            } => try_batch_transfer(deps, env, actions),

            HandleMsg::BatchSend { actions, ..
            } => try_batch_send(deps, env, actions),

            HandleMsg::Burn { amount, memo, ..
            } => try_burn(deps, env, amount, memo),

            HandleMsg::RegisterReceive { code_hash, ..
            } => try_register_receive(deps, env, code_hash),

            HandleMsg::CreateViewingKey { entropy, ..
            } => try_create_viewing_key(deps, env, entropy),

            HandleMsg::SetViewingKey { key, ..
            } => try_set_viewing_key(deps, env, key),

            HandleMsg::IncreaseAllowance { spender, amount, expiration, ..
            } => try_increase_allowance(deps, env, spender, amount, expiration),

            HandleMsg::DecreaseAllowance { spender, amount, expiration, ..
            } => try_decrease_allowance(deps, env, spender, amount, expiration),

            HandleMsg::TransferFrom { owner, recipient, amount, memo, ..
            } => try_transfer_from(deps, env, owner, recipient, amount, memo),

            HandleMsg::SendFrom { owner, recipient, recipient_code_hash, amount, msg, memo, ..
            } => try_send_from(deps, env, owner, recipient, recipient_code_hash, amount, msg, memo),

            HandleMsg::BatchTransferFrom { actions, ..
            } => try_batch_transfer_from(deps, env, actions),

            HandleMsg::BatchSendFrom { actions, ..
            } => try_batch_send_from(deps, env, actions),

            HandleMsg::BurnFrom { owner, amount, memo, ..
            } => try_burn_from(deps, env, owner, amount, memo),

            HandleMsg::BatchBurnFrom { actions, ..
            } => try_batch_burn_from(deps, env, actions),

            HandleMsg::Mint { recipient, amount, memo, ..
            } => try_mint(deps, env, recipient, amount, memo),

            HandleMsg::BatchMint { actions, ..
            } => try_batch_mint(deps, env, actions),

            HandleMsg::AddMinters { minters, ..
            } => try_add_minters(deps, env, minters),

            HandleMsg::RemoveMinters { minters, ..
            } => try_remove_minters(deps, env, minters),

            HandleMsg::SetMinters { minters, ..
            } => try_set_minters(deps, env, minters),

            HandleMsg::ChangeAdmin { address, ..
            } => try_change_admin(deps, env, address),

            HandleMsg::SetContractStatus { level, ..
            } => try_set_contract_status(deps, env, level),

            HandleMsg::RevokePermit { permit_name, ..
            } => try_revoke_permit(deps, env, permit_name),
        },
        RESPONSE_BLOCK_SIZE,
    )
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    pad_query_result(
        to_binary(&match msg {
            QueryMsg::TokenInfo { } => query::token_info(deps),
            QueryMsg::TokenConfig { } => query::token_config(deps),
            QueryMsg::ContractStatus { } => query::contract_status(deps),
            QueryMsg::ExchangeRate { } => query::exchange_rate(deps),
            QueryMsg::Minters { } => query::minters(deps),

            QueryMsg::WithPermit { permit, query } => {
                // Validate permit and get account
                let account = permit.validate(None)?.as_humanaddr(&deps.api)?;

                // Check that permit is not revoked
                if PermitKey::may_load(&deps.storage, (account.clone(), permit.params.permit_name.clone()))?.is_some() {
                    return Err(StdError::generic_err("Permit key is revoked"))
                }

                match query {
                    QueryWithPermit::Allowance { owner, spender, .. } => {
                        if !permit.params.contains(Permission::Allowance) {
                            return Err(StdError::generic_err("No permission to query allowance"))
                        }

                        if owner != account && spender != account {
                            return Err(StdError::generic_err("Only allowance owner or spender can query this"))
                        }

                        query::allowance(deps, owner, spender)
                    }
                    QueryWithPermit::Balance { } => {
                        if !permit.params.contains(Permission::Balance) {
                            return Err(StdError::generic_err("No permission to query balance"))
                        }

                        query::balance(deps, account.clone())
                    }
                    QueryWithPermit::TransferHistory {page, page_size } => {
                        if !permit.params.contains(Permission::History) {
                            return Err(StdError::generic_err("No permission to query history"))
                        }

                        query::transfer_history(deps, account.clone(), page.unwrap_or(0), page_size)
                    }
                    QueryWithPermit::TransactionHistory { page, page_size } => {
                        if !permit.params.contains(Permission::History) {
                            return Err(StdError::generic_err("No permission to query history"))
                        }

                        query::transaction_history(deps, account.clone(), page.unwrap_or(0), page_size)
                    }
                }
            }

            _ => {
                match msg {
                    QueryMsg::Allowance { owner, spender, key } => {
                        if Key::verify(&deps.storage, owner.clone(), key.clone())? ||
                            Key::verify(&deps.storage, spender.clone(), key)? {
                            query::allowance(deps, owner, spender)
                        }

                        else {
                            return Err(StdError::generic_err("Invalid viewing key"))
                        }
                    }
                    QueryMsg::Balance { address, key } => {
                        if Key::verify(&deps.storage, address.clone(), key.clone())? {
                            query::balance(deps, address.clone())
                        }

                        else {
                            return Err(StdError::generic_err("Invalid viewing key"))
                        }
                    }
                    QueryMsg::TransferHistory { address, key, page, page_size } => {
                        if Key::verify(&deps.storage, address.clone(), key.clone())? {
                            query::transfer_history(deps, address.clone(), page.unwrap_or(0), page_size)
                        }

                        else {
                            return Err(StdError::generic_err("Invalid viewing key"))
                        }
                    }
                    QueryMsg::TransactionHistory { address, key, page, page_size } => {
                        if Key::verify(&deps.storage, address.clone(), key.clone())? {
                            query::transaction_history(deps, address.clone(), page.unwrap_or(0), page_size)
                        }

                        else {
                            return Err(StdError::generic_err("Invalid viewing key"))
                        }
                    }
                    _ => return Err(StdError::generic_err("Not an authenticated msg"))
                }
            }
        }),
        RESPONSE_BLOCK_SIZE,
    )
}