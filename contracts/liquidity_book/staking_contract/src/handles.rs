use std::{
    collections::HashMap,
    ops::{Add, AddAssign, Sub},
    str::FromStr,
    vec,
};

use shade_protocol::{
    self,
    admin::helpers::{validate_admin, AdminPermissions},
    c_std::{
        from_binary,
        to_binary,
        Addr,
        Binary,
        ContractInfo,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
        Storage,
        Uint128,
        Uint256,
    },
    lb_libraries::types::TreeUint24,
    liquidity_book::{
        lb_pair::RewardsDistribution,
        lb_token::TransferAction,
        staking::{
            EpochInfo,
            ExecuteAnswer,
            InvokeMsg,
            Reward,
            RewardToken,
            RewardTokenInfo,
            StakerLiquidity,
            StakerLiquiditySnapshot,
            State,
        },
    },
    s_toolkit::{
        permit::RevokedPermits,
        viewing_key::{ViewingKey, ViewingKeyStore},
    },
    snip20::{helpers::token_info, ExecuteMsg as Snip20ExecuteMsg},
    utils::{asset::RawContract, pad_handle_result, ExecuteCallback},
    Contract,
    BLOCK_SIZE,
};

use crate::{
    helper::{
        assert_lb_pair,
        check_if_claimable,
        finding_total_liquidity,
        finding_user_liquidity,
        register_reward_tokens,
        require_lb_token,
        staker_init_checker,
        TokenKey,
    },
    state::{
        store_claim_rewards,
        store_stake,
        store_unstake,
        EPOCH_STORE,
        EXPIRED_AT_LOGGER,
        EXPIRED_AT_LOGGER_MAP,
        REWARD_TOKENS,
        REWARD_TOKEN_INFO,
        STAKERS,
        STAKERS_BIN_TREE,
        STAKERS_LIQUIDITY,
        STAKERS_LIQUIDITY_SNAPSHOT,
        STATE,
        TOTAL_LIQUIDITY,
        TOTAL_LIQUIDITY_SNAPSHOT,
    },
};

pub const SHADE_STAKING_VIEWING_KEY: &str = "SHADE_STAKING_VIEWING_KEY";
pub const PREFIX_REVOKED_PERMITS: &str = "revoked_permits";
pub fn receiver_callback_snip_1155(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: Addr,
    token_id: String,
    amount: Uint256,
    msg: Option<Binary>,
) -> StdResult<Response> {
    let msg = msg.ok_or_else(|| {
        StdError::generic_err("Receiver callback \"msg\" parameter cannot be empty.")
    })?;

    let state = STATE.load(deps.storage)?;
    require_lb_token(&state, &info.sender)?;

    pad_handle_result(
        match from_binary(&msg)? {
            InvokeMsg::Stake {
                from: invoke_msg_from,
                padding: _,
            } => {
                let checked_from = if let Some(invoke_msg_from) = invoke_msg_from {
                    deps.api.addr_validate(invoke_msg_from.as_str())?
                } else {
                    from
                };
                try_stake(
                    deps,
                    env,
                    state,
                    checked_from,
                    token_id.parse().unwrap(),
                    amount,
                )
            }
            InvokeMsg::AddRewards { .. } => Err(StdError::generic_err("Wrong Receiver called")),
        },
        BLOCK_SIZE,
    )
}

pub fn receiver_callback(
    deps: DepsMut,
    info: MessageInfo,
    from: Addr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<Response> {
    let msg = msg.ok_or_else(|| {
        StdError::generic_err("Receiver callback \"msg\" parameter cannot be empty.")
    })?;

    let state = STATE.load(deps.storage)?;

    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        from,
        &state.admin_auth.into(),
    )?;

    pad_handle_result(
        match from_binary(&msg)? {
            InvokeMsg::Stake { .. } => Err(StdError::generic_err("Wrong Receiver")),
            InvokeMsg::AddRewards { start, end } => try_add_rewards(deps, info, start, end, amount),
        },
        BLOCK_SIZE,
    )
}

