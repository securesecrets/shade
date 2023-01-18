use shade_protocol::{
    admin::helpers::{admin_is_valid, validate_admin, AdminPermissions},
    basic_staking::{Action, Config, ExecuteAnswer, RewardPool, Unbonding},
    c_std::{
        from_binary,
        to_binary,
        Addr,
        Binary,
        CosmosMsg,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
        Storage,
        Uint128,
    },
    snip20::helpers::{balance_query, register_receive, send_msg, set_viewing_key_msg},
    utils::{asset::Contract, generic_response::ResponseStatus},
};

use crate::storage::*;
use std::cmp::{max, min};

pub fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: Config,
) -> StdResult<Response> {
    let cur_config = CONFIG.load(deps.storage)?;

    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &cur_config.admin_auth,
    )?;

    // Save new info
    CONFIG.save(deps.storage, &config)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn register_reward(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token: Contract,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &config.admin_auth,
    )?;

    let reward_tokens = REWARD_TOKENS.load(deps.storage)?;

    if reward_tokens.contains(&token) {
        return Err(StdError::generic_err("Reward token already registered"));
    }

    Ok(Response::new()
        .add_messages(vec![
            set_viewing_key_msg(VIEWING_KEY.load(deps.storage)?, None, &token)?,
            register_receive(env.contract.code_hash, None, &token)?,
        ])
        .set_data(to_binary(&ExecuteAnswer::RegisterRewards {
            status: ResponseStatus::Success,
        })?))
}

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _sender: Addr,
    from: Addr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    let now = Uint128::new(env.block.time.seconds() as u128);

    match msg {
        Some(m) => match from_binary(&m)? {
            Action::Stake {} => {
                let stake_token = STAKE_TOKEN.load(deps.storage)?;
                if info.sender != stake_token.address {
                    return Err(StdError::generic_err(format!(
                        "Invalid Stake Token: {}",
                        info.sender
                    )));
                }

                let now = env.block.time.seconds();

                let mut total_staked = TOTAL_STAKED.load(deps.storage)?;

                let reward_pools = update_rewards(deps.storage, env.clone(), total_staked)?;

                let mut response = Response::new();

                if let Some(user_staked) = USER_STAKED.may_load(deps.storage, from.clone())? {
                    // Claim Rewards
                    for reward_pool in reward_pools {
                        let reward_claimed = reward_pool_claim(
                            deps.storage,
                            env.clone(),
                            from.clone(),
                            user_staked,
                            &reward_pool,
                        )?;
                        response = response.add_message(send_msg(
                            from.clone(),
                            reward_claimed,
                            None,
                            None,
                            None,
                            &reward_pool.token,
                        )?);
                    }
                    USER_STAKED.save(deps.storage, from.clone(), &(user_staked + amount))?;
                } else {
                    for reward_pool in reward_pools {
                        // make sure user rewards start now
                        USER_REWARD_PER_TOKEN_PAID.save(
                            deps.storage,
                            user_pool_key(from.clone(), reward_pool.uuid),
                            &reward_pool.reward_per_token,
                        )?;
                    }
                    USER_STAKED.save(deps.storage, from.clone(), &amount)?;
                }

                total_staked += amount;
                TOTAL_STAKED.save(deps.storage, &total_staked)?;

                USER_LAST_CLAIM.save(deps.storage, from, &Uint128::new(now as u128))?;

                Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Stake {
                    status: ResponseStatus::Success,
                })?))
            }
            Action::Rewards { start, end } => {
                let reward_tokens = REWARD_TOKENS.load(deps.storage)?;

                if let Some(token) = reward_tokens
                    .iter()
                    .find(|contract| contract.address == info.sender)
                {
                    let mut reward_pools = REWARD_POOLS.load(deps.storage)?;

                    let config = CONFIG.load(deps.storage)?;
                    let is_admin = admin_is_valid(
                        &deps.querier,
                        AdminPermissions::StakingAdmin,
                        from.to_string(),
                        &config.admin_auth,
                    )?;

                    // check user_pool limit
                    if !is_admin {
                        let user_pools_count = reward_pools
                            .iter()
                            .filter(|pool| !pool.admin_created)
                            .collect::<Vec<&RewardPool>>()
                            .len();
                        if Uint128::new(user_pools_count as u128) >= config.max_user_pools {
                            println!(
                                "user pools exceeded {} >= {}",
                                user_pools_count, config.max_user_pools
                            );
                            return Err(StdError::generic_err("Max user pools exceeded"));
                        }
                    }

                    // Disallow end before start
                    if start >= end {
                        return Err(StdError::generic_err("'start' must be after 'end'"));
                    }

                    // Disallow retro-active emissions (maybe could allow?)
                    if start < now {
                        return Err(StdError::generic_err("Cannot start emitting in the past"));
                    }

                    // Must emit at least 1 unit token per second (ddos protection)
                    /*
                    if amount < end - start {
                        return Err(StdError::generic_err(
                            "Cannot emit less than 1 unit token per second",
                        ));
                    }
                    */

                    let uuid = match reward_pools.last() {
                        Some(pool) => pool.uuid + Uint128::one(),
                        None => Uint128::zero(),
                    };

                    // Tokens per second emitted from this pool
                    let rate = amount * Uint128::new(10u128.pow(18)) / (end - start);

                    reward_pools.push(RewardPool {
                        uuid,
                        amount,
                        start,
                        end,
                        token: token.clone(),
                        rate,
                        reward_per_token: Uint128::zero(),
                        last_update: now,
                        creator: from,
                        admin_created: true,
                    });
                    REWARD_POOLS.save(deps.storage, &reward_pools)?;

                    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Rewards {
                        status: ResponseStatus::Success,
                    })?))
                } else {
                    return Err(StdError::generic_err(format!(
                        "Invalid Reward: {}",
                        info.sender
                    )));
                }
            }
        },
        None => {
            return Err(StdError::generic_err("No action provided"));
        }
    }
}
pub fn reward_per_token(total_staked: Uint128, now: u64, pool: &RewardPool) -> Uint128 {
    pool.reward_per_token
        + (min(pool.end, Uint128::new(now as u128)) - max(pool.last_update, pool.start)) * pool.rate
            / total_staked
}

