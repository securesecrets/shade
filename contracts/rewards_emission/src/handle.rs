use shade_protocol::c_std::{
    debug_print,
    to_binary,
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
    batch::SendFromAction,
    batch_send_from_msg,
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
            rewards_emission::{Config, HandleAnswer, Reward},
        },
        snip20::helpers::{fetch_snip20, Snip20Asset},
    },
    utils::{
        asset::{scrt_balance, Contract},
        generic_response::ResponseStatus,
    },
};

use crate::{
    query,
    state::{asset_r, asset_w, assets_w, config_r, config_w, self_address_r, viewing_key_r},
};

pub fn receive(
    deps: DepsMut,
    env: Env,
    _sender: Addr,
    _from: Addr,
    amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<Response> {
    //TODO: forward to distributor (quick fix mechanism)

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Receive {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_update_config(
    deps: DepsMut,
    env: Env,
    config: Config,
) -> StdResult<Response> {
    let cur_config = config_r(&deps.storage).load()?;

    if !cur_config.admins.contains(&info.sender) {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    config_w(deps.storage).save(&config)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn register_asset(
    deps: DepsMut,
    env: Env,
    contract: &Contract,
) -> StdResult<Response> {
    let config = config_r(&deps.storage).load()?;

    if !config.admins.contains(&info.sender) {
        return Err(StdError::unauthorized());
    }

    assets_w(deps.storage).update(|mut list| {
        if !list.contains(&contract.address) {
            list.push(contract.address.clone());
        }
        Ok(list)
    })?;

    asset_w(deps.storage).save(
        contract.address.to_string().as_bytes(),
        &fetch_snip20(contract, &deps.querier)?,
    )?;

    Ok(Response {
        messages: vec![
            // Register contract in asset
            register_receive(
                env.contract_code_hash.clone(),
                None,
                contract
            )?,
            // Set viewing key
            set_viewing_key_msg(
                viewing_key_r(&deps.storage).load()?,
                None,
                256,
                contract.code_hash.clone(),
                contract.address.clone(),
            )?,
        ],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RegisterAsset {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn refill_rewards(
    deps: DepsMut,
    env: Env,
    rewards: Vec<Reward>,
) -> StdResult<Response> {
    let config = config_r(&deps.storage).load()?;

    if info.sender != config.distributor {
        return Err(StdError::unauthorized());
    }

    let mut messages = vec![];

    for reward in rewards {
        let full_asset = match asset_r(&deps.storage).may_load(&reward.asset.as_str().as_bytes())? {
            Some(a) => a,
            None => {
                return Err(StdError::generic_err(format!(
                    "Unrecognized Asset {}",
                    reward.asset
                )));
            }
        };

        messages.push(send_from_msg(
            config.treasury.clone(),
            config.distributor.clone(),
            reward.amount,
            None,
            None,
            None,
            1,
            full_asset.contract.code_hash.clone(),
            full_asset.contract.address.clone(),
        )?);
    }

    Ok(Response {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RefillRewards {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn update(
    deps: DepsMut,
    env: Env,
    asset: Addr,
) -> StdResult<Response> {
    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Update {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn claim(
    deps: DepsMut,
    _env: Env,
    asset: Addr,
) -> StdResult<Response> {
    match asset_r(&deps.storage).may_load(&asset.as_str().as_bytes())? {
        Some(_) => Ok(Response {
            messages: vec![],
            log: vec![],
            data: Some(to_binary(&adapter::HandleAnswer::Claim {
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
