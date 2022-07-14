use shade_protocol::c_std::{Api, Binary, CosmosMsg, Env, DepsMut, Response, Addr, Querier, StdError, StdResult, Storage, to_binary};
use shade_protocol::c_std::Uint128;
use shade_protocol::contract_interfaces::snip20::{batch, HandleAnswer, ReceiverHandleMsg};
use shade_protocol::contract_interfaces::snip20::errors::transfer_disabled;
use shade_protocol::contract_interfaces::snip20::manager::{Allowance, Balance, CoinInfo, Config, ContractStatusLevel, ReceiverHash};
use shade_protocol::contract_interfaces::snip20::transaction_history::store_transfer;
use shade_protocol::utils::generic_response::ResponseStatus::Success;
use shade_protocol::utils::storage::plus::{ItemStorage, MapStorage};

pub fn try_transfer_impl(
    storage: &mut dyn Storage,
    sender: &Addr, //spender when using from
    owner: Option<&Addr>,
    recipient: &Addr,
    amount: Uint128,
    memo: Option<String>,
    denom: String,
    block: &shade_protocol::c_std::BlockInfo
) -> StdResult<()> {

    if !Config::transfer_enabled(storage)? {
        return Err(transfer_disabled())
    }

    let some_owner = match owner {
        None => sender,
        Some(owner) => {
            Allowance::spend(storage, owner, sender, amount, block)?;
            owner
        }
    };

    Balance::transfer(storage, amount, some_owner, recipient)?;

    store_transfer(
        storage,
        some_owner,
        sender,
        recipient,
        amount,
        denom,
        memo,
        block,
    )?;
    Ok(())
}

pub fn try_transfer(
    deps: DepsMut,
    env: Env,
    recipient: Addr,
    amount: Uint128,
    memo: Option<String>
) -> StdResult<Response> {
    let denom = CoinInfo::load(&deps.storage)?.symbol;
    try_transfer_impl(deps.storage, &info.sender, None, &recipient, amount, memo, denom, &env.block)?;
    Ok(Response{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Transfer { status: Success })?)
    })
}

pub fn try_batch_transfer(
    deps: DepsMut,
    env: Env,
    actions: Vec<batch::TransferAction>,
) -> StdResult<Response> {
    let sender = info.sender;
    let block = env.block;
    let denom = CoinInfo::load(&deps.storage)?.symbol;
    for action in actions {
        try_transfer_impl(deps.storage, &sender, None, &action.recipient, action.amount, action.memo, denom.clone(), &block)?;
    }
    Ok(Response{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BatchTransfer { status: Success })?)
    })
}

#[allow(clippy::too_many_arguments)]
fn try_add_receiver_api_callback(
    storage: &dyn Storage,
    messages: &mut Vec<CosmosMsg>,
    recipient: Addr,
    recipient_code_hash: Option<String>,
    msg: Option<Binary>,
    sender: Addr,
    from: Addr,
    amount: Uint128,
    memo: Option<String>,
) -> StdResult<()> {
    let receiver_hash = match recipient_code_hash {
        None => ReceiverHash::may_load(storage, recipient.clone())?,
        Some(hash) => Some(ReceiverHash(hash))
    };

    if let Some(hash) = receiver_hash {
        messages.push(
            ReceiverHandleMsg::new(sender, from, amount, memo, msg)
                .to_cosmos_msg(hash.0, recipient, None)?
        );
    }
    Ok(())
}

pub fn try_send_impl(
    storage: &mut dyn Storage,
    messages: &mut Vec<CosmosMsg>,
    sender: &Addr,
    owner: Option<&Addr>,
    recipient: &Addr,
    recipient_code_hash: Option<String>,
    amount: Uint128,
    memo: Option<String>,
    msg: Option<Binary>,
    denom: String,
    block: &shade_protocol::c_std::BlockInfo
) -> StdResult<()> {

    try_transfer_impl(storage, &sender, owner, &recipient, amount, memo.clone(), denom, block)?;
    try_add_receiver_api_callback(
        storage,
        messages,
        recipient.clone(),
        recipient_code_hash,
        msg,
        sender.clone(),
        sender.clone(),
        amount,
        memo,
    )?;

    Ok(())
}

pub fn try_send(
    deps: DepsMut,
    env: Env,
    recipient: Addr,
    recipient_code_hash: Option<String>,
    amount: Uint128,
    memo: Option<String>,
    msg: Option<Binary>
) -> StdResult<Response> {
    let mut messages = vec![];
    let denom = CoinInfo::load(&deps.storage)?.symbol;

    try_send_impl(
        deps.storage,
        &mut messages,
        &info.sender,
        None,
        &recipient,
        recipient_code_hash,
        amount,
        memo,
        msg,
        denom,
        &env.block
    )?;

    Ok(Response{
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Send { status: Success })?)
    })
}

pub fn try_batch_send(
    deps: DepsMut,
    env: Env,
    actions: Vec<batch::SendAction>
) -> StdResult<Response> {
    let mut messages = vec![];
    let sender = info.sender;
    let denom = CoinInfo::load(&deps.storage)?.symbol;

    for action in actions {
        try_send_impl(
            deps.storage,
            &mut messages,
            &sender,
            None,
            &action.recipient,
            action.recipient_code_hash,
            action.amount,
            action.memo,
            action.msg,
            denom.clone(),
            &env.block
        )?;
    }

    Ok(Response{
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BatchSend { status: Success })?)
    })
}