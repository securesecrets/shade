use shade_protocol::{
    c_std::{
        to_binary,
        Addr,
        Binary,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdResult,
        Storage,
        SubMsg,
        Uint128,
    },
    contract_interfaces::snip20::{
        batch,
        errors::transfer_disabled,
        manager::{Allowance, Balance, CoinInfo, Config, ReceiverHash},
        transaction_history::store_transfer,
        ExecuteAnswer,
        ReceiverHandleMsg,
    },
    utils::{
        generic_response::ResponseStatus::Success,
        storage::plus::{ItemStorage, MapStorage},
        ExecuteCallback,
    },
    Contract,
};

pub fn try_transfer_impl(
    storage: &mut dyn Storage,
    sender: &Addr, //spender when using from
    owner: Option<&Addr>,
    recipient: &Addr,
    amount: Uint128,
    memo: Option<String>,
    denom: String,
    block: &shade_protocol::c_std::BlockInfo,
) -> StdResult<()> {
    if !Config::transfer_enabled(storage)? {
        return Err(transfer_disabled());
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
        storage, some_owner, sender, recipient, amount, denom, memo, block,
    )?;
    Ok(())
}

pub fn try_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Addr,
    amount: Uint128,
    memo: Option<String>,
) -> StdResult<Response> {
    let denom = CoinInfo::load(deps.storage)?.symbol;
    try_transfer_impl(
        deps.storage,
        &info.sender,
        None,
        &recipient,
        amount,
        memo,
        denom,
        &env.block,
    )?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Transfer { status: Success })?))
}

pub fn try_batch_transfer(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    actions: Vec<batch::TransferAction>,
) -> StdResult<Response> {
    let sender = info.sender;
    let block = env.block;
    let denom = CoinInfo::load(deps.storage)?.symbol;
    for action in actions {
        try_transfer_impl(
            deps.storage,
            &sender,
            None,
            &deps.api.addr_validate(action.recipient.as_str())?,
            action.amount,
            action.memo,
            denom.clone(),
            &block,
        )?;
    }
    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::BatchTransfer { status: Success })?))
}

#[allow(clippy::too_many_arguments)]
fn try_add_receiver_api_callback(
    storage: &dyn Storage,
    messages: &mut Vec<SubMsg>,
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
        Some(hash) => Some(ReceiverHash(hash)),
    };

    if let Some(hash) = receiver_hash {
        messages.push(SubMsg::new(
            ReceiverHandleMsg::new(sender.to_string(), from.to_string(), amount, memo, msg)
                .to_cosmos_msg(
                    &Contract {
                        address: recipient,
                        code_hash: hash.0,
                    },
                    vec![],
                )?,
        ));
    }
    Ok(())
}

pub fn try_send_impl(
    storage: &mut dyn Storage,
    messages: &mut Vec<SubMsg>,
    sender: &Addr,
    owner: Option<&Addr>,
    recipient: &Addr,
    recipient_code_hash: Option<String>,
    amount: Uint128,
    memo: Option<String>,
    msg: Option<Binary>,
    denom: String,
    block: &shade_protocol::c_std::BlockInfo,
) -> StdResult<()> {
    try_transfer_impl(
        storage,
        &sender,
        owner,
        &recipient,
        amount,
        memo.clone(),
        denom,
        block,
    )?;
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
    info: MessageInfo,
    recipient: Addr,
    recipient_code_hash: Option<String>,
    amount: Uint128,
    memo: Option<String>,
    msg: Option<Binary>,
) -> StdResult<Response> {
    let mut messages = vec![];
    let denom = CoinInfo::load(deps.storage)?.symbol;

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
        &env.block,
    )?;

    Ok(Response::new()
        .set_data(to_binary(&ExecuteAnswer::Send { status: Success })?)
        .add_submessages(messages))
}

pub fn try_batch_send(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    actions: Vec<batch::SendAction>,
) -> StdResult<Response> {
    let mut messages = vec![];
    let sender = info.sender;
    let denom = CoinInfo::load(deps.storage)?.symbol;

    for action in actions {
        try_send_impl(
            deps.storage,
            &mut messages,
            &sender,
            None,
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
        .set_data(to_binary(&ExecuteAnswer::BatchSend { status: Success })?)
        .add_submessages(messages))
}
