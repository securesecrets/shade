use cosmwasm_std::{Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, to_binary};
use secret_toolkit::utils::HandleCallback;
use cosmwasm_math_compat::Uint128;
use shade_protocol::contract_interfaces::snip20_test::{batch, HandleAnswer, ReceiverHandleMsg};
use shade_protocol::contract_interfaces::snip20_test::manager::{Allowance, Balance, CoinInfo, Config, ReceiverHash};
use shade_protocol::contract_interfaces::snip20_test::transaction_history::store_transfer;
use shade_protocol::utils::generic_response::ResponseStatus::Success;
use shade_protocol::utils::storage::plus::{ItemStorage, MapStorage};

pub fn try_transfer_impl<S: Storage>(
    storage: &mut S,
    sender: &HumanAddr, //spender when using from
    owner: Option<&HumanAddr>,
    recipient: &HumanAddr,
    amount: Uint128,
    memo: Option<String>,
    denom: String,
    block: &cosmwasm_std::BlockInfo
) -> StdResult<()> {

    if !Config::transfer_enabled(storage)? {
        return Err(StdError::generic_err("Transfers are disabled"))
    }

    let some_owner = owner.unwrap_or(sender);

    if owner.is_some() {
        Allowance::spend(storage, some_owner, sender, amount, block)?;
    }

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

pub fn try_transfer<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    recipient: HumanAddr,
    amount: Uint128,
    memo: Option<String>
) -> StdResult<HandleResponse> {
    let denom = CoinInfo::load(&deps.storage)?.symbol;
    try_transfer_impl(&mut deps.storage, &env.message.sender, None, &recipient, amount, memo, denom, &env.block)?;
    Ok(HandleResponse{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Transfer { status: Success })?)
    })
}

pub fn try_batch_transfer<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    actions: Vec<batch::TransferAction>,
) -> StdResult<HandleResponse> {
    let sender = env.message.sender;
    let block = env.block;
    let denom = CoinInfo::load(&deps.storage)?.symbol;
    for action in actions {
        try_transfer_impl(&mut deps.storage, &sender, None, &action.recipient, action.amount, action.memo, denom.clone(), &block)?;
    }
    Ok(HandleResponse{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BatchTransfer { status: Success })?)
    })
}

#[allow(clippy::too_many_arguments)]
fn try_add_receiver_api_callback<S: Storage>(
    storage: &S,
    messages: &mut Vec<CosmosMsg>,
    recipient: HumanAddr,
    recipient_code_hash: Option<String>,
    msg: Option<Binary>,
    sender: HumanAddr,
    from: HumanAddr,
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

pub fn try_send_impl<S: Storage>(
    storage: &mut S,
    messages: &mut Vec<CosmosMsg>,
    sender: &HumanAddr,
    owner: Option<&HumanAddr>,
    recipient: &HumanAddr,
    recipient_code_hash: Option<String>,
    amount: Uint128,
    memo: Option<String>,
    msg: Option<Binary>,
    denom: String,
    block: &cosmwasm_std::BlockInfo
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

pub fn try_send<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    recipient: HumanAddr,
    recipient_code_hash: Option<String>,
    amount: Uint128,
    memo: Option<String>,
    msg: Option<Binary>
) -> StdResult<HandleResponse> {
    let mut messages = vec![];
    let denom = CoinInfo::load(&deps.storage)?.symbol;

    try_send_impl(
        &mut deps.storage,
        &mut messages,
        &env.message.sender,
        None,
        &recipient,
        recipient_code_hash,
        amount,
        memo,
        msg,
        denom,
        &env.block
    )?;

    Ok(HandleResponse{
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Send { status: Success })?)
    })
}

pub fn try_batch_send<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    actions: Vec<batch::SendAction>
) -> StdResult<HandleResponse> {
    let mut messages = vec![];
    let sender = env.message.sender;
    let denom = CoinInfo::load(&deps.storage)?.symbol;

    for action in actions {
        try_send_impl(
            &mut deps.storage,
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

    Ok(HandleResponse{
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BatchSend { status: Success })?)
    })
}