use shade_protocol::{
    admin::helpers::{admin_is_valid, validate_admin, AdminPermissions},
    basic_staking::{Action, ExecuteAnswer, RewardPoolInternal, Unbonding},
    c_std::{
        from_binary, to_binary, Addr, Binary, DepsMut, Env, MessageInfo, Response, StdError,
        StdResult, Storage, Uint128,
    },
    contract_interfaces::airdrop::ExecuteMsg::CompleteTask,
    snip20::helpers::{register_receive, send_msg, set_viewing_key_msg},
    utils::{
        asset::{Contract, RawContract},
        generic_response::ResponseStatus,
        ExecuteCallback,
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
    airdrop: Option<RawContract>,
    unbond_period: Option<Uint128>,
    max_user_pools: Option<Uint128>,
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

    if let Some(airdrop) = airdrop {
        config.airdrop = Some(airdrop.into_valid(deps.api)?);
    }

    if let Some(unbond_period) = unbond_period {
        config.unbond_period = unbond_period;
    }

    if let Some(max_user_pools) = max_user_pools {
        config.max_user_pools = max_user_pools;
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

    let mut reward_tokens = REWARD_TOKENS.load(deps.storage)?;

    if reward_tokens.contains(&token) {
        return Err(StdError::generic_err("Reward token already registered"));
    }

    reward_tokens.push(token.clone());
    REWARD_TOKENS.save(deps.storage, &reward_tokens)?;

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
            Action::Stake {
                compound,
                airdrop_task,
            } => {
                let stake_token = STAKE_TOKEN.load(deps.storage)?;
                if info.sender != stake_token.address {
                    return Err(StdError::generic_err(format!(
                        "Invalid Stake Token: {}",
                        info.sender
                    )));
                }

                let compound = compound.unwrap_or(false);

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
                            response = response
                                .add_message(send_msg(
                                    info.sender.clone(),
                                    reward_claimed,
                                    None,
                                    None,
                                    None,
                                    &reward_pool.token,
                                )?)
                                .add_attribute(
                                    reward_pool.token.address.to_string(),
                                    reward_claimed,
                                );
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

                // Send airdrop message
                if let Some(true) = airdrop_task {
                    let config = CONFIG.load(deps.storage)?;
                    if let Some(airdrop) = config.airdrop {
                        response = response.add_message(
                            CompleteTask {
                                address: from.clone(),
                                padding: None,
                            }
                            .to_cosmos_msg(&airdrop, vec![])?,
                        );
                    } else {
                        return Err(StdError::generic_err("No airdrop contract configured"));
                    }
                }

                if compound_amount > Uint128::zero() {
                    response = response.add_attribute("compounded", compound_amount);
                }

                USER_STAKED.save(
                    deps.storage,
                    from.clone(),
                    &(user_staked + amount + compound_amount),
                )?;
                TOTAL_STAKED.save(deps.storage, &(total_staked + amount + compound_amount))?;

                REWARD_POOLS.save(deps.storage, &reward_pools.clone())?;

                Ok(response.set_data(to_binary(&ExecuteAnswer::Stake {
                    staked: user_staked + amount,
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

                    let new_id = MAX_POOL_ID.load(deps.storage)? + Uint128::new(1);
                    MAX_POOL_ID.save(deps.storage, &new_id)?;

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

        response = response
            .add_message(send_msg(
                info.sender.clone(),
                reward_claimed,
                None,
                None,
                None,
                &reward_pool.token,
            )?)
            .add_attribute(reward_pool.token.address.to_string(), reward_claimed);
    }

    REWARD_POOLS.save(deps.storage, &reward_pools)?;

    Ok(response.set_data(to_binary(&ExecuteAnswer::Claim {
        //claimed:
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
        return Err(StdError::generic_err("Must unbond non-zero amount"));
    }

    if let Some(mut user_staked) = USER_STAKED.may_load(deps.storage, info.sender.clone())? {
        // if not compounding, check staked >= unbond amount
        if !compound && user_staked < amount {
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
                response = response
                    .add_message(send_msg(
                        info.sender.clone(),
                        reward_claimed,
                        None,
                        None,
                        None,
                        &reward_pool.token,
                    )?)
                    .add_attribute(reward_pool.token.address.to_string(), reward_claimed);
            }
        }

        // if compounding, check staked + compounded >= unbond amount
        if user_staked + compound_amount < amount {
            return Err(StdError::generic_err(format!(
                "Cannot unbond {}, only {} staked after compounding",
                amount,
                user_staked + compound_amount,
            )));
        }
        if compound_amount > Uint128::zero() {
            response = response.add_attribute("compounded", compound_amount);
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
            unbonded: amount,
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

    let now = Uint128::new(env.block.time.seconds() as u128);

    let mut withdrawn_ids = vec![];
    let mut withdrawn_amount = Uint128::zero();

    for id in ids.into_iter() {
        if let Some(unbonding) =
            USER_UNBONDING.may_load(deps.storage, user_unbonding_key(info.sender.clone(), id))?
        {
            if now >= unbonding.complete {
                withdrawn_amount += unbonding.amount;
                withdrawn_ids.push(id);
            }
        } else {
            return Err(StdError::generic_err(format!("Bad ID {}", id)));
        }
    }

    if withdrawn_amount.is_zero() {
        return Ok(Response::new()
            .add_attribute("withdrawn", withdrawn_amount)
            .set_data(to_binary(&ExecuteAnswer::Withdraw {
                withdrawn: withdrawn_amount,
                status: ResponseStatus::Success,
            })?));
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
            withdrawn_amount,
            None,
            None,
            None,
            &STAKE_TOKEN.load(deps.storage)?,
        )?)
        .add_attribute("withdrawn", withdrawn_amount)
        .set_data(to_binary(&ExecuteAnswer::Withdraw {
            withdrawn: withdrawn_amount,
            status: ResponseStatus::Success,
        })?))
}

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
            response = response
                .add_message(send_msg(
                    info.sender.clone(),
                    reward_claimed,
                    None,
                    None,
                    None,
                    &reward_pool.token,
                )?)
                .add_attribute(reward_pool.token.address.to_string(), reward_claimed);
        }
    }
    REWARD_POOLS.save(deps.storage, &reward_pools)?;

    if compound_amount > Uint128::zero() {
        response = response.add_attribute("compounded", compound_amount);
    }

    USER_STAKED.save(
        deps.storage,
        info.sender.clone(),
        &(user_staked + compound_amount),
    )?;
    TOTAL_STAKED.save(deps.storage, &(total_staked + compound_amount))?;

    Ok(response.set_data(to_binary(&ExecuteAnswer::Compound {
        compounded: compound_amount,
        status: ResponseStatus::Success,
    })?))
}

pub fn end_reward_pool(
    deps: DepsMut,
    env: Env,
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

    let total_staked = TOTAL_STAKED.load(deps.storage)?;
    let mut reward_pools =
        update_rewards(env.clone(), &REWARD_POOLS.load(deps.storage)?, total_staked);

    // Amount of rewards pulled from contract
    let mut extract_amount = Uint128::zero();

    let now = Uint128::new(env.block.time.seconds() as u128);

    let pool_i = match reward_pools.iter().position(|p| p.id == id) {
        Some(i) => i,
        None => {
            return Err(StdError::generic_err("Could not match id"));
        }
    };

    // Remove reward pool, will edit & push it later
    let mut reward_pool = reward_pools.remove(pool_i);

    // Delete reward pool if it hasn't started
    let deleted = if reward_pool.start > now {
        println!("DELETING BEFORE START");
        extract_amount = reward_pool.amount;
        true
    }
    // Reward pool hasn't ended, trim off un-emitted tokens & edit pool to end now
    else if reward_pool.end > now {
        // remove rewards from now -> end
        extract_amount = reward_pool.rate * (reward_pool.end - now) / Uint128::new(10u128.pow(18));
        println!("EXTRACTING {}", extract_amount);
        reward_pool.end = now;
        reward_pool.amount -= extract_amount;

        if reward_pool.claimed == reward_pool.amount {
            true
        } else {
            reward_pools.push(reward_pool.clone());
            false
        }
    }
    // Delete reward pool if reward pool is fully claimed, or forced
    else if reward_pool.claimed == reward_pool.amount || force {
        extract_amount += reward_pool.amount - reward_pool.claimed;
        true
    } else {
        return Err(StdError::generic_err(
            "Reward pool is complete but claims are still pending",
        ));
    };

    REWARD_POOLS.save(deps.storage, &reward_pools)?;

    Ok(Response::new()
        .add_message(send_msg(
            info.sender,
            extract_amount,
            None,
            None,
            None,
            &reward_pool.token,
        )?)
        .set_data(to_binary(&ExecuteAnswer::EndRewardPool {
            deleted,
            extracted: extract_amount,
            status: ResponseStatus::Success,
        })?))
}

pub fn add_transfer_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    user: Addr,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &config.admin_auth,
    )?;

    let mut whitelist = TRANSFER_WL.load(deps.storage)?;

    if whitelist.contains(&user) {
        return Err(StdError::generic_err("User already whitelisted"));
    }

    whitelist.push(user);

    TRANSFER_WL.save(deps.storage, &whitelist)?;

    Ok(
        Response::default().set_data(to_binary(&ExecuteAnswer::AddTransferWhitelist {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn rm_transfer_whitelist(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    user: Addr,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &config.admin_auth,
    )?;

    let mut whitelist = TRANSFER_WL.load(deps.storage)?;

    match whitelist.iter().position(|u| *u == user) {
        Some(i) => {
            whitelist.remove(i);
        }
        None => {
            return Err(StdError::generic_err("User not in whitelist"));
        }
    }

    TRANSFER_WL.save(deps.storage, &whitelist)?;

    Ok(
        Response::default().set_data(to_binary(&ExecuteAnswer::AddTransferWhitelist {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn transfer_stake(
    deps: DepsMut,
    env: Env,
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
    let total_staked = TOTAL_STAKED.load(deps.storage)?;

    let mut reward_pools =
        update_rewards(env.clone(), &REWARD_POOLS.load(deps.storage)?, total_staked);

    let stake_token = STAKE_TOKEN.load(deps.storage)?;

    let mut response = Response::new();

    let sender_staked = USER_STAKED
        .may_load(deps.storage, info.sender.clone())?
        .unwrap_or(Uint128::zero());

    if sender_staked == Uint128::zero() {
        return Err(StdError::generic_err("Cannot transfer with 0 staked"));
    }

    let mut sender_compound_amount = Uint128::zero();

    // Claim/Compound rewards for Sender
    for reward_pool in reward_pools.iter_mut() {
        println!("reward pool claim sender");
        let reward_claimed = reward_pool_claim(
            deps.storage,
            info.sender.clone(),
            sender_staked,
            &reward_pool,
        )?;
        println!("POST reward pool claim sender");
        reward_pool.claimed += reward_claimed;

        if compound && reward_pool.token == stake_token {
            // Compound stake_token rewards
            sender_compound_amount += reward_claimed;
        } else {
            // Claim if not compound or not stake token rewards
            response = response
                .add_message(send_msg(
                    info.sender.clone(),
                    reward_claimed,
                    None,
                    None,
                    None,
                    &reward_pool.token,
                )?)
                .add_attribute(reward_pool.token.address.to_string(), reward_claimed);
        }
    }

    if sender_staked + sender_compound_amount < amount {
        return Err(StdError::generic_err(format!(
            "Cannot transfer {}, only {} available",
            amount,
            sender_staked + sender_compound_amount
        )));
    }

    println!("sender compound amount {}", sender_compound_amount);

    if sender_compound_amount > Uint128::zero() {
        response = response.add_attribute("compounded", sender_compound_amount);
    }

    // Adjust sender staked
    USER_STAKED.save(
        deps.storage,
        info.sender,
        &(sender_staked + sender_compound_amount - amount),
    )?;

    // Claim for receiving user
    let recipient_staked = USER_STAKED
        .may_load(deps.storage, recipient.clone())?
        .unwrap_or(Uint128::zero());

    // Claim rewards for Receiver (no compound)
    for reward_pool in reward_pools.iter_mut() {
        println!("reward pool claim recipient");
        let reward_claimed = reward_pool_claim(
            deps.storage,
            recipient.clone(),
            recipient_staked,
            &reward_pool,
        )?;
        reward_pool.claimed += reward_claimed;

        // Claim if not compound or not stake token rewards
        println!(
            "SENDING RECIPIETN REWARD {} {}",
            reward_claimed,
            recipient.clone()
        );
        response = response.add_message(send_msg(
            recipient.clone(),
            reward_claimed,
            None,
            None,
            None,
            &reward_pool.token,
        )?);
    }

    // Adjust recipient staked
    USER_STAKED.save(deps.storage, recipient, &(recipient_staked + amount))?;

    Ok(response.set_data(to_binary(&ExecuteAnswer::TransferStake {
        transferred: amount,
        status: ResponseStatus::Success,
    })?))
}
