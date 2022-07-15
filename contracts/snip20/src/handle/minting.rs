use shade_protocol::c_std::{Api, Env, DepsMut, Response, Addr, Querier, StdError, StdResult, Storage, to_binary, MessageInfo};
use shade_protocol::c_std::Uint128;
use shade_protocol::contract_interfaces::snip20::{batch, HandleAnswer};
use shade_protocol::contract_interfaces::snip20::errors::{minting_disabled, not_admin, not_minter};
use shade_protocol::contract_interfaces::snip20::manager::{Admin, Balance, CoinInfo, Config, Minters, ReceiverHash, TotalSupply};
use shade_protocol::contract_interfaces::snip20::transaction_history::{store_burn, store_mint};
use shade_protocol::utils::generic_response::ResponseStatus::Success;
use shade_protocol::utils::storage::plus::{ItemStorage, MapStorage};

fn try_mint_impl(
    storage: &mut dyn Storage,
    minter: &Addr,
    recipient: &Addr,
    amount: Uint128,
    denom: String,
    memo: Option<String>,
    block: &shade_protocol::c_std::BlockInfo,
) -> StdResult<()> {
    Balance::add(storage, amount, recipient)?;
    store_mint(storage, minter, recipient, amount, denom, memo, block)?;
    Ok(())
}

pub fn try_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    recipient: Addr,
    amount: Uint128,
    memo: Option<String>,
) -> StdResult<Response> {
    // Mint enabled
    if !Config::mint_enabled(deps.storage)? {
        return Err(minting_disabled())
    }
    // User is minter
    if !Minters::load(deps.storage)?.0.contains(&info.sender) {
        return Err(not_minter(&info.sender))
    }
    // Inc total supply
    TotalSupply::add(deps.storage, amount)?;
    let sender = info.sender;
    let block = env.block;
    let denom = CoinInfo::load(deps.storage)?.symbol;
    try_mint_impl(deps.storage, &sender, &recipient, amount, denom, memo, &block)?;

    Ok(Response{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Mint { status: Success })?)
    })
}

pub fn try_batch_mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    actions: Vec<batch::MintAction>,
) -> StdResult<Response> {
    // Mint enabled
    if !Config::mint_enabled(deps.storage)? {
        return Err(minting_disabled())
    }
    // User is minter
    if !Minters::load(deps.storage)?.0.contains(&info.sender) {
        return Err(not_minter(&info.sender))
    }

    let sender = info.sender;
    let block = env.block;
    let denom = CoinInfo::load(deps.storage)?.symbol;
    let mut supply = TotalSupply::load(deps.storage)?;
    for action in actions {
        supply.0.checked_add(action.amount)?;
        try_mint_impl(
            deps.storage,
            &sender,
            &action.recipient,
            action.amount,
            denom.clone(),
            action.memo,
            &block
        )?;
    }
    supply.save(deps.storage)?;

    Ok(Response{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BatchMint { status: Success })?)
    })
}

pub fn try_add_minters(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_minters: Vec<Addr>
) -> StdResult<Response> {
    // Mint enabled
    if !Config::mint_enabled(deps.storage)? {
        return Err(minting_disabled())
    }
    if Admin::load(deps.storage)?.0 != info.sender {
        return Err(not_admin())
    }

    let mut minters = Minters::load(deps.storage)?;
    minters.0.extend(new_minters);
    minters.save(deps.storage)?;

    Ok(Response{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddMinters { status: Success })?)
    })
}

pub fn try_remove_minters(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    minters_to_remove: Vec<Addr>
) -> StdResult<Response> {
    // Mint enabled
    if !Config::mint_enabled(deps.storage)? {
        return Err(minting_disabled())
    }
    if Admin::load(deps.storage)?.0 != info.sender {
        return Err(not_admin())
    }

    let mut minters = Minters::load(deps.storage)?;
    for minter in minters_to_remove {
        minters.0.retain(|x| x != &minter);
    }
    minters.save(deps.storage)?;

    Ok(Response{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RemoveMinters { status: Success })?)
    })
}

pub fn try_set_minters(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    minters: Vec<Addr>
) -> StdResult<Response> {
    // Mint enabled
    if !Config::mint_enabled(deps.storage)? {
        return Err(minting_disabled())
    }
    if Admin::load(deps.storage)?.0 != info.sender {
        return Err(not_admin())
    }

    Minters(minters).save(deps.storage)?;

    Ok(Response{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetMinters { status: Success })?)
    })
}