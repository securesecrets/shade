use cosmwasm_std::{
    Api,
    BalanceResponse,
    BankQuery,
    Binary,
    Coin,
    CosmosMsg,
    debug_print,
    Env,
    Extern,
    HandleResponse,
    HumanAddr,
    Querier,
    StakingMsg,
    StdError,
    StdResult,
    Storage,
    to_binary,
    Uint128,
    Validator,
};

use secret_toolkit::snip20::{
    batch::SendFromAction,
    batch_send_from_msg,
    deposit_msg,
    redeem_msg,
    register_receive_msg,
    send_from_msg,
    set_viewing_key_msg,
};

use shade_protocol::{
    contract_interfaces::{
        snip20::{fetch_snip20, Snip20Asset},
        dao::{
            rewards_emission::{Config, HandleAnswer, Reward},
        },
    },
    utils::{
        asset::{Contract, scrt_balance},
        generic_response::ResponseStatus,
    },
};
use shade_protocol::contract_interfaces::dao::adapter;

use crate::{
    query,
    state::{asset_r, asset_w, assets_w, config_r, config_w, self_address_r, viewing_key_r},
};

pub fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _sender: HumanAddr,
    _from: HumanAddr,
    amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<HandleResponse> {
    //TODO: forward to distributor (quick fix mechanism)

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Receive {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: Config,
) -> StdResult<HandleResponse> {
    let cur_config = config_r(&deps.storage).load()?;

    if !cur_config.admins.contains(&env.message.sender) {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    config_w(&mut deps.storage).save(&config)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn register_asset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    contract: &Contract,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    if !config.admins.contains(&env.message.sender) {
        return Err(StdError::unauthorized());
    }

    assets_w(&mut deps.storage).update(|mut list| {
        if !list.contains(&contract.address) {
            list.push(contract.address.clone());
        }
        Ok(list)
    })?;

    asset_w(&mut deps.storage).save(
        contract.address.to_string().as_bytes(),
        &fetch_snip20(contract, &deps.querier)?,
    )?;

    Ok(HandleResponse {
        messages: vec![
            // Register contract in asset
            register_receive_msg(
                env.contract_code_hash.clone(),
                None,
                256,
                contract.code_hash.clone(),
                contract.address.clone(),
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

pub fn refill_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    rewards: Vec<Reward>,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    if env.message.sender != config.distributor {
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

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RefillRewards {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn update<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    asset: HumanAddr,
) -> StdResult<HandleResponse> {
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Update {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    asset: HumanAddr,
) -> StdResult<HandleResponse> {
    match asset_r(&deps.storage).may_load(&asset.as_str().as_bytes())? {
        Some(_) => Ok(HandleResponse {
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
