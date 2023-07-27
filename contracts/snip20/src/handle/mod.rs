pub mod allowance;
pub mod burning;
pub mod minting;
pub mod transfers;

use shade_protocol::{
    c_std::{
        to_binary,
        Addr,
        BankMsg,
        Coin,
        CosmosMsg,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdResult,
        Uint128,
    },
    contract_interfaces::snip20::{
        errors::{
            deposit_disabled,
            no_tokens_received,
            not_admin,
            not_enough_tokens,
            redeem_disabled,
            unsupported_token,
        },
        manager::{
            Admin,
            Balance,
            CoinInfo,
            Config,
            ContractStatusLevel,
            HashedKey,
            Key,
            PermitKey,
            RandSeed,
            ReceiverHash,
            TotalSupply,
        },
        transaction_history::{store_deposit, store_redeem},
        ExecuteAnswer,
    },
    query_authentication::viewing_keys::ViewingKey,
    snip20::manager::QueryAuth,
    utils::{
        generic_response::ResponseStatus::Success,
        storage::plus::{ItemStorage, MapStorage},
    },
    Contract,
};

pub fn try_redeem(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> StdResult<Response> {
    let sender = info.sender;

    if !Config::redeem_enabled(deps.storage)? {
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

    let denom = CoinInfo::load(deps.storage)?.symbol;

    store_redeem(deps.storage, &sender, amount, denom, &env.block)?;

    Ok(Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: sender.into(),
            amount: withdrawal_coins,
        }))
        .set_data(to_binary(&ExecuteAnswer::Redeem { status: Success })?))
}

pub fn try_deposit(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let sender = info.sender;
    let mut amount = Uint128::zero();
    for coin in &info.funds {
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

    if !Config::deposit_enabled(deps.storage)? {
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

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Deposit { status: Success })?))
}

pub fn try_change_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: Addr,
) -> StdResult<Response> {
    if info.sender != Admin::load(deps.storage)?.0 {
        return Err(not_admin());
    }

    Admin(address).save(deps.storage)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::ChangeAdmin { status: Success })?))
}

pub fn try_update_query_auth(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    auth: Option<Contract>,
) -> StdResult<Response> {
    if info.sender != Admin::load(deps.storage)?.0 {
        return Err(not_admin());
    }

    if let Some(auth) = auth {
        QueryAuth(auth).save(deps.storage)?;
    } else {
        QueryAuth::remove(deps.storage);
    }

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::UpdateQueryAuth {
            status: Success,
        })?),
    )
}

pub fn try_set_contract_status(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    status_level: ContractStatusLevel,
) -> StdResult<Response> {
    if info.sender != Admin::load(deps.storage)?.0 {
        return Err(not_admin());
    }

    status_level.save(deps.storage)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::SetContractStatus {
            status: Success,
        })?),
    )
}

pub fn try_register_receive(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    code_hash: String,
) -> StdResult<Response> {
    ReceiverHash(code_hash).save(deps.storage, info.sender)?;
    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::RegisterReceive {
            status: Success,
        })?),
    )
}

pub fn try_create_viewing_key(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    entropy: String,
) -> StdResult<Response> {
    let seed = RandSeed::load(deps.storage)?.0;

    let key = Key::generate(&info, &env, seed.as_slice(), (&entropy).as_ref());

    HashedKey(key.hash()).save(deps.storage, info.sender)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::CreateViewingKey { key: key.0 })?))
}

pub fn try_set_viewing_key(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    key: String,
) -> StdResult<Response> {
    // TODO: review this
    //let seed = RandSeed::load(deps.storage)?.0;

    HashedKey(Key(key).hash()).save(deps.storage, info.sender)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::SetViewingKey { status: Success })?))
}

pub fn try_revoke_permit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    permit_name: String,
) -> StdResult<Response> {
    PermitKey::revoke(deps.storage, permit_name, info.sender)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::RevokePermit { status: Success })?))
}
