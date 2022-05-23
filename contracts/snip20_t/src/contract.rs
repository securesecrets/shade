use cosmwasm_std::{Api, Binary, Env, Extern, from_binary, HandleResponse, HandleResult, InitResponse, Querier, StdError, StdResult, Storage, to_binary};
use secret_toolkit::utils::{pad_handle_result, pad_query_result};
use shade_protocol::contract_interfaces::snip20_test::{InitMsg, HandleMsg, HandleAnswer, QueryMsg, QueryAnswer, Extended};
use crate::handle::transfers::{try_batch_transfer, try_transfer};

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
    pad_handle_result(
        match msg {
            HandleMsg::Redeem { amount, denom, ..
            } => try_redeem(deps, env, amount, denom),

            HandleMsg::Deposit { ..
            } => try_deposit(deps, env),

            HandleMsg::Transfer { recipient, amount, memo, ..
            } => try_transfer(deps, env, recipient, amount, memo),

            HandleMsg::Send { recipient, recipient_code_hash, amount, msg, memo, ..
            } => try_send(deps, env, recipient, recipient_code_hash, amount, msg, memo),

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
        match msg {
            QueryMsg::TokenInfo { .. } => {}
            QueryMsg::TokenConfig { .. } => {}
            QueryMsg::ContractStatus { .. } => {}
            QueryMsg::ExchangeRate { .. } => {}
            QueryMsg::Allowance { .. } => {}
            QueryMsg::Balance { .. } => {}
            QueryMsg::TransferHistory { .. } => {}
            QueryMsg::TransactionHistory { .. } => {}
            QueryMsg::Minters { .. } => {}
            QueryMsg::WithPermit { .. } => {}
        },
        RESPONSE_BLOCK_SIZE,
    )
}