pub fn rewards_earned(
    user_staked: Uint128,
    reward_per_token: Uint128,
    user_reward_per_token_paid: Uint128,
) -> Uint128 {
    println!(
        "rewards earned {} - {}",
        reward_per_token, user_reward_per_token_paid
    );
    user_staked * (reward_per_token - user_reward_per_token_paid) / Uint128::new(10u128.pow(18))
}

/*
 * Returns new reward_per_token
 */
pub fn updated_reward_pool(
    reward_pool: &RewardPool,
    total_staked: Uint128,
    now: u64,
) -> RewardPool {
    let mut pool = reward_pool.clone();
    pool.reward_per_token = reward_per_token(total_staked, now, &reward_pool);
    pool.last_update = min(reward_pool.end, Uint128::new(now as u128));
    pool
}

pub fn update_rewards(
    storage: &mut dyn Storage,
    env: Env,
    total_staked: Uint128,
) -> StdResult<Vec<RewardPool>> {
    let reward_pools = REWARD_POOLS.load(storage)?;
    let reward_pools = reward_pools
        .iter()
        .map(|pool| updated_reward_pool(pool, total_staked, env.block.time.seconds()))
        .collect();

    REWARD_POOLS.save(storage, &reward_pools)?;
    Ok(reward_pools)
}

pub fn reward_pool_claim(
    storage: &mut dyn Storage,
    env: Env,
    user: Addr,
    user_staked: Uint128,
    reward_pool: &RewardPool,
) -> StdResult<Uint128> {
    println!(
        "Reward Pool {} rewards {}",
        reward_pool.uuid, reward_pool.amount,
    );
    let user_reward_per_token_paid = USER_REWARD_PER_TOKEN_PAID
        .may_load(storage, user_pool_key(user.clone(), reward_pool.uuid))?
        .unwrap_or(Uint128::zero());
    println!("user reward per token paid {}", user_reward_per_token_paid);

    let user_reward = rewards_earned(
        user_staked,
        reward_pool.reward_per_token,
        user_reward_per_token_paid,
    );
    println!("reward earned {}", user_reward);

    // Send reward
    USER_REWARD_PER_TOKEN_PAID.save(
        storage,
        user_pool_key(user.clone(), reward_pool.uuid),
        &reward_pool.reward_per_token,
    )?;

    println!("Sending {} rewards to {}", user_reward, user.clone());

    Ok(user_reward)
    /*
    send_msg(
        user.clone(),
        user_reward,
        None,
        None,
        None,
        &reward_pool.token,
    )
        */
}

pub fn claim(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    // let user_last_claim = USER_LAST_CLAIM.load(deps.storage, info.sender.clone())?;
    let user_staked = USER_STAKED.load(deps.storage, info.sender.clone())?;
    let total_staked = TOTAL_STAKED.load(deps.storage)?;

    println!("Total Staked {}", total_staked);

    let now = env.block.time.seconds();
    /*
    let stake_token = STAKE_TOKEN.load(deps.storage)?;

    let stake_token_balance = balance_query(
        &deps.querier,
        env.contract.address.clone(),
        VIEWING_KEY.load(deps.storage)?,
        &stake_token,
    )?;
    println!("Contract Stake Token Balance {}", stake_token_balance);
    */

    let reward_pools = update_rewards(deps.storage, env.clone(), total_staked)?;

    let mut response = Response::new();

    for reward_pool in reward_pools.iter() {
        let reward_claimed = reward_pool_claim(
            deps.storage,
            env.clone(),
            info.sender.clone(),
            user_staked,
            reward_pool,
        )?;
        response = response.add_message(send_msg(
            info.sender.clone(),
            reward_claimed,
            None,
            None,
            None,
            &reward_pool.token,
        )?);
    }

    USER_LAST_CLAIM.save(deps.storage, info.sender, &Uint128::new(now.into()))?;

    Ok(response.set_data(to_binary(&ExecuteAnswer::Claim {
        status: ResponseStatus::Success,
    })?))
}

