use shade_protocol::{
    c_std::{to_binary, Addr, DepsMut, Env, MessageInfo, Response, StdResult, Uint128},
    contract_interfaces::snip20::{
        batch,
        errors::burning_disabled,
        manager::{Allowance, Balance, CoinInfo, Config, TotalSupply},
        transaction_history::store_burn,
        ExecuteAnswer,
    },
    utils::{generic_response::ResponseStatus::Success, storage::plus::ItemStorage},
};

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

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Burn { status: Success })?))
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

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::BurnFrom { status: Success })?))
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
            &deps.api.addr_validate(action.owner.as_str())?,
            &sender,
            action.amount,
            &env.block,
        )?;

        Balance::sub(
            deps.storage,
            action.amount,
            &deps.api.addr_validate(action.owner.as_str())?,
        )?;

        // Dec total supply
        supply.0 = supply.0.checked_sub(action.amount)?;

        store_burn(
            deps.storage,
            &deps.api.addr_validate(action.owner.as_str())?,
            &sender,
            action.amount,
            denom.clone(),
            action.memo,
            &env.block,
        )?;
    }

    supply.save(deps.storage)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::BatchBurnFrom { status: Success })?))
}