pub fn try_stake(
    deps: DepsMut,
    env: Env,
    mut state: State,
    staker: Addr,
    token_id: u32,
    amount: Uint256,
) -> StdResult<Response> {
    //LOADING general use readonly stores
    let epoch_obj = match EPOCH_STORE.may_load(deps.storage, state.epoch_index)? {
        Some(e) => Ok(e),
        None => Err(StdError::generic_err("Reward token storage already exists")),
    }?;
    //1) UPDATING: staker_info and staker_liquidity_snapshot
    //*INIT STAKER_INFO if not initialized already
    staker_init_checker(deps.storage, &state, &staker)?;
    //*LOADING readonly stores
    let mut staker_liq = STAKERS_LIQUIDITY
        .load(deps.storage, (&staker, token_id))
        .unwrap_or_default();
    let mut staker_liq_snap = STAKERS_LIQUIDITY_SNAPSHOT
        .load(deps.storage, (&staker, state.epoch_index, token_id))
        .unwrap_or_default();

    //**Fetching Liquidity
    let liquidity: Uint256;
    if staker_liq_snap.liquidity.is_zero() {
        liquidity = staker_liq.amount_delegated;
    } else {
        liquidity = staker_liq_snap.liquidity; //can panic here
    }

    //**Only adding to liquidity if round has not ended yet
    if env.block.time.seconds() >= epoch_obj.end_time {
        staker_liq_snap.liquidity = liquidity;
    } else {
        staker_liq_snap.liquidity = liquidity.add(amount.multiply_ratio(
            if let Some(time) = epoch_obj.end_time.checked_sub(env.block.time.seconds()) {
                time
            } else {
                return Err(StdError::generic_err("Under-flow sub error"));
            },
            epoch_obj.duration,
        ));
    }

    if staker_liq.amount_delegated.is_zero() {
        STAKERS_BIN_TREE.update(deps.storage, &staker, |tree| -> StdResult<_> {
            if let Some(mut t) = tree {
                t.add(token_id);
                Ok(t)
            } else {
                let mut new_tree = TreeUint24::new();
                new_tree.add(token_id);
                Ok(new_tree)
            }
        })?;
    }

    //*STORING user_info_obj and user_liquidity_snapshot
    staker_liq.amount_delegated.add_assign(amount);
    STAKERS_LIQUIDITY.save(deps.storage, (&staker, token_id), &staker_liq)?;

    staker_liq_snap.amount_delegated = staker_liq.amount_delegated;
    STAKERS_LIQUIDITY_SNAPSHOT.save(
        deps.storage,
        (&staker, state.epoch_index, token_id),
        &staker_liq_snap,
    )?;

    //2) UPDATING: PoolState & PoolStateLiquidityStats & Config
    //*LOADING readonly stores
    let mut total_liq = TOTAL_LIQUIDITY
        .load(deps.storage, token_id)
        .unwrap_or_default();
    let mut total_liq_snap = TOTAL_LIQUIDITY_SNAPSHOT
        .load(deps.storage, (state.epoch_index, token_id))
        .unwrap_or_default();
    //*UPDATING PoolStateLiquidityStats
    //**Fetching Liquidity
    let total_liquidity: Uint256;
    if total_liq_snap.liquidity.is_zero() {
        total_liquidity = total_liq.amount_delegated;
    } else {
        total_liquidity = total_liq_snap.liquidity;
    }

    //**Only adding to liquidity if round has not ended yet
    if env.block.time.seconds() >= epoch_obj.end_time {
        total_liq_snap.liquidity = total_liquidity;
    } else {
        total_liq_snap.liquidity = total_liquidity.add(amount.multiply_ratio(
            if let Some(time) = epoch_obj.end_time.checked_sub(env.block.time.seconds()) {
                time
            } else {
                return Err(StdError::generic_err("Under-flow sub error"));
            },
            epoch_obj.duration,
        ));
    }

    total_liq.amount_delegated.add_assign(amount);
    total_liq.last_deposited = Some(state.epoch_index);
    total_liq_snap.amount_delegated = total_liq.amount_delegated;
    //*STORING pool_state
    TOTAL_LIQUIDITY.save(deps.storage, token_id, &total_liq)?;
    //*STORING pool_state_liquidity_snapshot
    TOTAL_LIQUIDITY_SNAPSHOT.save(deps.storage, (state.epoch_index, token_id), &total_liq_snap)?;

    store_stake(
        deps.storage,
        staker,
        &mut state,
        vec![token_id],
        vec![amount],
        env.block.time.seconds(),
        env.block.height,
    )?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

pub fn try_unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_ids: Vec<u32>,
    amounts: Vec<Uint256>,
) -> StdResult<Response> {
    if amounts.len() != token_ids.len() {
        return Err(StdError::generic_err(
            "ids array not equal to amounts array",
        ));
    }

    let mut state = STATE.load(deps.storage)?;
    let epoch_obj = EPOCH_STORE
        .may_load(deps.storage, state.epoch_index)?
        .ok_or_else(|| StdError::generic_err("Reward token storage does not exist"))?;

    staker_init_checker(deps.storage, &state, &info.sender)?;

    let mut actions: Vec<TransferAction> = Vec::new();

    for (token_id, amount) in token_ids.iter().zip(amounts.iter()) {
        let mut staker_liq = STAKERS_LIQUIDITY
            .load(deps.storage, (&info.sender, *token_id))
            .unwrap_or_default();
        let mut staker_liq_snap = STAKERS_LIQUIDITY_SNAPSHOT
            .load(deps.storage, (&info.sender, state.epoch_index, *token_id))
            .unwrap_or_default();

        if amount.is_zero() {
            return Err(StdError::generic_err("cannot request to withdraw 0"));
        }

        if staker_liq.amount_delegated < *amount {
            return Err(StdError::generic_err(format!(
                "insufficient funds to redeem: balance={}, required={}",
                staker_liq.amount_delegated, amount
            )));
        }

        let liquidity = if staker_liq_snap.liquidity.is_zero() {
            staker_liq.amount_delegated
        } else {
            staker_liq_snap.liquidity
        };

        if env.block.time.seconds() < epoch_obj.end_time {
            let subtraction_amount = amount.multiply_ratio(
                epoch_obj
                    .end_time
                    .checked_sub(env.block.time.seconds())
                    .ok_or_else(|| StdError::generic_err("Overflow in time calculation"))?,
                epoch_obj.duration,
            );

            staker_liq_snap.liquidity = liquidity
                .checked_sub(subtraction_amount)
                .map_err(|_| StdError::generic_err("Underflow in subtracting from liquidity"))?;
        } else {
            staker_liq_snap.liquidity = liquidity;
        }

        update_staker_and_total_liquidity(
            deps.storage,
            &env,
            &state,
            &epoch_obj,
            &info.sender,
            *token_id,
            *amount,
            &mut staker_liq,
            &mut staker_liq_snap,
        )?;

        actions.push(TransferAction {
            token_id: token_id.to_string(),
            from: env.contract.address.clone(),
            recipient: info.sender.clone(),
            amount: *amount,
            memo: None,
        });
    }

    let message = shade_protocol::liquidity_book::lb_token::ExecuteMsg::BatchTransfer {
        actions,
        padding: None,
    }
    .to_cosmos_msg(
        state.lb_token.code_hash.clone(),
        state.lb_token.address.to_string(),
        None,
    )?;

    store_unstake(
        deps.storage,
        info.sender,
        &mut state,
        token_ids,
        amounts,
        env.block.time.seconds(),
        env.block.height,
    )?;
    STATE.save(deps.storage, &state)?;

    Ok(Response::default().add_message(message))
}

