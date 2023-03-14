use shade_protocol::{
    admin::helpers::{admin_is_valid, validate_admin, AdminPermissions},
    basic_staking::{Action, ExecuteAnswer, RewardPoolInternal, Unbonding},
    c_std::{
        from_binary,
        to_binary,
        Addr,
        Binary,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
        Storage,
        Uint128,
    },
    snip20::helpers::{register_receive, send_msg, set_viewing_key_msg},
    utils::{
        asset::{Contract, RawContract},
        generic_response::ResponseStatus,
    },
};

use crate::storage::*;
use std::cmp::{max, min};

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin_auth: Option<RawContract>,
    query_auth: Option<RawContract>,
    unbond_period: Option<Uint128>,
    max_user_pools: Option<Uint128>,
    reward_cancel_threshold: Option<Uint128>,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;

    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &config.admin_auth,
    )?;

    if let Some(admin_auth) = admin_auth {
        config.admin_auth = admin_auth.into_valid(deps.api)?;
    }

    if let Some(query_auth) = query_auth {
        config.query_auth = query_auth.into_valid(deps.api)?;
    }

    if let Some(unbond_period) = unbond_period {
        config.unbond_period = unbond_period;
    }

    if let Some(max_user_pools) = max_user_pools {
        config.max_user_pools = max_user_pools;
    }

    if let Some(reward_cancel_threshold) = reward_cancel_threshold {
        config.reward_cancel_threshold = reward_cancel_threshold;
    }

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
    let now = Uint128::new(env.block.time.seconds() as u128);

    match msg {
        Some(m) => match from_binary(&m)? {
            Action::Stake { compound } => {
                let stake_token = STAKE_TOKEN.load(deps.storage)?;
                if info.sender != stake_token.address {
                    return Err(StdError::generic_err(format!(
                        "Invalid Stake Token: {}",
                        info.sender
                    )));
                }

                let compound = compound.unwrap_or(false);

                let now = env.block.time.seconds();

                let total_staked = TOTAL_STAKED.load(deps.storage)?;

                let mut reward_pools =
                    update_rewards(env.clone(), &REWARD_POOLS.load(deps.storage)?, total_staked);

                let mut response = Response::new();

                let mut compound_amount = Uint128::zero();

                let user_staked = USER_STAKED
                    .may_load(deps.storage, from.clone())?
                    .unwrap_or(Uint128::zero());

                if !user_staked.is_zero() {
                    // Claim Rewards
                    for reward_pool in reward_pools.iter_mut() {
                        let reward_claimed = reward_pool_claim(
                            deps.storage,
                            from.clone(),
                            user_staked,
                            &reward_pool,
                        )?;
                        reward_pool.claimed += reward_claimed;
                        if compound && reward_pool.token == stake_token {
                            // Compound stake_token rewards
                            compound_amount += reward_claimed;
                        } else {
                            // Claim if not compound or not stake token rewards
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
                } else {
                    for reward_pool in reward_pools.iter() {
                        // make sure user rewards start now
                        USER_REWARD_PER_TOKEN_PAID.save(
                            deps.storage,
                            user_pool_key(from.clone(), reward_pool.id),
                            &reward_pool.reward_per_token,
                        )?;
                    }
                }
                USER_STAKED.save(deps.storage, from.clone(), &(user_staked + amount))?;

                REWARD_POOLS.save(deps.storage, &reward_pools.clone())?;
                TOTAL_STAKED.save(deps.storage, &(total_staked + amount))?;
                USER_LAST_CLAIM.save(deps.storage, from, &Uint128::new(now as u128))?;

                Ok(response.set_data(to_binary(&ExecuteAnswer::Stake {
                    status: ResponseStatus::Success,
                })?))
            }
            Action::Rewards { start, end } => {
                let reward_tokens = REWARD_TOKENS.load(deps.storage)?;

                if let Some(token) = reward_tokens
                    .iter()
                    .find(|contract| contract.address == info.sender)
                {
                    // Disallow end before start
                    if start >= end {
                        return Err(StdError::generic_err("'start' must be after 'end'"));
                    }

                    // Disallow retro-active emissions (maybe could allow?)
                    if start < now {
                        return Err(StdError::generic_err("Cannot start emitting in the past"));
                    }

                    let mut reward_pools = REWARD_POOLS.load(deps.storage)?;

                    let config = CONFIG.load(deps.storage)?;
                    let is_admin = match admin_is_valid(
                        &deps.querier,
                        AdminPermissions::StakingAdmin,
                        from.to_string(),
                        &config.admin_auth,
                    ) {
                        Ok(_) => true,
                        Err(_) => false,
                    };

                    // check user_pool limit
                    if !is_admin {
                        let user_pools_count = reward_pools
                            .iter()
                            .filter(|pool| !pool.official)
                            .collect::<Vec<&RewardPoolInternal>>()
                            .len();
                        if user_pools_count as u128 >= config.max_user_pools.u128() {
                            return Err(StdError::generic_err("Max user pools exceeded"));
                        }
                    }

                    let new_id = match reward_pools.last() {
                        Some(pool) => pool.id + Uint128::one(),
                        None => Uint128::zero(),
                    };

                    // Tokens per second emitted from this pool
                    let rate = amount * Uint128::new(10u128.pow(18)) / (end - start);

                    reward_pools.push(RewardPoolInternal {
                        id: new_id,
                        amount,
                        start,
                        end,
                        token: token.clone(),
                        rate,
                        reward_per_token: Uint128::zero(),
                        claimed: Uint128::zero(),
                        last_update: now,
                        creator: from,
                        official: is_admin,
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

pub fn reward_per_token(total_staked: Uint128, now: u64, pool: &RewardPoolInternal) -> Uint128 {
    if total_staked.is_zero() {
        return Uint128::zero();
    }

    let start = max(pool.last_update, pool.start);
    let end = min(pool.end, Uint128::new(now as u128));

    if start > end {
        return pool.reward_per_token;
    }

    pool.reward_per_token + (((end - start) * pool.rate) / total_staked)
}

pub fn rewards_earned(
    user_staked: Uint128,
    reward_per_token: Uint128,
    user_reward_per_token_paid: Uint128,
) -> Uint128 {
    user_staked * (reward_per_token - user_reward_per_token_paid) / Uint128::new(10u128.pow(18))
}

/*
 * Returns new reward_per_token
 */
pub fn updated_reward_pool(
    reward_pool: &RewardPoolInternal,
    total_staked: Uint128,
    now: u64,
) -> RewardPoolInternal {
    let mut pool = reward_pool.clone();
    pool.reward_per_token = reward_per_token(total_staked, now, &reward_pool);
    pool.last_update = min(reward_pool.end, Uint128::new(now as u128));
    pool
}

pub fn update_rewards(
    env: Env,
    reward_pools: &Vec<RewardPoolInternal>,
    total_staked: Uint128,
) -> Vec<RewardPoolInternal> {
    reward_pools
        .iter()
        .map(|pool| updated_reward_pool(pool, total_staked, env.block.time.seconds()))
        .collect()
}

/* returns the earned rewards
 * Reward must be sent buy calling code
 */
pub fn reward_pool_claim(
    storage: &mut dyn Storage,
    user: Addr,
    user_staked: Uint128,
    reward_pool: &RewardPoolInternal,
) -> StdResult<Uint128> {
    let user_reward_per_token_paid = USER_REWARD_PER_TOKEN_PAID
        .may_load(storage, user_pool_key(user.clone(), reward_pool.id))?
        .unwrap_or(Uint128::zero());

    let user_reward = rewards_earned(
        user_staked,
        reward_pool.reward_per_token,
        user_reward_per_token_paid,
    );

    USER_REWARD_PER_TOKEN_PAID.save(
        storage,
        user_pool_key(user.clone(), reward_pool.id),
        &reward_pool.reward_per_token,
    )?;

    Ok(user_reward)
}

pub fn claim(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let user_staked = USER_STAKED.load(deps.storage, info.sender.clone())?;

    if user_staked.is_zero() {
        return Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Claim {
            status: ResponseStatus::Success,
        })?));
    }

    let total_staked = TOTAL_STAKED.load(deps.storage)?;

    let now = env.block.time.seconds();

    let mut reward_pools =
        update_rewards(env.clone(), &REWARD_POOLS.load(deps.storage)?, total_staked);

    let mut response = Response::new();

    for reward_pool in reward_pools.iter_mut() {
        let reward_claimed =
            reward_pool_claim(deps.storage, info.sender.clone(), user_staked, &reward_pool)?;

        reward_pool.claimed += reward_claimed;

        response = response.add_message(send_msg(
            info.sender.clone(),
            reward_claimed,
            None,
            None,
            None,
            &reward_pool.token,
        )?);
    }

    REWARD_POOLS.save(deps.storage, &reward_pools)?;
    USER_LAST_CLAIM.save(deps.storage, info.sender, &Uint128::new(now.into()))?;

    Ok(response.set_data(to_binary(&ExecuteAnswer::Claim {
        status: ResponseStatus::Success,
    })?))
}

pub fn unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    compound: bool,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    if amount.is_zero() {
        return Err(StdError::generic_err("Cannot unbond 0"));
    }

    if let Some(mut user_staked) = USER_STAKED.may_load(deps.storage, info.sender.clone())? {
        if user_staked.is_zero() || user_staked < amount {
            return Err(StdError::generic_err(format!(
                "Cannot unbond {}, only {} staked",
                amount, user_staked
            )));
        }

        let now = env.block.time.seconds();

        let mut total_staked = TOTAL_STAKED.load(deps.storage)?;

        let mut reward_pools =
            update_rewards(env.clone(), &REWARD_POOLS.load(deps.storage)?, total_staked);

        let stake_token = STAKE_TOKEN.load(deps.storage)?;
        let mut compound_amount = Uint128::zero();

        let mut response = Response::new();

        // Claim/Compound rewards
        for reward_pool in reward_pools.iter_mut() {
            let reward_claimed =
                reward_pool_claim(deps.storage, info.sender.clone(), user_staked, &reward_pool)?;
            reward_pool.claimed += reward_claimed;

            if compound && reward_pool.token == stake_token {
                // Compound stake_token rewards
                compound_amount += reward_claimed;
            } else {
                // Claim if not compound or not stake token rewards
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

        user_staked = (user_staked + compound_amount) - amount;
        total_staked = (total_staked + compound_amount) - amount;

        TOTAL_STAKED.save(deps.storage, &total_staked)?;
        USER_STAKED.save(deps.storage, info.sender.clone(), &user_staked)?;
        REWARD_POOLS.save(deps.storage, &reward_pools)?;

        let mut user_unbonding_ids = USER_UNBONDING_IDS
            .may_load(deps.storage, info.sender.clone())?
            .unwrap_or(vec![]);

        let next_id = *user_unbonding_ids.iter().max().unwrap_or(&Uint128::zero()) + Uint128::one();

        user_unbonding_ids.push(next_id);
        USER_UNBONDING_IDS.save(deps.storage, info.sender.clone(), &user_unbonding_ids)?;

        USER_UNBONDING.save(
            deps.storage,
            user_unbonding_key(info.sender, next_id),
            &Unbonding {
                id: next_id,
                amount,
                complete: Uint128::new(now as u128) + config.unbond_period,
            },
        )?;

        Ok(response.set_data(to_binary(&ExecuteAnswer::Unbond {
            id: next_id,
            status: ResponseStatus::Success,
        })?))
    } else {
        return Err(StdError::generic_err("User is not a staker"));
    }
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    ids: Option<Vec<Uint128>>,
) -> StdResult<Response> {
    let mut user_unbonding_ids = USER_UNBONDING_IDS
        .may_load(deps.storage, info.sender.clone())?
        .unwrap_or(vec![]);

    // If null ids, use all user unbondings
    let ids = ids.unwrap_or(user_unbonding_ids.clone());

    let mut withdraw_amount = Uint128::zero();

    let now = Uint128::new(env.block.time.seconds() as u128);

    let mut withdrawn_ids = vec![];

    for id in ids.into_iter() {
        if let Some(unbonding) =
            USER_UNBONDING.may_load(deps.storage, user_unbonding_key(info.sender.clone(), id))?
        {
            if now >= unbonding.complete {
                withdraw_amount += unbonding.amount;
                withdrawn_ids.push(id);
            }
        } else {
            return Err(StdError::generic_err(format!("Bad ID {}", id)));
        }
    }

    if withdraw_amount.is_zero() {
        return Err(StdError::generic_err("No unbondings to withdraw"));
    }

    // Sort lists so the operation is O(n)
    withdrawn_ids.sort();
    user_unbonding_ids.sort();

    // To store remaining ids
    let mut new_unbonding_ids = vec![];

    // Maps to the withdrawn_ids list
    let mut withdrawn_i = 0;

    for i in 0..user_unbonding_ids.len() {
        // If all withdrawn handled, or it doesn't collide with withdrawn
        if withdrawn_i >= withdrawn_ids.len() || user_unbonding_ids[i] != withdrawn_ids[withdrawn_i]
        {
            new_unbonding_ids.push(user_unbonding_ids[i]);
        } else {
            // advance withdrawn index
            withdrawn_i += 1;
        }
    }

    USER_UNBONDING_IDS.save(deps.storage, info.sender.clone(), &new_unbonding_ids)?;

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

/*
pub fn do_claim(
    deps: DepsMut,
    &mut reward_pools: Vec<RewardPoolInternal>,
    user: Addr,
    user_staked: Uint128,
    compound: bool,
    stake_token: Addr,
) -> StdResult<Vec<RewardPoolInternal>> {
    for reward_pool in reward_pools.iter_mut() {
        let reward_claimed =
            reward_pool_claim(deps.storage, user.clone(), user_staked, &reward_pool)?;
        reward_pool.claimed += reward_claimed;

        if reward_pool.token.address == stake_token {
            // Compound stake_token rewards
            compound_amount += reward_claimed;
        } else {
            // Claim non-stake_token rewards
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
    Ok(vec![])
}
*/

pub fn compound(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let mut response = Response::new();

    let user_staked = USER_STAKED
        .may_load(deps.storage, info.sender.clone())?
        .unwrap_or(Uint128::zero());

    if user_staked.is_zero() {
        return Err(StdError::generic_err("User has no stake"));
    }

    let total_staked = TOTAL_STAKED.load(deps.storage)?;
    let mut reward_pools =
        update_rewards(env.clone(), &REWARD_POOLS.load(deps.storage)?, total_staked);
    let stake_token = STAKE_TOKEN.load(deps.storage)?;

    let mut compound_amount = Uint128::zero();

    for reward_pool in reward_pools.iter_mut() {
        let reward_claimed =
            reward_pool_claim(deps.storage, info.sender.clone(), user_staked, &reward_pool)?;
        reward_pool.claimed += reward_claimed;

        if reward_pool.token == stake_token {
            // Compound stake_token rewards
            compound_amount += reward_claimed;
        } else {
            // Claim non-stake_token rewards
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
    REWARD_POOLS.save(deps.storage, &reward_pools)?;

    USER_STAKED.save(
        deps.storage,
        info.sender.clone(),
        &(user_staked + compound_amount),
    )?;
    TOTAL_STAKED.save(deps.storage, &(total_staked + compound_amount))?;
    USER_LAST_CLAIM.save(
        deps.storage,
        info.sender,
        &Uint128::new(env.block.time.seconds().into()),
    )?;

    Ok(response.set_data(to_binary(&ExecuteAnswer::Compound {
        status: ResponseStatus::Success,
    })?))
}

pub fn cancel_reward_pool(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: Uint128,
    force: bool,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &config.admin_auth,
    )?;

    let mut reward_pools = REWARD_POOLS.load(deps.storage)?;

    let total_staked = TOTAL_STAKED.load(deps.storage)?;
    let mut response = Response::new();

    if let Some(i) = reward_pools.iter().position(|p| p.id == id) {
        if !force {
            let claim_percent = Uint128::new(
                (reward_pools[i].claimed.u128() * 10u128.pow(18)) / total_staked.u128(),
            );
            if claim_percent < config.reward_cancel_threshold {
                return Err(StdError::generic_err(format!(
                    "Percent claimed {} does not exceed threshold {}",
                    claim_percent, config.reward_cancel_threshold
                )));
            }
        }

        // Send unclaimed funds to creator
        let unclaimed = reward_pools[i].amount - reward_pools[i].claimed;
        if !unclaimed.is_zero() {
            response = response.add_message(send_msg(
                reward_pools[i].creator.clone(),
                unclaimed,
                None,
                None,
                None,
                &reward_pools[i].token,
            )?);
        }
        reward_pools.remove(i);
    } else {
        return Err(StdError::generic_err("Invalid pool id"));
    }

    Ok(
        response.set_data(to_binary(&ExecuteAnswer::CancelRewardPool {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn transfer_stake(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
    recipient: Addr,
    compound: bool,
) -> StdResult<Response> {
    let whitelist = TRANSFER_WL.load(deps.storage)?;

    if !whitelist.contains(&info.sender) {
        return Err(StdError::generic_err(format!(
            "Transfer Stake not allowed for {}",
            info.sender
        )));
    }

    // Claim/Compound for sending user

    // Claim for receiving user

    let sender_staked = USER_STAKED
        .may_load(deps.storage, info.sender.clone())?
        .unwrap_or(Uint128::zero());

    let recipient_staked = USER_STAKED
        .may_load(deps.storage, recipient.clone())?
        .unwrap_or(Uint128::zero());

    // Adjust sender staked
    USER_STAKED.save(deps.storage, info.sender, &(sender_staked - amount))?;

    // Adjust recipient staked
    USER_STAKED.save(deps.storage, recipient, &(recipient_staked + amount))?;

    return Err(StdError::generic_err("Transfer Stake Not Implemented"));

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::TransferStake {
            status: ResponseStatus::Success,
        })?),
    )
}
