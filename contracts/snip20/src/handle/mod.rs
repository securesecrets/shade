pub mod allowance;
pub mod burning;
pub mod minting;
pub mod transfers;

use shade_protocol::c_std::Uint128;
use shade_protocol::c_std::{
    to_binary,
    Api,
    BankMsg,
    Coin,
    CosmosMsg,
    Env,
    DepsMut,
    Response,
    Addr,
    Querier,
    StdError,
    StdResult,
    Storage,
};
use shade_protocol::query_authentication::viewing_keys::ViewingKey;
use shade_protocol::{
    contract_interfaces::snip20::{
        batch,
        manager::{
            Admin,
            Balance,
            CoinInfo,
            Config,
            ContractStatusLevel,
            HashedKey,
            Key,
            Minters,
            PermitKey,
            RandSeed,
            ReceiverHash,
            TotalSupply,
        },
        transaction_history::{store_deposit, store_mint, store_redeem},
        HandleAnswer,
    },
    utils::{
        generic_response::ResponseStatus::Success,
        storage::plus::{ItemStorage, MapStorage},
    },
};
use shade_protocol::contract_interfaces::snip20::errors::{deposit_disabled, no_tokens_received, not_admin, not_enough_tokens, redeem_disabled, unsupported_token};

pub fn try_redeem<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    amount: Uint128,
) -> StdResult<Response> {
    let sender = info.sender;

    if !Config::redeem_enabled(&deps.storage)? {
        return Err(redeem_disabled());
    }

    Balance::sub(deps.storage, amount, &sender)?;
    TotalSupply::sub(deps.storage, amount)?;

    let token_reserve = Uint128::from(
        deps.querier
            .query_balance(&env.contract.address, "uscrt")?
            .amount,
    );
    if amount > token_reserve {
        return Err(not_enough_tokens(amount, token_reserve));
    }

    let withdrawal_coins: Vec<Coin> = vec![Coin {
        denom: "uscrt".to_string(),
        amount: amount.into(),
    }];

    let denom = CoinInfo::load(&deps.storage)?.symbol;

    store_redeem(deps.storage, &sender, amount, denom, &env.block)?;

    Ok(Response {
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
    deps: DepsMut,
    env: Env,
) -> StdResult<Response> {
    let sender = info.sender;
    let mut amount = Uint128::zero();
    for coin in &env.message.sent_funds {
        // TODO: implement IBC coins
        if coin.denom == "uscrt" {
            amount = Uint128::from(coin.amount)
        } else {
            return Err(unsupported_token());
        }
    }

    if amount.is_zero() {
        return Err(no_tokens_received());
    }

    if !Config::deposit_enabled(&deps.storage)? {
        return Err(deposit_disabled());
    }

    TotalSupply::add(deps.storage, amount)?;
    Balance::add(deps.storage, amount, &sender)?;

    store_deposit(
        deps.storage,
        &sender,
        amount,
        "uscrt".to_string(),
        &env.block,
    )?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Deposit { status: Success })?),
    })
}

pub fn try_change_admin<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    address: Addr,
) -> StdResult<Response> {
    if info.sender != Admin::load(&deps.storage)?.0 {
        return Err(not_admin());
    }

    Admin(address).save(deps.storage)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ChangeAdmin { status: Success })?),
    })
}

pub fn try_set_contract_status<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    status_level: ContractStatusLevel,
) -> StdResult<Response> {
    if info.sender != Admin::load(&deps.storage)?.0 {
        return Err(not_admin());
    }

    status_level.save(deps.storage)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetContractStatus {
            status: Success,
        })?),
    })
}

pub fn try_register_receive<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    code_hash: String,
) -> StdResult<Response> {
    ReceiverHash(code_hash).save(deps.storage, info.sender)?;
    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RegisterReceive {
            status: Success,
        })?),
    })
}

pub fn try_create_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    entropy: String,
) -> StdResult<Response> {
    let seed = RandSeed::load(&deps.storage)?.0;

    let key = Key::generate(&env, seed.as_slice(), (&entropy).as_ref());

    HashedKey(key.hash()).save(deps.storage, info.sender)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CreateViewingKey { key: key.0 })?),
    })
}

pub fn try_set_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    key: String,
) -> StdResult<Response> {
    let seed = RandSeed::load(&deps.storage)?.0;

    HashedKey(Key(key).hash()).save(deps.storage, info.sender)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetViewingKey { status: Success })?),
    })
}

pub fn try_revoke_permit<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    permit_name: String,
) -> StdResult<Response> {
    PermitKey::revoke(deps.storage, permit_name, info.sender)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RevokePermit { status: Success })?),
    })
}
