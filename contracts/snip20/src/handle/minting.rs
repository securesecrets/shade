use shade_protocol::c_std::{Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdResult, Storage, to_binary};
use shade_protocol::math_compat::Uint128;
use shade_protocol::contract_interfaces::snip20::{batch, HandleAnswer};
use shade_protocol::contract_interfaces::snip20::errors::{minting_disabled, not_admin, not_minter};
use shade_protocol::contract_interfaces::snip20::manager::{Admin, Balance, CoinInfo, Config, Minters, ReceiverHash, TotalSupply};
use shade_protocol::contract_interfaces::snip20::transaction_history::{store_burn, store_mint};
use shade_protocol::utils::generic_response::ResponseStatus::Success;
use shade_protocol::utils::storage::plus::{ItemStorage, MapStorage};

fn try_mint_impl<S: Storage>(
    storage: &mut S,
    minter: &HumanAddr,
    recipient: &HumanAddr,
    amount: Uint128,
    denom: String,
    memo: Option<String>,
    block: &shade_protocol::c_std::BlockInfo,
) -> StdResult<()> {
    Balance::add(storage, amount, recipient)?;
    store_mint(storage, minter, recipient, amount, denom, memo, block)?;
    Ok(())
}

pub fn try_mint<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    recipient: HumanAddr,
    amount: Uint128,
    memo: Option<String>,
) -> StdResult<HandleResponse> {
    // Mint enabled
    if !Config::mint_enabled(&deps.storage)? {
        return Err(minting_disabled())
    }
    // User is minter
    if !Minters::load(&deps.storage)?.0.contains(&env.message.sender) {
        return Err(not_minter(&env.message.sender))
    }
    // Inc total supply
    TotalSupply::add(&mut deps.storage, amount)?;
    let sender = env.message.sender;
    let block = env.block;
    let denom = CoinInfo::load(&deps.storage)?.symbol;
    try_mint_impl(&mut deps.storage, &sender, &recipient, amount, denom, memo, &block)?;

    Ok(HandleResponse{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Mint { status: Success })?)
    })
}

pub fn try_batch_mint<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    actions: Vec<batch::MintAction>,
) -> StdResult<HandleResponse> {
    // Mint enabled
    if !Config::mint_enabled(&deps.storage)? {
        return Err(minting_disabled())
    }
    // User is minter
    if !Minters::load(&deps.storage)?.0.contains(&env.message.sender) {
        return Err(not_minter(&env.message.sender))
    }

    let sender = env.message.sender;
    let block = env.block;
    let denom = CoinInfo::load(&deps.storage)?.symbol;
    let mut supply = TotalSupply::load(&deps.storage)?;
    for action in actions {
        supply.0.checked_add(action.amount)?;
        try_mint_impl(
            &mut deps.storage,
            &sender,
            &action.recipient,
            action.amount,
            denom.clone(),
            action.memo,
            &block
        )?;
    }
    supply.save(&mut deps.storage)?;

    Ok(HandleResponse{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BatchMint { status: Success })?)
    })
}

pub fn try_add_minters<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    new_minters: Vec<HumanAddr>
) -> StdResult<HandleResponse> {
    // Mint enabled
    if !Config::mint_enabled(&deps.storage)? {
        return Err(minting_disabled())
    }
    if Admin::load(&deps.storage)?.0 != env.message.sender {
        return Err(not_admin())
    }

    let mut minters = Minters::load(&deps.storage)?;
    minters.0.extend(new_minters);
    minters.save(&mut deps.storage)?;

    Ok(HandleResponse{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddMinters { status: Success })?)
    })
}

pub fn try_remove_minters<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    minters_to_remove: Vec<HumanAddr>
) -> StdResult<HandleResponse> {
    // Mint enabled
    if !Config::mint_enabled(&deps.storage)? {
        return Err(minting_disabled())
    }
    if Admin::load(&deps.storage)?.0 != env.message.sender {
        return Err(not_admin())
    }

    let mut minters = Minters::load(&deps.storage)?;
    for minter in minters_to_remove {
        minters.0.retain(|x| x != &minter);
    }
    minters.save(&mut deps.storage)?;

    Ok(HandleResponse{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RemoveMinters { status: Success })?)
    })
}

pub fn try_set_minters<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    minters: Vec<HumanAddr>
) -> StdResult<HandleResponse> {
    // Mint enabled
    if !Config::mint_enabled(&deps.storage)? {
        return Err(minting_disabled())
    }
    if Admin::load(&deps.storage)?.0 != env.message.sender {
        return Err(not_admin())
    }

    Minters(minters).save(&mut deps.storage)?;

    Ok(HandleResponse{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetMinters { status: Success })?)
    })
}