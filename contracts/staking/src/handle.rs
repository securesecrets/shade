use cosmwasm_std::{to_binary, Api, Binary, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage, CosmosMsg, HumanAddr, Uint128, WasmMsg};
use crate::state::{config_r, config_w, staker_w, unbonding_w, staker_r, total_staked_w};
use shade_protocol::{
    staking::{HandleMsg, HandleAnswer, QueryMsg, QueryAnswer},
    generic_response::ResponseStatus::{Success, Failure}};
use shade_protocol::staking::Unbonding;
use secret_toolkit::snip20::send_msg;
use shade_protocol::governance::{UserVote, VoteTally, Vote};
use secret_toolkit::utils::HandleCallback;
use shade_protocol::asset::Contract;


pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    admin: Option<Contract>,
    unbond_time: Option<u64>
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    // Check if admin
    if env.message.sender != config.admin.address {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    config_w(&mut deps.storage).update(|mut config| {
        if let Some(admin) = admin {
            config.admin = admin;
        }
        if let Some(unbond_time) = unbond_time {
            config.unbond_time = unbond_time;
        }
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

pub fn stake_weight(stake: Uint128, weight: u8) -> Uint128 {
    stake.multiply_ratio(100 as u128, weight)
}

pub fn try_vote<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    proposal_id: Uint128,
    votes: Vec<UserVote>,
) -> StdResult<HandleResponse> {

    let stake = staker_r(&deps.storage).load(env.message.sender.to_string().as_bytes())?;
    // check that percentage is <= 100 and calculate distribution
    let mut total_votes = VoteTally {
        yes: Uint128(0),
        no: Uint128(0),
        abstain: Uint128(0)
    };

    let mut count = 0;

    for vote in votes {
        match vote.vote {
            Vote::Yes => {
                total_votes.yes += stake_weight(stake.clone(), vote.weight);
            }
            Vote::No => {
                total_votes.no += stake_weight(stake.clone(), vote.weight);
            }
            Vote::Abstain => {
                total_votes.abstain += stake_weight(stake.clone(), vote.weight);
            }
        };
        count += vote.weight;
    }

    if count > 100 {
        return Err(StdError::GenericErr { msg: "Total weight must be 100 or less".to_string(), backtrace: None })
    }



    // Admin is governance, send to governance
    let config = config_r(&deps.storage).load()?;
    let messages = vec![shade_protocol::governance::HandleMsg::MakeVote {
        voter: env.message.sender.clone(),
        proposal_id,
        votes: total_votes,
    }.to_cosmos_msg(config.admin.code_hash,
                    config.admin.address, None)?];

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some( to_binary( &HandleAnswer::Vote {
            status: Success,
        })?),
    })
}

pub fn try_get_staker<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    account: HumanAddr
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    // Check if admin or account queried
    if !(env.message.sender == config.admin.address || env.message.sender == account) {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let stake = staker_r(&deps.storage).load(account.to_string().as_bytes())?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::GetStaker {
            status: Success,
            stake
        })?),
    })
}

pub fn try_get_stakers<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    stakers: Vec<HumanAddr>
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    // Check if admin
    if env.message.sender != config.admin.address {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let mut stake = vec![];

    for staker in stakers {
        stake.push(staker_r(&deps.storage).load(staker.to_string().as_bytes())?);
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::GetStakers {
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