fn update_staker_and_total_liquidity(
    storage: &mut dyn Storage,
    env: &Env,
    state: &State,
    epoch_obj: &EpochInfo,
    sender: &Addr,
    token_id: u32,
    amount: Uint256,
    staker_liq: &mut StakerLiquidity,
    staker_liq_snap: &mut StakerLiquiditySnapshot,
) -> StdResult<()> {
    // Example logic (replace with actual implementation)
    // Update staker's liquidity
    staker_liq.amount_delegated =
        staker_liq
            .amount_delegated
            .checked_sub(amount)
            .map_err(|_| {
                StdError::generic_err("Underflow in subtracting from staker amount_delegated")
            })?;

    // Update staker's liquidity snapshot
    staker_liq_snap.amount_delegated = staker_liq.amount_delegated;

    STAKERS_LIQUIDITY.save(storage, (sender, token_id), staker_liq)?;
    STAKERS_LIQUIDITY_SNAPSHOT.save(
        storage,
        (sender, state.epoch_index, token_id),
        staker_liq_snap,
    )?;

    //2) UPDATING: PoolState & PoolStateLiquidityStats & Config
    //*LOADING readonly stores
    let mut total_liq = TOTAL_LIQUIDITY.load(storage, token_id).unwrap_or_default();
    let mut total_liq_snap = TOTAL_LIQUIDITY_SNAPSHOT
        .load(storage, (state.epoch_index, token_id))
        .unwrap_or_default();
    //*UPDATING PoolStateLiquidityStats
    //**Fetching Liquidity
    let total_liquidity: Uint256;
    if total_liq_snap.liquidity.is_zero() {
        total_liquidity = total_liq.amount_delegated;
    } else {
        total_liquidity = total_liq_snap.liquidity;
    }

    //**Only adding to liquidity if round has not ended yet
    if env.block.time.seconds() >= epoch_obj.end_time {
        total_liq_snap.liquidity = total_liquidity;
    } else {
        let subtraction_amount = amount.multiply_ratio(
            if let Some(time) = epoch_obj.end_time.checked_sub(env.block.time.seconds()) {
                time
            } else {
                // Handle overflow in time calculation
                return Err(StdError::generic_err("Overflow in time calculation"));
            },
            epoch_obj.duration,
        );

        // Use checked_sub to prevent underflow when subtracting from total_liquidity
        total_liq_snap.liquidity =
            if let Ok(new_liquidity) = total_liquidity.checked_sub(subtraction_amount) {
                new_liquidity
            } else {
                // Handle underflow in subtracting from total_liquidity
                return Err(StdError::generic_err(
                    "Underflow in subtracting from total_liquidity",
                ));
            };
    }

    // Use checked_sub to safely subtract amounts[i] from total_liq.amount_delegated
    if let Ok(new_amount_delegated) = total_liq.amount_delegated.checked_sub(amount) {
        total_liq.amount_delegated = new_amount_delegated;
    } else {
        // Handle potential underflow error
        return Err(StdError::generic_err(
            "Underflow in subtracting from amount_delegated",
        ));
    }
    total_liq.last_deposited = Some(state.epoch_index);
    total_liq_snap.amount_delegated = total_liq.amount_delegated;
    //*STORING pool_state
    TOTAL_LIQUIDITY.save(storage, token_id, &total_liq)?;
    //*STORING pool_state_liquidity_snapshot
    TOTAL_LIQUIDITY_SNAPSHOT.save(storage, (state.epoch_index, token_id), &total_liq_snap)?;

    Ok(())
}

