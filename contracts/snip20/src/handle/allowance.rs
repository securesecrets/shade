use crate::handle::transfers::{try_send_impl, try_transfer_impl};
use shade_protocol::{
    c_std::{to_binary, Addr, Binary, DepsMut, Env, MessageInfo, Response, StdResult, Uint128},
    contract_interfaces::snip20::{
        batch,
        manager::{Allowance, CoinInfo},
        ExecuteAnswer,
    },
    utils::{
        generic_response::ResponseStatus::Success,
        storage::plus::{ItemStorage, MapStorage},
    },
};

pub fn try_increase_allowance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: Addr,
    amount: Uint128,
    expiration: Option<u64>,
) -> StdResult<Response> {
    let owner = info.sender;
    let mut allowance = Allowance::may_load(deps.storage, (owner.clone(), spender.clone()))?
        .unwrap_or(Allowance::default());

    // Reset allowance if its expired
    if allowance.is_expired(&env.block) {
        allowance.amount = amount;
        allowance.expiration = None;
    } else {
        allowance.amount = match allowance.amount.checked_add(amount) {
            Ok(amount) => amount,
            Err(_) => Uint128::MAX,
        }
    }

    if expiration.is_some() {
        allowance.expiration = expiration;
    }

    allowance.save(deps.storage, (owner.clone(), spender.clone()))?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::IncreaseAllowance {
            spender,
            owner,
            allowance: allowance.amount,
        })?),
    )
}

pub fn try_decrease_allowance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    spender: Addr,
    amount: Uint128,
    expiration: Option<u64>,
) -> StdResult<Response> {
    let owner = info.sender;

    let mut allowance = Allowance::load(deps.storage, (owner.clone(), spender.clone()))?;

    // Reset allowance if its expired
    if allowance.is_expired(&env.block) {
        allowance = Allowance::default();
    } else {
        allowance.amount = match allowance.amount.checked_sub(amount) {
            Ok(amount) => amount,
            Err(_) => Uint128::zero(),
        }
    }

    if expiration.is_some() {
        allowance.expiration = expiration;
    }

    allowance.save(deps.storage, (owner.clone(), spender.clone()))?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::IncreaseAllowance {
            spender,
            owner,
            allowance: allowance.amount,
        })?),
    )
}

pub fn try_transfer_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: Addr,
    recipient: Addr,
    amount: Uint128,
    memo: Option<String>,
) -> StdResult<Response> {
    let denom = CoinInfo::load(deps.storage)?.symbol;
    try_transfer_impl(
        deps.storage,
        &info.sender,
        Some(&owner),
        &recipient,
        amount,
        memo,
        denom,
        &env.block,
    )?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::TransferFrom { status: Success })?))
}

pub fn try_batch_transfer_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    actions: Vec<batch::TransferFromAction>,
) -> StdResult<Response> {
    let denom = CoinInfo::load(deps.storage)?.symbol;
    let block = &env.block;
    for action in actions {
        try_transfer_impl(
            deps.storage,
            &info.sender,
            Some(&deps.api.addr_validate(action.owner.as_str())?),
            &deps.api.addr_validate(action.recipient.as_str())?,
            action.amount,
            action.memo,
            denom.clone(),
            block,
        )?;
    }

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::BatchTransferFrom {
            status: Success,
        })?),
    )
}

pub fn try_send_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: Addr,
    recipient: Addr,
    recipient_code_hash: Option<String>,
    amount: Uint128,
    msg: Option<Binary>,
    memo: Option<String>,
) -> StdResult<Response> {
    let mut messages = vec![];
    let denom = CoinInfo::load(deps.storage)?.symbol;
    try_send_impl(
        deps.storage,
        &mut messages,
        &info.sender,
        Some(&owner),
        &recipient,
        recipient_code_hash,
        amount,
        memo,
        msg,
        denom,
        &env.block,
    )?;

    Ok(Response::new()
        .set_data(to_binary(&ExecuteAnswer::SendFrom { status: Success })?)
        .add_submessages(messages))
}

pub fn try_batch_send_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    actions: Vec<batch::SendFromAction>,
) -> StdResult<Response> {
    let mut messages = vec![];
    let sender = info.sender;
    let denom = CoinInfo::load(deps.storage)?.symbol;

    for action in actions {
        try_send_impl(
            deps.storage,
            &mut messages,
            &sender,
            Some(&deps.api.addr_validate(action.owner.as_str())?),
            &deps.api.addr_validate(action.recipient.as_str())?,
            action.recipient_code_hash,
            action.amount,
            action.memo,
            action.msg,
            denom.clone(),
            &env.block,
        )?;
    }

    Ok(Response::new()
        .set_data(to_binary(&ExecuteAnswer::BatchSendFrom { status: Success })?)
        .add_submessages(messages))
}
