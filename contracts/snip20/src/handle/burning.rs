use shade_protocol::c_std::{MessageInfo, Uint128};
use shade_protocol::c_std::{
    to_binary,
    Api,
    Env,
    DepsMut,
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

pub fn try_burn(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    memo: Option<String>,
) -> StdResult<Response> {
    let sender = &info.sender;
    let denom = CoinInfo::load(deps.storage)?.symbol;

    // Burn enabled
    if !Config::burn_enabled(deps.storage)? {
        return Err(burning_disabled());
    }

    Balance::sub(deps.storage, amount, sender)?;
    // Dec total supply
    TotalSupply::sub(deps.storage, amount)?;

    store_burn(
        deps.storage,
        &sender,
        &sender,
        amount,
        denom,
        memo,
        &env.block,
    )?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::Burn { status: Success })?))
}

pub fn try_burn_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    owner: Addr,
    amount: Uint128,
    memo: Option<String>,
) -> StdResult<Response> {
    let sender = &info.sender;
    let denom = CoinInfo::load(deps.storage)?.symbol;

    // Burn enabled
    if !Config::burn_enabled(deps.storage)? {
        return Err(burning_disabled());
    }

    Allowance::spend(deps.storage, &owner, &sender, amount, &env.block)?;
    Balance::sub(deps.storage, amount, &owner)?;
    // Dec total supply
    TotalSupply::sub(deps.storage, amount)?;

    store_burn(
        deps.storage,
        &owner,
        &sender,
        amount,
        denom,
        memo,
        &env.block,
    )?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::BurnFrom { status: Success })?))
}

pub fn try_batch_burn_from(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    actions: Vec<batch::BurnFromAction>,
) -> StdResult<Response> {
    let sender = &info.sender;
    let denom = CoinInfo::load(deps.storage)?.symbol;

    // Burn enabled
    if !Config::burn_enabled(deps.storage)? {
        return Err(burning_disabled());
    }

    let mut supply = TotalSupply::load(deps.storage)?;

    for action in actions {
        Allowance::spend(
            deps.storage,
            &action.owner,
            &sender,
            action.amount,
            &env.block,
        )?;

        Balance::sub(deps.storage, action.amount, &action.owner)?;

        // Dec total supply
        supply.0 = supply.0.checked_sub(action.amount)?;

        store_burn(
            deps.storage,
            &action.owner,
            &sender,
            action.amount,
            denom.clone(),
            action.memo,
            &env.block,
        )?;
    }

    supply.save(deps.storage)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::BatchBurnFrom { status: Success })?))
}