pub fn try_end_epoch(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    rewards_distribution: RewardsDistribution,
) -> StdResult<Response> {
    let mut state = STATE.load(deps.storage)?;
    //check that only be called by lp-pair
    assert_lb_pair(&state, info)?;

    let mut reward_tokens = vec![];

    for reward_token in REWARD_TOKENS.load(deps.storage)? {
        let rewards_token_info;
        if let Ok(r_t_i) = REWARD_TOKEN_INFO.load(deps.storage, &reward_token.address) {
            rewards_token_info = r_t_i;
        } else {
            continue;
        };

        // Create a filtered list of rewards to add to reward_tokens
        let rewards_to_add: Vec<_> = rewards_token_info
            .iter()
            .filter(|reward| reward.end >= state.epoch_index)
            .cloned()
            .collect();

        // Extend reward_tokens with the filtered rewards
        reward_tokens.extend(rewards_to_add.clone());

        // Check if any rewards were removed, and save the updated list if so
        if rewards_to_add.len() != rewards_token_info.len() {
            REWARD_TOKEN_INFO.save(deps.storage, &reward_token.address, &rewards_to_add)?;
        }
    }

    //saves the distribution
    let mut epoch_obj = EPOCH_STORE.load(deps.storage, state.epoch_index)?;
    epoch_obj.rewards_distribution = Some(rewards_distribution);
    epoch_obj.reward_tokens = Some(reward_tokens);

    if let Some(expiry_duration) = state.expiry_durations {
        let expired_at = state.epoch_index + expiry_duration;
        epoch_obj.expired_at = Some(expired_at);
        EXPIRED_AT_LOGGER.update(deps.storage, |mut list| -> StdResult<Vec<u64>> {
            if !list.contains(&expired_at) {
                list.push(expired_at);
            }
            Ok(list)
        })?;
        EXPIRED_AT_LOGGER_MAP.update(
            deps.storage,
            expired_at,
            |logger| -> StdResult<Vec<u64>> {
                if let Some(mut l) = logger {
                    l.push(state.epoch_index);
                    Ok(l)
                } else {
                    Ok(vec![state.epoch_index])
                }
            },
        )?;
    }

    EPOCH_STORE.save(deps.storage, state.epoch_index, &epoch_obj)?;

    let now = env.block.time.seconds();
    EPOCH_STORE.save(deps.storage, state.epoch_index.add(1), &EpochInfo {
        rewards_distribution: None,
        start_time: now,
        end_time: now + state.epoch_durations,
        duration: state.epoch_durations,
        reward_tokens: None,
        expired_at: None,
    })?;

    state.epoch_index.add_assign(1);

    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

pub fn try_claim_rewards(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let mut state = STATE.load(deps.storage)?;
    let staker_result = STAKERS.load(deps.storage, &info.sender);
    let mut messages = Vec::new();

    let mut rewards_accumulator: HashMap<(u64, TokenKey), (Uint128, Vec<Uint128>, Vec<u32>)> =
        HashMap::new();

    // Check if staker exists
    let mut staker = match staker_result {
        Ok(staker) => staker,
        Err(_) => {
            return Err(StdError::generic_err(format!(
                "No staker found for address {:?}.",
                info.sender
            )));
        }
    };

    // Determine the starting epoch for reward calculation
    let starting_epoch = if let Some(last_claim_round) = staker.last_claim_rewards_round {
        if last_claim_round >= state.epoch_index.saturating_sub(1) {
            // If rewards were already claimed for the last epoch
            return Err(StdError::generic_err(
                "You have already claimed rewards for the latest epoch.",
            ));
        }
        last_claim_round + 1
    } else {
        // First-time claim check
        check_if_claimable(staker.starting_round, state.epoch_index)?;
        staker.starting_round.unwrap_or_default()
    };

    let ending_epoch = state.epoch_index;

    for round_epoch in starting_epoch..ending_epoch {
        let mut epoch_info = EPOCH_STORE.load(deps.storage, round_epoch)?;

        // Check if the epoch has expired or has no reward tokens
        let is_expired = epoch_info
            .expired_at
            .map_or(false, |expired_at| expired_at <= state.epoch_index);
        let has_no_reward_tokens = epoch_info
            .reward_tokens
            .as_ref()
            .map_or(true, Vec::is_empty);

        if is_expired || has_no_reward_tokens {
            continue;
        }

        // Process reward distribution
        if let Some(rewards_distribution) = &epoch_info.rewards_distribution {
            for (id, weightage) in rewards_distribution
                .clone()
                .ids
                .iter()
                .zip(rewards_distribution.weightages.iter())
            {
                let (staker_liq_snap, is_calculated) =
                    finding_user_liquidity(deps.storage, &info, &staker, round_epoch, *id)?;
                if is_calculated {
                    STAKERS_LIQUIDITY_SNAPSHOT.save(
                        deps.storage,
                        (&info.sender, round_epoch, *id),
                        &staker_liq_snap,
                    )?;
                }
                if staker_liq_snap.liquidity.is_zero() {
                    continue;
                }

                let (total_liquidity_snap, is_calculated) =
                    finding_total_liquidity(deps.storage, round_epoch, *id)?;

                if is_calculated {
                    TOTAL_LIQUIDITY_SNAPSHOT.save(
                        deps.storage,
                        (round_epoch, *id),
                        &total_liquidity_snap,
                    )?;
                }

                if total_liquidity_snap.liquidity.is_zero() {
                    continue;
                }

                // Calculate and distribute rewards
                if let Some(reward_tokens) = &mut epoch_info.reward_tokens {
                    for r_t in reward_tokens.iter_mut() {
                        let total_bin_rewards = Uint256::from(
                            r_t.reward_per_epoch
                                .multiply_ratio(*weightage, rewards_distribution.denominator),
                        );

                        let staker_rewards = Uint128::from_str(
                            &total_bin_rewards
                                .multiply_ratio(
                                    staker_liq_snap.liquidity,
                                    total_liquidity_snap.liquidity,
                                )
                                .to_string(),
                        )?;
                        r_t.claimed_rewards += staker_rewards;

                        let token_key = TokenKey {
                            address: r_t.token.address.clone(),
                            code_hash: r_t.token.code_hash.clone(),
                        };

                        let entry = rewards_accumulator
                            .entry((round_epoch, token_key.clone()))
                            .or_insert((Uint128::zero(), Vec::new(), Vec::new()));
                        entry.0 += staker_rewards;
                        entry.1.push(staker_rewards);
                        entry.2.push(*id);
                    }
                }
            }
        }
        EPOCH_STORE.save(deps.storage, round_epoch, &epoch_info)?;
    }

    let mut rewards_by_epoch: HashMap<u64, Vec<RewardToken>> = HashMap::new();

    // Create messages for each token with the accumulated rewards
    for ((epoch_id, token_key), (total_amount, amounts, ids)) in rewards_accumulator {
        let contract_info = ContractInfo {
            address: token_key.address,
            code_hash: token_key.code_hash,
        };
        messages.push(
            Snip20ExecuteMsg::Send {
                recipient: info.sender.to_string(),
                recipient_code_hash: None,
                amount: total_amount,
                msg: None,
                memo: None,
                padding: None,
            }
            .to_cosmos_msg(&contract_info, vec![])?,
        );

        rewards_by_epoch
            .entry(epoch_id)
            .or_default()
            .push(RewardToken {
                token: contract_info,
                ids,
                amounts,
                total_amount,
            });
    }

    staker.last_claim_rewards_round = Some(ending_epoch - 1);
    STAKERS.save(deps.storage, &info.sender, &staker)?;

    let rewards: Vec<Reward> = rewards_by_epoch
        .into_iter()
        .map(|(epoch_index, reward_tokens)| Reward {
            epoch_index,
            rewards: reward_tokens,
        })
        .collect();

    if rewards.len() > 0 {
        store_claim_rewards(
            deps.storage,
            info.sender,
            &mut state,
            rewards,
            env.block.time.seconds(),
            env.block.height,
        )?;
    }
    STATE.save(deps.storage, &state)?;

    Ok(Response::default().add_messages(messages))
}

pub fn try_register_reward_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    tokens: Vec<ContractInfo>,
) -> StdResult<Response> {
    let state = STATE.load(deps.storage)?;
    //check that only be called by lb-pair
    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &state.admin_auth.into(),
    )?;

    let msgs = register_reward_tokens(deps.storage, tokens, env.contract.code_hash)?;

    Ok(Response::default().add_messages(msgs))
}

