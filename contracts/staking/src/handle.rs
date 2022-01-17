use crate::state::{
    config_r,
    config_w,
    stake_state_r,
    stake_state_w,
    staker_r,
    staker_w,
    unbonding_w,
    user_unbonding_w,
    viewking_key_w,
};
use binary_heap_plus::BinaryHeap;
use cosmwasm_std::{
    to_binary,
    Api,
    Env,
    Extern,
    HandleResponse,
    HumanAddr,
    Querier,
    StdError,
    StdResult,
    Storage,
    Uint128,
};
use secret_toolkit::{snip20::send_msg, utils::HandleCallback};
use shade_protocol::{
    asset::Contract,
    generic_response::ResponseStatus::Success,
    governance::vote::{UserVote, Vote, VoteTally},
    staking::{
        stake::{Stake, Unbonding, UserStake},
        HandleAnswer,
    },
};

pub(crate) fn calculate_shares(tokens: Uint128, state: &Stake) -> Uint128 {
    if state.total_shares.is_zero() && state.total_tokens.is_zero() {
        tokens
    } else {
        tokens.multiply_ratio(state.total_shares, state.total_tokens)
    }
}

pub(crate) fn calculate_tokens(shares: Uint128, state: &Stake) -> Uint128 {
    if state.total_shares.is_zero() && state.total_tokens.is_zero() {
        shares
    } else {
        shares.multiply_ratio(state.total_tokens, state.total_shares)
    }
}

pub(crate) fn calculate_rewards(user: &UserStake, state: &Stake) -> Uint128 {
    (calculate_tokens(user.shares, state) - user.tokens_staked).unwrap()
}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    admin: Option<Contract>,
    unbond_time: Option<u64>,
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
        data: Some(to_binary(&HandleAnswer::UpdateUnbondTime {
            status: Success,
        })?),
    })
}

pub fn try_stake<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    sender: HumanAddr,
    _from: HumanAddr,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;
    // Check if staking token
    if env.message.sender != config.staked_token.address {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let mut state = stake_state_r(&deps.storage).load()?;

    // Either create a new account or add stake
    staker_w(&mut deps.storage).update(sender.as_str().as_bytes(), |user_state| {
        // Calculate shares proportional to stake amount
        let shares = calculate_shares(amount, &state);

        let new_state = match user_state {
            None => UserStake {
                shares,
                tokens_staked: amount,
            },
            Some(mut user_state) => {
                user_state.tokens_staked += amount;
                user_state.shares += shares;
                user_state
            }
        };

        state.total_shares += shares;
        state.total_tokens += amount;

        Ok(new_state)
    })?;

    // Update total stake
    stake_state_w(&mut deps.storage).save(&state)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Stake { status: Success })?),
    })
}

pub fn try_unbond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let sender = env.message.sender.clone();

    let mut state = stake_state_r(&deps.storage).load()?;

    // Check if user has >= amount
    staker_w(&mut deps.storage).update(sender.to_string().as_bytes(), |user_state| {
        let shares = calculate_shares(amount, &state);

        let new_state = match user_state {
            None => {
                return Err(StdError::GenericErr {
                    msg: "Not enough staked".to_string(),
                    backtrace: None,
                });
            }
            Some(user_state) => {
                if user_state.tokens_staked >= amount {
                    UserStake {
                        shares: (user_state.shares - shares)?,
                        tokens_staked: (user_state.tokens_staked - amount)?,
                    }
                } else {
                    return Err(StdError::GenericErr {
                        msg: "Not enough staked".to_string(),
                        backtrace: None,
                    });
                }
            }
        };

        // Theres no pretty way of doing this
        state.total_shares = (state.total_shares - shares)?;
        state.total_tokens = (state.total_tokens - amount)?;

        Ok(new_state)
    })?;

    let config = config_r(&deps.storage).load()?;
    let unbonding = Unbonding {
        amount,
        unbond_time: env.block.time + config.unbond_time,
    };

    unbonding_w(&mut deps.storage).update(|mut unbonding_queue| {
        unbonding_queue.push(unbonding.clone());
        Ok(unbonding_queue)
    })?;

    user_unbonding_w(&mut deps.storage).update(
        env.message.sender.to_string().as_bytes(),
        |queue| {
            let mut unbonding_queue = match queue {
                None => BinaryHeap::new_min(),
                Some(queue) => queue,
            };

            unbonding_queue.push(unbonding);

            Ok(unbonding_queue)
        },
    )?;

    stake_state_w(&mut deps.storage).save(&state)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Unbond { status: Success })?),
    })
}

