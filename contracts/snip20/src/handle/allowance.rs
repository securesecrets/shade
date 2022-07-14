use shade_protocol::c_std::{Api, Binary, Env, Extern, HandleResponse, HumanAddr, Querier, StdResult, Storage, to_binary};
use shade_protocol::math_compat::Uint128;
use shade_protocol::contract_interfaces::snip20::{batch, HandleAnswer};
use shade_protocol::contract_interfaces::snip20::manager::{Allowance, CoinInfo};
use shade_protocol::utils::generic_response::ResponseStatus::Success;
use shade_protocol::utils::storage::plus::{ItemStorage, MapStorage};
use crate::handle::transfers::{try_send_impl, try_transfer_impl};

pub fn try_increase_allowance<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    spender: HumanAddr,
    amount: Uint128,
    expiration: Option<u64>,
) -> StdResult<HandleResponse> {
    let owner = env.message.sender;

    let mut allowance = Allowance::may_load(
        &deps.storage,
        (owner.clone(), spender.clone())
    )?.unwrap_or(Allowance::default());

    // Reset allowance if its expired
    if allowance.is_expired(&env.block) {
        allowance.amount = amount;
        allowance.expiration = None;
    } else {
        allowance.amount = match allowance.amount.checked_add(amount) {
            Ok(amount) => amount,
            Err(_) => Uint128::MAX
        }
    }

    if expiration.is_some() {
        allowance.expiration = expiration;
    }

    allowance.save(&mut deps.storage, (owner.clone(), spender.clone()))?;

    Ok(HandleResponse{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::IncreaseAllowance {
            spender,
            owner,
            allowance: allowance.amount
        })?)
    })
}

pub fn try_decrease_allowance<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    spender: HumanAddr,
    amount: Uint128,
    expiration: Option<u64>,
) -> StdResult<HandleResponse> {
    let owner = env.message.sender;

    let mut allowance = Allowance::load(&deps.storage, (owner.clone(), spender.clone()))?;

    // Reset allowance if its expired
    if allowance.is_expired(&env.block) {
        allowance = Allowance::default();
    } else {
        allowance.amount = match allowance.amount.checked_sub(amount) {
            Ok(amount) => amount,
            Err(_) => Uint128::zero()
        }
    }

    if expiration.is_some() {
        allowance.expiration = expiration;
    }

    allowance.save(&mut deps.storage, (owner.clone(), spender.clone()))?;

    Ok(HandleResponse{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::IncreaseAllowance {
            spender,
            owner,
            allowance: allowance.amount
        })?)
    })
}

pub fn try_transfer_from<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: HumanAddr,
    recipient: HumanAddr,
    amount: Uint128,
    memo: Option<String>,
) -> StdResult<HandleResponse> {
    let denom = CoinInfo::load(&deps.storage)?.symbol;
    try_transfer_impl(
        &mut deps.storage,
        &env.message.sender,
        Some(&owner),
        &recipient,
        amount,
        memo,
        denom,
        &env.block
    )?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::TransferFrom { status: Success })?),
    })
}

pub fn try_batch_transfer_from<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    actions: Vec<batch::TransferFromAction>,
) -> StdResult<HandleResponse> {
    let denom = CoinInfo::load(&deps.storage)?.symbol;
    let block = &env.block;
    for action in actions {
        try_transfer_impl(
            &mut deps.storage,
            &env.message.sender,
            Some(&action.owner),
            &action.recipient,
            action.amount,
            action.memo,
            denom.clone(),
            block
        )?;
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BatchTransferFrom {
            status: Success,
        })?),
    })
}

pub fn try_send_from<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: HumanAddr,
    recipient: HumanAddr,
    recipient_code_hash: Option<String>,
    amount: Uint128,
    msg: Option<Binary>,
    memo: Option<String>,
) -> StdResult<HandleResponse> {
    let mut messages = vec![];
    let denom = CoinInfo::load(&deps.storage)?.symbol;
    try_send_impl(
        &mut deps.storage,
        &mut messages,
        &env.message.sender,
        Some(&owner),
        &recipient,
        recipient_code_hash,
        amount,
        memo,
        msg,
        denom,
        &env.block
    )?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SendFrom { status: Success })?),
    })
}

pub fn try_batch_send_from<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    actions: Vec<batch::SendFromAction>
) -> StdResult<HandleResponse> {
    let mut messages = vec![];
    let sender = env.message.sender;
    let denom = CoinInfo::load(&deps.storage)?.symbol;

    for action in actions {
        try_send_impl(
            &mut deps.storage,
            &mut messages,
            &sender,
            Some(&action.owner),
            &action.recipient,
            action.recipient_code_hash,
            action.amount,
            action.memo,
            action.msg,
            denom.clone(),
            &env.block
        )?;
    }

    Ok(HandleResponse{
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BatchSendFrom { status: Success })?)
    })
}