pub fn try_update_config(
    deps: DepsMut,
    info: MessageInfo,
    admin_auth: Option<RawContract>,
    query_auth: Option<RawContract>,
    epoch_duration: Option<u64>,
    expiry_duration: Option<u64>,
) -> StdResult<Response> {
    let mut state = STATE.load(deps.storage)?;
    //check that only be called by lb-pair
    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &state.admin_auth.clone().into(),
    )?;

    if let Some(ad_auth) = admin_auth {
        state.admin_auth = ad_auth.valid(deps.api)?;
    }

    if let Some(q_auth) = query_auth {
        state.query_auth = Some(q_auth.valid(deps.api)?);
    }

    if let Some(ep_duration) = epoch_duration {
        state.epoch_durations = ep_duration;
    }

    if let Some(exp_duration) = expiry_duration {
        state.expiry_durations = Some(exp_duration);
    }

    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

pub fn try_add_rewards(
    deps: DepsMut,
    info: MessageInfo,
    start: Option<u64>,
    end: u64,
    amount: Uint128,
) -> StdResult<Response> {
    let state = STATE.load(deps.storage)?;
    let start = start.unwrap_or(state.epoch_index);
    let reward_tokens = REWARD_TOKENS.load(deps.storage)?;

    if let Some(token) = reward_tokens
        .iter()
        .find(|contract| contract.address == info.sender)
    {
        // Disallow end before start
        if start > end {
            return Err(StdError::generic_err("'start' must be after 'end'"));
        }
        // Disallow retro-active emissions (maybe could allow?)
        if start < state.epoch_index {
            return Err(StdError::generic_err("Cannot start emitting in the past"));
        }

        let decimals = token_info(&deps.querier, &Contract {
            address: token.address.clone(),
            code_hash: token.code_hash.clone(),
        })?
        .decimals;

        let total_epoches = end.sub(start) + 1;

        let reward_per_epoch = amount.multiply_ratio(Uint128::one(), total_epoches);

        let rewards_info_obj = RewardTokenInfo {
            token: token.clone(),
            decimals,
            reward_per_epoch,
            start,
            end,
            total_rewards: amount,
            claimed_rewards: Uint128::zero(),
        };

        REWARD_TOKEN_INFO.update(
            deps.storage,
            &token.address,
            |i| -> StdResult<Vec<RewardTokenInfo>> {
                if let Some(mut t) = i {
                    t.push(rewards_info_obj);
                    Ok(t)
                } else {
                    Ok(vec![rewards_info_obj])
                }
            },
        )?;
    } else {
        return Err(StdError::generic_err(format!(
            "Invalid Reward: {}",
            info.sender
        )));
    }

    Ok(Response::default())
}

