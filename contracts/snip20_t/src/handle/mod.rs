pub mod allowance;
pub mod transfers;
pub mod minting;
pub mod burning;

use cosmwasm_std::{Api, BankMsg, Coin, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, to_binary};
use query_authentication::viewing_keys::ViewingKey;
use cosmwasm_math_compat::Uint128;
use shade_protocol::contract_interfaces::snip20_test::{batch, HandleAnswer};
use shade_protocol::contract_interfaces::snip20_test::manager::{Admin, Balance, CoinInfo, Config, ContractStatusLevel, HashedKey, Key, Minters, PermitKey, RandSeed, ReceiverHash, TotalSupply};
use shade_protocol::contract_interfaces::snip20_test::transaction_history::{store_deposit, store_mint, store_redeem};
use shade_protocol::utils::generic_response::ResponseStatus::Success;
use shade_protocol::utils::storage::plus::{ItemStorage, MapStorage};

pub fn try_redeem<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let sender = env.message.sender;

    if !Config::redeem_enabled(&deps.storage)? {
        return Err(StdError::generic_err(
            "Redeem functionality is not enabled for this token.",
        ));
    }

    Balance::sub(&mut deps.storage, amount, &sender)?;
    TotalSupply::sub(&mut deps.storage, amount)?;

    let token_reserve = deps
        .querier
        .query_balance(&env.contract.address, "uscrt")?
        .amount;
    if amount > token_reserve {
        return Err(StdError::generic_err(
            "You are trying to redeem for more SCRT than the token has in its deposit reserve.",
        ));
    }

    let withdrawal_coins: Vec<Coin> = vec![Coin {
        denom: "uscrt".to_string(),
        amount: amount.into(),
    }];

    store_redeem(
        &mut deps.storage,
        &sender,
        amount,
        CoinInfo::load(&deps.storage)?.symbol,
        &env.block,
    )?;

    Ok(HandleResponse {
        messages: vec![CosmosMsg::Bank(BankMsg::Send {
            from_address: env.contract.address,
            to_address: sender,
            amount: withdrawal_coins,
        })],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Redeem { status: Success })?),
    })
}

pub fn try_deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let sender = env.message.sender;
    let mut amount = Uint128::zero();
    for coin in &env.message.sent_funds {
        // TODO: implement IBC coins
        if coin.denom == "uscrt" {
            amount = Uint128::from(coin.amount)
        } else {
            return Err(StdError::generic_err(
                "Tried to deposit an unsupported token",
            ));
        }
    }

    if amount.is_zero() {
        return Err(StdError::generic_err("No funds were sent to be deposited"));
    }

    if !Config::deposit_enabled(&deps.storage)? {
        return Err(StdError::generic_err(
            "Deposit functionality is not enabled for this token.",
        ));
    }

    TotalSupply::add(&mut deps.storage, amount)?;
    Balance::add(&mut deps.storage, amount, &sender)?;

    store_deposit(
        &mut deps.storage,
        &sender,
        amount,
        "uscrt".to_string(),
        &env.block,
    )?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Deposit { status: Success })?),
    })
}

pub fn try_change_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    address: HumanAddr
) -> StdResult<HandleResponse> {
    if env.message.sender != Admin::load(&deps.storage)?.0 {
        return Err(StdError::unauthorized())
    }

    Admin(address).save(&mut deps.storage)?;

    Ok(HandleResponse{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ChangeAdmin { status: Success })?)
    })
}

pub fn try_set_contract_status<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    status_level: ContractStatusLevel
) -> StdResult<HandleResponse> {
    if env.message.sender != Admin::load(&deps.storage)?.0 {
        return Err(StdError::unauthorized())
    }

    status_level.save(&mut deps.storage)?;

    Ok(HandleResponse{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetContractStatus { status: Success })?)
    })
}

pub fn try_register_receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    code_hash: String
) -> StdResult<HandleResponse> {
    ReceiverHash(code_hash).save(&mut deps.storage, env.message.sender)?;
    Ok(HandleResponse{
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RegisterReceive { status: Success })?)
    })
}

pub fn try_create_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    entropy: String,
) -> StdResult<HandleResponse> {
    let seed = RandSeed::load(&deps.storage)?.0;

    let key = Key::generate(&env, seed.as_slice(), (&entropy).as_ref());

    HashedKey(key.hash()).save(&mut deps.storage, env.message.sender)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CreateViewingKey { key: key.0 })?),
    })
}

pub fn try_set_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    entropy: String,
) -> StdResult<HandleResponse> {
    let seed = RandSeed::load(&deps.storage)?.0;

    let key = Key::generate(&env, seed.as_slice(), (&entropy).as_ref());

    HashedKey(key.hash()).save(&mut deps.storage, env.message.sender)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CreateViewingKey { key: key.0 })?),
    })
}

pub fn try_revoke_permit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    permit_name: String,
) -> StdResult<HandleResponse> {

    PermitKey::revoke(&mut deps.storage, permit_name, env.message.sender)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RevokePermit { status: Success })?),
    })
}