pub fn unbond(deps: DepsMut, env: Env, info: MessageInfo, amount: Uint128) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    println!("{} unbond {}", info.sender.clone(), amount);

    if amount.is_zero() {
        return Err(StdError::generic_err("Cannot unbond 0"));
    }

    if let Some(mut user_staked) = USER_STAKED.may_load(deps.storage, info.sender.clone())? {
        if user_staked < amount {
            return Err(StdError::generic_err(format!(
                "Cannot unbond {}, staked: {}",
                amount, user_staked
            )));
        }

        let now = env.block.time.seconds();

        let mut total_staked = TOTAL_STAKED.load(deps.storage)?;

        let reward_pools = update_rewards(deps.storage, env.clone(), total_staked)?;

        let mut response = Response::new();
        for reward_pool in reward_pools {
            let reward_claimed = reward_pool_claim(
                deps.storage,
                env.clone(),
                info.sender.clone(),
                user_staked,
                &reward_pool,
            )?;
            response = response.add_message(send_msg(
                info.sender.clone(),
                reward_claimed,
                None,
                None,
                None,
                &reward_pool.token,
            )?);
        }

        total_staked -= amount;
        TOTAL_STAKED.save(deps.storage, &total_staked)?;

        user_staked -= amount;
        USER_STAKED.save(deps.storage, info.sender.clone(), &user_staked)?;

        let mut user_unbondings = USER_UNBONDINGS
            .may_load(deps.storage, info.sender.clone())?
            .unwrap_or(vec![]);
        user_unbondings.push(Unbonding {
            amount,
            complete: Uint128::new(now as u128) + config.unbond_period,
        });

        USER_UNBONDINGS.save(deps.storage, info.sender, &user_unbondings)?;

        Ok(response.set_data(to_binary(&ExecuteAnswer::Unbond {
            status: ResponseStatus::Success,
        })?))
    } else {
        return Err(StdError::generic_err("User has no staked tokens"));
    }
}

pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let user_unbonding = USER_UNBONDINGS.load(deps.storage, info.sender.clone())?;

    let mut withdraw_amount = Uint128::zero();

    let mut remaining_unbondings = vec![];

    let now = Uint128::new(env.block.time.seconds() as u128);

    for unbonding in user_unbonding {
        if now >= unbonding.complete {
            withdraw_amount += unbonding.amount;
        } else {
            remaining_unbondings.push(unbonding);
        }
    }

    USER_UNBONDINGS.save(deps.storage, info.sender.clone(), &remaining_unbondings)?;

    if withdraw_amount.is_zero() {
        return Err(StdError::generic_err("No completed unbondings"));
    }

    let stake_token = STAKE_TOKEN.load(deps.storage)?;
    let stake_token_balance = balance_query(
        &deps.querier,
        env.contract.address.clone(),
        VIEWING_KEY.load(deps.storage)?,
        &stake_token,
    )?;
    println!("stake token balance {}", stake_token_balance);
    println!("Withdrawing {}", withdraw_amount);
    Ok(Response::new()
        .add_message(send_msg(
            info.sender,
            withdraw_amount,
            None,
            None,
            None,
            &STAKE_TOKEN.load(deps.storage)?,
        )?)
        .set_data(to_binary(&ExecuteAnswer::Withdraw {
            status: ResponseStatus::Success,
        })?))
}

pub fn compound(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    println!("Compounding rewards");
    let total_staked = TOTAL_STAKED.load(deps.storage)?;

    let reward_pools = update_rewards(deps.storage, env.clone(), total_staked)?;
    let user_staked = USER_STAKED
        .may_load(deps.storage, info.sender.clone())?
        .unwrap_or(Uint128::zero());
    let stake_token = STAKE_TOKEN.load(deps.storage)?;

    let mut compound_amount = Uint128::zero();

    let mut response = Response::new();
    for reward_pool in reward_pools {
        let reward_claimed = reward_pool_claim(
            deps.storage,
            env.clone(),
            info.sender.clone(),
            user_staked,
            &reward_pool,
        )?;
        if reward_pool.token == stake_token {
            println!(
                "Compounding {} {}",
                reward_claimed, reward_pool.token.address
            );
            compound_amount += reward_claimed;
        } else {
            println!("Claiming {} {}", reward_claimed, reward_pool.token.address);
            // Send/Claim non-stake_token rewards
            response = response.add_message(send_msg(
                info.sender.clone(),
                reward_claimed,
                None,
                None,
                None,
                &reward_pool.token,
            )?);
        }
    }

    USER_STAKED.save(
        deps.storage,
        info.sender.clone(),
        &(user_staked + compound_amount),
    )?;
    TOTAL_STAKED.save(deps.storage, &(total_staked + compound_amount))?;

    Ok(response.set_data(to_binary(&ExecuteAnswer::Compound {
        status: ResponseStatus::Success,
    })?))
}