pub fn try_create_viewing_key(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    entropy: String,
) -> StdResult<Response> {
    let key = ViewingKey::create(
        deps.storage,
        &info,
        &env,
        info.sender.as_str(),
        entropy.as_ref(),
    );

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::CreateViewingKey { key })?))
}

pub fn try_set_viewing_key(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    key: String,
) -> StdResult<Response> {
    ViewingKey::set(deps.storage, info.sender.as_str(), key.as_str());
    Ok(Response::new())
}

pub fn try_revoke_permit(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    permit_name: String,
) -> StdResult<Response> {
    RevokedPermits::revoke_permit(
        deps.storage,
        PREFIX_REVOKED_PERMITS,
        info.sender.as_ref(),
        &permit_name,
    );

    Ok(Response::new())
}

pub fn try_recover_funds(deps: DepsMut, _env: Env, info: MessageInfo) -> StdResult<Response> {
    let state = STATE.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &state.admin_auth.into(),
    )?;
    let mut messages = Vec::new();

    let mut sorted_epochs = EXPIRED_AT_LOGGER.load(deps.storage)?;
    sorted_epochs.sort();

    // Find the first epoch that is greater than the current state epoch_index
    let first_future_epoch = sorted_epochs
        .iter()
        .find(|&&epoch| epoch > state.epoch_index);

    for &expired_epoch in sorted_epochs.iter() {
        if Some(&expired_epoch) == first_future_epoch {
            break;
        }

        let epoch_ids = EXPIRED_AT_LOGGER_MAP.load(deps.storage, expired_epoch)?;

        for &epoch_id in &epoch_ids {
            if let Some(reward_tokens) = EPOCH_STORE.load(deps.storage, epoch_id)?.reward_tokens {
                for reward in reward_tokens {
                    if reward.reward_per_epoch > reward.claimed_rewards {
                        let amount = reward.reward_per_epoch.sub(reward.claimed_rewards);
                        messages.push(
                            Snip20ExecuteMsg::Send {
                                recipient: state.recover_funds_receiver.to_string(),
                                recipient_code_hash: None,
                                amount,
                                msg: None,
                                memo: None,
                                padding: None,
                            }
                            .to_cosmos_msg(&reward.token, vec![])?,
                        );
                    }
                }
            }
        }
    }

    // Update EXPIRED_AT_LOGGER with future epochs only
    if let Some(&last_future_epoch) = first_future_epoch {
        let future_epochs: Vec<u64> = sorted_epochs
            .into_iter()
            .filter(|&epoch| epoch > last_future_epoch)
            .collect();
        EXPIRED_AT_LOGGER.save(deps.storage, &future_epochs)?;
    }

    Ok(Response::new().add_messages(messages))
}
