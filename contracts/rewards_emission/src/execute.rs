use shade_protocol::c_std::{
    to_binary,
    MessageInfo,
    Api,
    BalanceResponse,
    BankQuery,
    Binary,
    Coin,
    CosmosMsg,
    Env,
    DepsMut,
    Response,
    Addr,
    Querier,
    StakingMsg,
    StdError,
    StdResult,
    Storage,
    Uint128,
    Validator,
};

use shade_protocol::snip20::helpers::{
    deposit_msg,
    redeem_msg,
    register_receive,
    send_from_msg,
    set_viewing_key_msg,
};

use shade_protocol::{
    contract_interfaces::{
        dao::{
            adapter,
            rewards_emission::{Config, ExecuteAnswer, Reward},
        },
        snip20::helpers::{fetch_snip20, Snip20Asset},
    },
    utils::{
        asset::{scrt_balance, Contract},
        generic_response::ResponseStatus,
        cycle::{Cycle, exceeds_cycle, utc_now, parse_utc_datetime},
    },
};

use crate::{
    query,
    storage::*,
};

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _sender: Addr,
    _from: Addr,
    amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<Response> {
    //TODO: forward to distributor (quick fix mechanism)

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Receive {
        status: ResponseStatus::Success,
    })?))
}

pub fn try_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: Config,
) -> StdResult<Response> {
    let cur_config = CONFIG.load(deps.storage)?;

    if !cur_config.admins.contains(&info.sender) {
        return Err(StdError::generic_err("unauthorized"));
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::UpdateConfig {
        status: ResponseStatus::Success,
    })?))
}

pub fn refill_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {

    let config = CONFIG.load(deps.storage)?;
    let mut messages = vec![];

    if let Some(mut reward) = REWARD.may_load(deps.storage, info.sender.clone())? {

        let token = TOKEN.load(deps.storage)?;
        let now = utc_now(&env);

        // Check expiration
        if let Some(expiry) = reward.expiration.clone() {
            if now > parse_utc_datetime(&expiry)? {
                return Err(StdError::generic_err(format!("Rewards expired on {}", expiry)));
            }
        }

        if exceeds_cycle(&now, &parse_utc_datetime(&reward.last_refresh.clone())?, reward.cycle.clone()) {
            reward.last_refresh = now.to_rfc3339();
            REWARD.save(deps.storage, info.sender, &reward)?;
            // Send from treasury
            messages.push(send_from_msg(
                config.treasury.clone(),
                reward.distributor.address.clone(),
                reward.amount,
                None,
                None,
                None,
                &token.contract.clone(),
            )?);
        }
        else {
            return Err(StdError::generic_err(format!("Last rewards were requested on {}", reward.last_refresh)));
        }
    }
    else {
        return Err(StdError::generic_err("No rewards for you"));
    }

    Ok(Response::new()
       .add_messages(messages)
       .set_data(to_binary(&ExecuteAnswer::RefillRewards {
            status: ResponseStatus::Success,
        })?)
   )
}

pub fn register_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token: Addr,
    distributor: Contract,
    amount: Uint128,
    cycle: Cycle,
    expiration: Option<String>,
) -> StdResult<Response> {

    if token != TOKEN.load(deps.storage)?.contract.address {
        return Err(StdError::generic_err("Invalid token"));
    }

    REWARD.save(deps.storage, info.sender, &Reward {
        distributor,
        amount,
        cycle,
        //TODO change to null/zero for first refresh
        last_refresh: utc_now(&env).to_rfc3339(),
        expiration,
    })?;

    Ok(Response::new()
       .set_data(to_binary(&ExecuteAnswer::RegisterReward{
            status: ResponseStatus::Success,
        })?)
    )
}

/*
pub fn update(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: Addr,
) -> StdResult<Response> {
    Ok(Response::new().set_data(to_binary(&adapter::ExecuteAnswer::Update {
        status: ResponseStatus::Success,
    })?))
}

pub fn claim(
    deps: DepsMut,
    _env: Env,
    asset: Addr,
) -> StdResult<Response> {
    match asset_r(deps.storage).may_load(&asset.as_str().as_bytes())? {
        Some(_) => Ok(Response {
            messages: vec![],
            log: vec![],
            data: Some(to_binary(&adapter::ExecuteAnswer::Claim {
                status: ResponseStatus::Success,
                amount: Uint128::zero(),
            })?),
        }),
        None => Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        ))),
    }
}
*/
