use cosmwasm_std::{to_binary, Api, Binary, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage, CosmosMsg, HumanAddr, Uint128, WasmMsg};
use crate::state::{config_r, config_w, staker_w, unbonding_w, staker_r, total_staked_w};
use shade_protocol::{
    staking::{HandleMsg, HandleAnswer, QueryMsg, QueryAnswer},
    generic_response::ResponseStatus::{Success, Failure}};
use shade_protocol::staking::Unbonding;
use secret_toolkit::snip20::send_msg;


pub fn try_update_unbond_time<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    unbond_time: u64
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    // Check if admin
    if env.message.sender != config.admin {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    config_w(&mut deps.storage).update(|mut config| {
        config.unbond_time = unbond_time;
        Ok(config)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::UpdateUnbondTime {
            status: Success,
        })?),
    })
}

pub fn try_stake<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    sender: HumanAddr,
    from: HumanAddr,
    amount: Uint128,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    // Check if staking token
    if env.message.sender != config.staked_token.address {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Either create a new account or add stake
    staker_w(&mut deps.storage).update(sender.to_string().as_bytes(), |stake| {
        let total = match stake {
            None => amount,
            Some(stake) => stake + amount,
        };

        Ok(total)
    })?;

    // Update total stake
    total_staked_w(&mut deps.storage).update(|total| {
        Ok(total + amount)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::Stake {
            status: Success,
        })?),
    })
}

pub fn try_unbond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    amount: Uint128
) -> StdResult<HandleResponse> {

    let sender = env.message.sender.clone();

    // Check if user has >= amount
    staker_w(&mut deps.storage).update(sender.to_string().as_bytes(), |stake| {
        let total = match stake {
            None => return Err(StdError::GenericErr {
                msg: "Not enough staked".to_string(),
                backtrace: None }),
            Some(stake) => {
                if stake >= amount {
                    (stake - amount)
                } else {
                    return Err(StdError::GenericErr {
                        msg: "Not enough staked".to_string(),
                        backtrace: None })
                }
            }
        };

        total
    })?;

    let config = config_r(&deps.storage).load()?;
    unbonding_w(&mut deps.storage).update(|mut unbonding_queue| {
        unbonding_queue.push(Unbonding{
            account: sender,
            amount,
            unbond_time: env.block.time + config.unbond_time
        });

        Ok(unbonding_queue)
    })?;

    total_staked_w(&mut deps.storage).update(|total| { total - amount })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::Unbond {
            status: Success,
        })?),
    })
}

pub fn try_query_staker<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    account: HumanAddr
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    // Check if admin or account queried
    if !(env.message.sender == config.admin || env.message.sender == account) {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let stake = staker_r(&deps.storage).load(account.to_string().as_bytes())?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::QueryStaker {
            status: Success,
            stake
        })?),
    })
}

pub fn try_query_stakers<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    stakers: Vec<HumanAddr>
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    // Check if admin
    if env.message.sender != config.admin {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let mut stake = vec![];

    for staker in stakers {
        stake.push(staker_r(&deps.storage).load(staker.to_string().as_bytes())?);
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::QueryStakers {
            status: Success,
            stake
        })?),
    })
}

pub fn try_trigger_unbounds<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    let mut messages = vec![];

    unbonding_w(&mut deps.storage).update(|mut queue| {
        while !queue.is_empty() && env.block.time >= queue.peek().unwrap().unbond_time {
            let unbond = queue.pop().unwrap();
            messages.push(send_msg(unbond.account, unbond.amount, None, None, 1,
                     config.staked_token.code_hash.clone(),
                     config.staked_token.address.clone())?);
        }

        Ok(queue)
    })?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some( to_binary( &HandleAnswer::TriggerUnbonds {
            status: Success,
        })?),
    })
}