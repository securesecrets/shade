use cosmwasm_std::{Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, to_binary};
use cosmwasm_math_compat::Uint128;
use shade_protocol::contract_interfaces::snip20_test::{batch, HandleAnswer};
use shade_protocol::contract_interfaces::snip20_test::manager::{Allowance, Balance, CoinInfo, Config, TotalSupply};
use shade_protocol::contract_interfaces::snip20_test::transaction_history::store_burn;
use shade_protocol::utils::generic_response::ResponseStatus::Success;
use shade_protocol::utils::storage::plus::ItemStorage;

pub fn try_burn<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
    memo: Option<String>,
) -> StdResult<HandleResponse> {
    let sender = &env.message.sender;
    let denom = CoinInfo::load(&deps.storage)?.symbol;

    // Burn enabled
    if !Config::burn_enabled(&deps.storage)? {
        return Err(StdError::generic_err("Burning not enabled"))
    }

    Balance::sub(&mut deps.storage, amount, sender)?;
    // Dec total supply
    TotalSupply::sub(&mut deps.storage, amount)?;

    store_burn(
        &mut deps.storage,
        &sender,
        &sender,
        amount,
        denom,
        memo,
        &env.block,
    )?;

    Ok(HandleResponse{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Burn { status: Success })?)
    })
}

pub fn try_burn_from<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: HumanAddr,
    amount: Uint128,
    memo: Option<String>,
) -> StdResult<HandleResponse> {
    let sender = &env.message.sender;
    let denom = CoinInfo::load(&deps.storage)?.symbol;

    // Burn enabled
    if !Config::burn_enabled(&deps.storage)? {
        return Err(StdError::generic_err("Burning not enabled"))
    }

    Allowance::spend(&mut deps.storage, &owner, &sender, amount, &env.block)?;
    Balance::sub(&mut deps.storage, amount, &owner)?;
    // Dec total supply
    TotalSupply::sub(&mut deps.storage, amount)?;

    store_burn(
        &mut deps.storage,
        &owner,
        &sender,
        amount,
        denom,
        memo,
        &env.block,
    )?;

    Ok(HandleResponse{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BurnFrom { status: Success })?)
    })
}

pub fn try_batch_burn_from<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    actions: Vec<batch::BurnFromAction>,
) -> StdResult<HandleResponse> {
    let sender = &env.message.sender;
    let denom = CoinInfo::load(&deps.storage)?.symbol;

    // Burn enabled
    if !Config::burn_enabled(&deps.storage)? {
        return Err(StdError::generic_err("Burning not enabled"))
    }

    let mut supply = TotalSupply::load(&deps.storage)?;

    for action in actions {
        Allowance::spend(
            &mut deps.storage,
            &action.owner,
            &sender,
            action.amount,
            &env.block
        )?;

        Balance::sub(&mut deps.storage, action.amount, &action.owner)?;

        // Dec total supply
        // TODO: cannot burn more than total supply error
        supply.0 = supply.0.checked_sub(action.amount)?;

        store_burn(
            &mut deps.storage,
            &action.owner,
            &sender,
            action.amount,
            denom.clone(),
            action.memo,
            &env.block,
        )?;
    }

    supply.save(&mut deps.storage)?;

    Ok(HandleResponse{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BatchBurnFrom { status: Success })?)
    })
}