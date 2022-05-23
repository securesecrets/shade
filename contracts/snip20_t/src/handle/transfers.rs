use cosmwasm_std::{Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdResult, Storage, to_binary};
use cosmwasm_math_compat::Uint128;
use shade_protocol::contract_interfaces::snip20_test::{batch, HandleAnswer};
use shade_protocol::contract_interfaces::snip20_test::manager::{Balance, CoinInfo};
use shade_protocol::contract_interfaces::snip20_test::transaction_history::store_transfer;
use shade_protocol::utils::generic_response::ResponseStatus::Success;
use shade_protocol::utils::storage::plus::ItemStorage;

fn try_transfer_impl<S: Storage>(
    deps: &mut Extern<S, A, Q>,
    sender: &HumanAddr,
    recipient: &HumanAddr,
    amount: Uint128,
    memo: Option<String>,
    block: &cosmwasm_std::BlockInfo
) -> StdResult<()> {

    Balance::transfer(&mut deps.storage, amount, sender, recipient)?;

    store_transfer(
        &mut deps.storage,
        &sender,
        &sender,
        &recipient,
        amount,
        CoinInfo::load(&deps.storage)?.symbol,
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
    try_transfer_impl(deps, &env.message.sender, &recipient, amount, memo, &env.block)?;
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
    for action in actions {
        try_transfer_impl(deps, &sender, &action.recipient, action.amount, action.memo, &block)?;
    }
    Ok(HandleResponse{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BatchTransfer { status: Success })?)
    })
}