pub fn stake_weight(stake: Uint128, weight: u8) -> Uint128 {
    stake.multiply_ratio(weight, 100u128)
}

pub fn try_vote<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    proposal_id: Uint128,
    votes: Vec<UserVote>,
) -> StdResult<HandleResponse> {
    let user_state = staker_r(&deps.storage).load(env.message.sender.to_string().as_bytes())?;
    // check that percentage is <= 100 and calculate distribution
    let mut total_votes = VoteTally {
        yes: Uint128(0),
        no: Uint128(0),
        abstain: Uint128(0),
    };

    let mut count = 0;

    for vote in votes {
        match vote.vote {
            Vote::Yes => {
                total_votes.yes += stake_weight(user_state.tokens_staked, vote.weight);
            }
            Vote::No => {
                total_votes.no += stake_weight(user_state.tokens_staked, vote.weight);
            }
            Vote::Abstain => {
                total_votes.abstain += stake_weight(user_state.tokens_staked, vote.weight);
            }
        };
        count += vote.weight;
    }

    if count > 100 {
        return Err(StdError::GenericErr {
            msg: "Total weight must be 100 or less".to_string(),
            backtrace: None,
        });
    }

    // Admin is governance, send to governance
    let config = config_r(&deps.storage).load()?;
    let messages = vec![
        shade_protocol::governance::HandleMsg::MakeVote {
            voter: env.message.sender.clone(),
            proposal_id,
            votes: total_votes,
        }
        .to_cosmos_msg(config.admin.code_hash, config.admin.address, None)?,
    ];

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Vote { status: Success })?),
    })
}

pub fn try_claim_unbond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    let mut total = Uint128::zero();

    let mut messages = vec![];

    user_unbonding_w(&mut deps.storage).update(
        env.message.sender.clone().to_string().as_bytes(),
        |queue| {
            let mut new_queue = queue.ok_or_else(|| StdError::not_found("user"))?;

            while let Some(unbonding) = new_queue.peek() {
                if env.block.time < unbonding.unbond_time {
                    break;
                }

                total += unbonding.amount;
                new_queue.pop();
            }

            messages.push(send_msg(
                env.message.sender.clone(),
                total,
                None,
                None,
                None,
                1,
                config.staked_token.code_hash.clone(),
                config.staked_token.address.clone(),
            )?);

            Ok(new_queue)
        },
    )?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ClaimUnbond { status: Success })?),
    })
}

pub fn try_claim_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    let mut state = stake_state_r(&deps.storage).load()?;
    let mut messages = vec![];

    staker_w(&mut deps.storage).update(
        env.message.sender.to_string().as_bytes(),
        |user_state| {
            let mut user = user_state.ok_or_else(|| StdError::NotFound {
                kind: "user".to_string(),
                backtrace: None,
            })?;

            let rewards = calculate_rewards(&user, &state);
            let shares = calculate_shares(rewards, &state);
            user.shares = (user.shares - shares)?;
            state.total_shares = (state.total_shares - shares)?;
            state.total_tokens = (state.total_tokens - rewards)?;

            messages.push(send_msg(
                env.message.sender.clone(),
                rewards,
                None,
                None,
                None,
                1,
                config.staked_token.code_hash.clone(),
                config.staked_token.address.clone(),
            )?);

            Ok(user)
        },
    )?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ClaimRewards { status: Success })?),
    })
}

pub fn try_set_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    key: String,
) -> StdResult<HandleResponse> {
    viewking_key_w(&mut deps.storage).save(env.message.sender.to_string().as_bytes(), &key)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetViewingKey { status: Success })?),
    })
}
