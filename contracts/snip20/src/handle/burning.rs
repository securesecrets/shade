use shade_protocol::c_std::Uint128;
use shade_protocol::c_std::{
    to_binary,
    Api,
    Env,
    Extern,
    Response,
    Addr,
    Querier,
    StdError,
    StdResult,
    Storage,
};
use shade_protocol::{
    contract_interfaces::snip20::{
        batch,
        manager::{Allowance, Balance, CoinInfo, Config, TotalSupply},
        transaction_history::store_burn,
        HandleAnswer,
    },
    utils::{generic_response::ResponseStatus::Success, storage::plus::ItemStorage},
};
use shade_protocol::contract_interfaces::snip20::errors::burning_disabled;

pub fn try_burn<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
    memo: Option<String>,
) -> StdResult<Response> {
    let sender = &env.message.sender;
    let denom = CoinInfo::load(&deps.storage)?.symbol;

    // Burn enabled
    if !Config::burn_enabled(&deps.storage)? {
        return Err(burning_disabled());
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

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Burn { status: Success })?),
    })
}

pub fn try_burn_from<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Addr,
    amount: Uint128,
    memo: Option<String>,
) -> StdResult<Response> {
    let sender = &env.message.sender;
    let denom = CoinInfo::load(&deps.storage)?.symbol;

    // Burn enabled
    if !Config::burn_enabled(&deps.storage)? {
        return Err(burning_disabled());
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

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BurnFrom { status: Success })?),
    })
}

pub fn try_batch_burn_from<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    actions: Vec<batch::BurnFromAction>,
) -> StdResult<Response> {
    let sender = &env.message.sender;
    let denom = CoinInfo::load(&deps.storage)?.symbol;

    // Burn enabled
    if !Config::burn_enabled(&deps.storage)? {
        return Err(burning_disabled());
    }

    let mut supply = TotalSupply::load(&deps.storage)?;

    for action in actions {
        Allowance::spend(
            &mut deps.storage,
            &action.owner,
            &sender,
            action.amount,
            &env.block,
        )?;

        Balance::sub(&mut deps.storage, action.amount, &action.owner)?;

        // Dec total supply
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

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BatchBurnFrom { status: Success })?),
    })
}
