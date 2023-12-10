use std::{
    ops::{Add, AddAssign, Sub},
    str::FromStr,
};

use shade_protocol::{
    self,
    admin::helpers::{validate_admin, AdminPermissions},
    c_std::{
        entry_point,
        from_binary,
        to_binary,
        Addr,
        Attribute,
        Binary,
        ContractInfo,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
        Uint128,
        Uint256,
    },
    contract_interfaces::liquidity_book::lb_libraries::viewing_keys::{
        register_receive,
        set_viewing_key_msg,
    },
    lb_libraries::types::TreeUint24,
    liquidity_book::{
        lb_pair::RewardsDistribution,
        lb_token::TransferAction,
        staking::{
            EpochInfo,
            ExecuteMsg,
            InstantiateMsg,
            InvokeMsg,
            QueryMsg,
            RewardTokenCreate,
            RewardTokenInfo,
            State,
        },
    },
    snip20::{helpers::token_info, ExecuteMsg as Snip20ExecuteMsg},
    utils::{pad_handle_result, ExecuteCallback},
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
        require_lp_token,
        staker_init_checker,
        store_empty_reward_set,
    },
    state::{
        EPOCH_STORE,
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

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let mut state = State {
        lp_token: msg.lb_token.valid(deps.api)?,
        lb_pair: deps.api.addr_validate(&msg.amm_pair)?,
        admin_auth: msg.admin_auth.valid(deps.api)?,
        query_auth: None,
        total_amount_staked: Uint128::zero(),
        epoch_index: msg.epoch_index,
        epoch_durations: msg.epoch_duration,
        expiry_durations: msg.expiry_duration,
    };

    if let Some(query_auth) = msg.query_auth {
        state.query_auth = Some(query_auth.valid(deps.api)?);
    }

    let mut messages = vec![
        register_receive(
            env.contract.code_hash.clone(),
            None,
            &state.lp_token.clone().into(),
        )?,
        set_viewing_key_msg(
            SHADE_STAKING_VIEWING_KEY.to_string(),
            None,
            &state.lp_token.clone().into(),
        )?,
    ];

    let mut response: Response = Response::new();
    response.data = Some(env.contract.address.as_bytes().into());

    let now = env.block.time.seconds();
    EPOCH_STORE.save(deps.storage, state.epoch_index, &EpochInfo {
        rewards_distribution: None,
        start_time: now,
        end_time: now + state.epoch_durations,
        duration: state.epoch_durations,
        reward_tokens: None,
        expired_at: None,
    })?;

    if let Some(reward_token_update) = msg.first_reward_token {
        let reward_token = reward_token_update.reward_token.valid(deps.api)?;

        let msgs =
            register_reward_tokens(deps.storage, vec![reward_token], env.contract.code_hash)?;
        response = response.add_messages(msgs);
    } else {
        store_empty_reward_set(deps.storage)?;
    }
    STATE.save(deps.storage, &state)?;

    Ok(response
        .add_messages(messages)
        .add_attributes(vec![Attribute::new(
            "staking_contract_addr",
            env.contract.address,
        )]))
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    pad_handle_result(
        match msg {
            ExecuteMsg::Snip1155Receive(msg) => {
                let checked_from = deps.api.addr_validate(&msg.from.as_str())?;
                receiver_callback_snip_1155(
                    deps,
                    env,
                    info,
                    checked_from,
                    msg.token_id,
                    msg.amount,
                    msg.msg,
                )
            }
            ExecuteMsg::Receive(msg) => {
                let checked_from = deps.api.addr_validate(&msg.from.as_str())?;
                receiver_callback(deps, env, info, checked_from, msg.amount, msg.msg)
            }
            ExecuteMsg::ClaimRewards {} => try_claim_rewards(deps, env, info),
            ExecuteMsg::Unstake { token_ids, amounts } => {
                try_unstake(deps, env, info, token_ids, amounts)
            }
            ExecuteMsg::UpdateRewardTokens(_) => todo!(),
            ExecuteMsg::RegisterRewardTokens(tokens) => {
                try_register_reward_tokens(deps, env, info, tokens)
            }
            ExecuteMsg::UpdateConfig {
                admin_auth,
                query_auth,
                padding,
            } => todo!(),
            ExecuteMsg::RecoverFunds {
                token,
                amount,
                to,
                msg,
                padding,
            } => todo!(),
            ExecuteMsg::EndEpoch {
                rewards_distribution,
            } => try_end_epoch(deps, env, info, rewards_distribution),
        },
        BLOCK_SIZE,
    )
}

fn receiver_callback_snip_1155(
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
    require_lp_token(&state, &info.sender)?;

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
            InvokeMsg::AddRewards { .. } => Err(StdError::generic_err("Wrong Receiver")),
        },
        BLOCK_SIZE,
    )
}

fn receiver_callback(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: Addr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<Response> {
    let msg = msg.ok_or_else(|| {
        StdError::generic_err("Receiver callback \"msg\" parameter cannot be empty.")
    })?;

    let state = STATE.load(deps.storage)?;
    require_lp_token(&state, &info.sender)?;

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
    state: State,
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

    //TODO: remove the id when removing liquidity
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

    Ok(Response::default())
}

pub fn try_unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token_ids: Vec<u32>,
    amounts: Vec<Uint256>,
) -> StdResult<Response> {
    //LOADING general use readonly stores
    let state = STATE.load(deps.storage)?;
    let epoch_obj = match EPOCH_STORE.may_load(deps.storage, state.epoch_index)? {
        Some(e) => Ok(e),
        None => Err(StdError::generic_err("Reward token storage already exists")),
    }?;
    //1) UPDATING: staker_info and staker_liquidity_snapshot
    //*INIT STAKER_INFO if not initialized already
    staker_init_checker(deps.storage, &state, &info.sender)?;
    let sender = &info.sender;
    let contract_addr = &env.contract;

    //TODO: check amounts and ids length

    let mut actions: Vec<TransferAction> = Vec::new();

    for (i, token_id) in token_ids.iter().enumerate() {
        //*LOADING readonly stores
        let mut staker_liq = STAKERS_LIQUIDITY
            .load(deps.storage, (&sender, *token_id))
            .unwrap_or_default();
        let mut staker_liq_snap = STAKERS_LIQUIDITY_SNAPSHOT
            .load(deps.storage, (&sender, state.epoch_index, *token_id))
            .unwrap_or_default();

        // Checking: If the amount unbonded is not greater than amount delegated
        if staker_liq.amount_delegated < amounts[i] {
            return Err(StdError::generic_err(format!(
                "insufficient funds to redeem: balance={}, required={}",
                staker_liq.amount_delegated, amounts[i]
            )));
        }

        if amounts[i].is_zero() {
            return Err(StdError::generic_err(format!(
                "cannot request to withdraw 0"
            )));
        }

        //**Fetching Liquidity
        let liquidity: Uint256;
        if staker_liq_snap.liquidity.is_zero() {
            liquidity = staker_liq.amount_delegated;
        } else {
            liquidity = staker_liq_snap.liquidity; //can panic here
        }

        //**Only adding to liquidity if round has not ended yet
        //**Only subtracting from liquidity if round has not ended yet
        if env.block.time.seconds() >= epoch_obj.end_time {
            staker_liq_snap.liquidity = liquidity;
        } else {
            // Calculate the amount to potentially subtract from liquidity
            let subtract_amount = amounts[i].multiply_ratio(
                if let Some(time) = epoch_obj.end_time.checked_sub(env.block.time.seconds()) {
                    time
                } else {
                    // If there's an overflow in calculating time, return an error
                    return Err(StdError::generic_err("Overflow in time calculation"));
                },
                epoch_obj.duration,
            );

            // Use checked_sub to prevent underflow when subtracting from liquidity
            staker_liq_snap.liquidity =
                if let Ok(new_liquidity) = liquidity.checked_sub(subtract_amount) {
                    new_liquidity
                } else {
                    // If there's an underflow in subtracting from liquidity, return an error
                    return Err(StdError::generic_err(
                        "Underflow in subtracting from liquidity",
                    ));
                };
        }

        if staker_liq.amount_delegated.is_zero() {
            STAKERS_BIN_TREE.update(deps.storage, &sender, |tree| -> StdResult<_> {
                if let Some(mut t) = tree {
                    t.remove(*token_id);
                    Ok(t)
                } else {
                    Ok(TreeUint24::new())
                }
            })?;
        }

        //*STORING user_info_obj and user_liquidity_snapshot
        // Use checked_sub to safely subtract amounts[i] from staker_liq.amount_delegated
        if let Ok(new_amount) = staker_liq.amount_delegated.checked_sub(amounts[i]) {
            staker_liq.amount_delegated = new_amount;
        } else {
            // Handle potential underflow error
            return Err(StdError::generic_err(
                "Underflow in subtracting from amount_delegated",
            ));
        }
        STAKERS_LIQUIDITY.save(deps.storage, (&sender, *token_id), &staker_liq)?;

        staker_liq_snap.amount_delegated = staker_liq.amount_delegated;
        STAKERS_LIQUIDITY_SNAPSHOT.save(
            deps.storage,
            (&sender, state.epoch_index, *token_id),
            &staker_liq_snap,
        )?;

        //2) UPDATING: PoolState & PoolStateLiquidityStats & Config
        //*LOADING readonly stores
        let mut total_liq = TOTAL_LIQUIDITY
            .load(deps.storage, *token_id)
            .unwrap_or_default();
        let mut total_liq_snap = TOTAL_LIQUIDITY_SNAPSHOT
            .load(deps.storage, (state.epoch_index, *token_id))
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
            let subtraction_amount = amounts[i].multiply_ratio(
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
        if let Ok(new_amount_delegated) = total_liq.amount_delegated.checked_sub(amounts[i]) {
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
        TOTAL_LIQUIDITY.save(deps.storage, *token_id, &total_liq)?;
        //*STORING pool_state_liquidity_snapshot
        TOTAL_LIQUIDITY_SNAPSHOT.save(
            deps.storage,
            (state.epoch_index, *token_id),
            &total_liq_snap,
        )?;

        actions.push(TransferAction {
            token_id: token_id.to_string(),
            from: contract_addr.address.clone(),
            recipient: sender.clone(),
            amount: amounts[i],
            memo: None,
        })
    }

    let message = shade_protocol::liquidity_book::lb_token::ExecuteMsg::BatchTransfer {
        actions,
        padding: None,
    }
    .to_cosmos_msg(
        state.lp_token.code_hash,
        state.lp_token.address.to_string(),
        None,
    )?;

    Ok(Response::default().add_message(message))
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
    //saves the distribution
    let mut epoch_obj = EPOCH_STORE.load(deps.storage, state.epoch_index)?;
    epoch_obj.rewards_distribution = Some(rewards_distribution);

    if let Some(expiry_duration) = epoch_obj.expired_at {
        epoch_obj.expired_at = Some(state.epoch_index + expiry_duration)
    }

    EPOCH_STORE.save(deps.storage, state.epoch_index, &epoch_obj)?;

    let mut reward_tokens = vec![];

    for reward_token in REWARD_TOKENS.load(deps.storage)? {
        let rewards_token_info = REWARD_TOKEN_INFO.load(deps.storage, &reward_token.address)?;

        // Create a filtered list of rewards to add to reward_tokens
        let rewards_to_add: Vec<_> = rewards_token_info
            .iter()
            .filter(|reward| reward.end > state.epoch_index)
            .cloned()
            .collect();

        // Extend reward_tokens with the filtered rewards
        reward_tokens.extend(rewards_to_add.clone());

        // Check if any rewards were removed, and save the updated list if so
        if rewards_to_add.len() != rewards_token_info.len() {
            REWARD_TOKEN_INFO.save(deps.storage, &reward_token.address, &rewards_to_add)?;
        }
    }

    let now = env.block.time.seconds();
    EPOCH_STORE.save(deps.storage, state.epoch_index.add(1), &EpochInfo {
        rewards_distribution: None,
        start_time: now,
        end_time: now + state.epoch_durations,
        duration: state.epoch_durations,
        reward_tokens: Some(reward_tokens),
        expired_at: None,
    })?;

    state.epoch_index.add_assign(1);

    STATE.save(deps.storage, &state)?;

    Ok(Response::default())
}

pub fn try_claim_rewards(deps: DepsMut, _env: Env, info: MessageInfo) -> StdResult<Response> {
    let state = STATE.load(deps.storage)?;
    let staker_result = STAKERS.load(deps.storage, &info.sender);
    let mut messages = Vec::new();

    // Check if staker exists
    let staker = match staker_result {
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
        let epoch_info = EPOCH_STORE.load(deps.storage, round_epoch)?;

        // Skip epochs with no reward tokens or expired rewards
        if epoch_info
            .reward_tokens
            .as_ref()
            .map_or(true, Vec::is_empty)
            || epoch_info
                .expired_at
                .map_or(false, |expired_at| expired_at < round_epoch)
        {
            continue;
        }

        // Process reward distribution
        if let Some(rewards_distribution) = &epoch_info.rewards_distribution {
            for (i, dis) in rewards_distribution.ids.iter().enumerate() {
                let staker_liq =
                    finding_user_liquidity(deps.storage, &info, &staker, state.epoch_index, *dis)?;

                let total_liquidity =
                    finding_total_liquidity(deps.storage, state.epoch_index, *dis)?;

                // Calculate and distribute rewards
                if let Some(reward_tokens) = &epoch_info.reward_tokens {
                    for x in reward_tokens.iter() {
                        let total_bin_rewards = Uint256::from(x.reward_per_epoch.multiply_ratio(
                            rewards_distribution.weightages[i],
                            rewards_distribution.denominator,
                        ));

                        let staker_rewards = Uint128::from_str(
                            &total_bin_rewards
                                .multiply_ratio(staker_liq, total_liquidity)
                                .to_string(),
                        )?;

                        messages.push(
                            Snip20ExecuteMsg::Send {
                                recipient: info.sender.to_string(),
                                recipient_code_hash: None,
                                amount: staker_rewards,
                                msg: None,
                                memo: None,
                                padding: None,
                            }
                            .to_cosmos_msg(&x.token, vec![])?,
                        );
                    }
                }
            }
        }
    }

    Ok(Response::default().add_messages(messages))
}

pub fn try_register_reward_tokens(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    tokens: Vec<ContractInfo>,
) -> StdResult<Response> {
    let state = STATE.load(deps.storage)?;
    //check that only be called by lp-pair
    //TODO: assert_admin

    validate_admin(
        &deps.querier,
        AdminPermissions::StakingAdmin,
        info.sender.to_string(),
        &state.admin_auth.into(),
    )?;

    let msgs = register_reward_tokens(deps.storage, tokens, env.contract.code_hash)?;

    Ok(Response::default().add_messages(msgs))
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
        if start >= end {
            return Err(StdError::generic_err("'start' must be after 'end'"));
        }
        // Disallow retro-active emissions (maybe could allow?)
        if start < state.epoch_index {
            return Err(StdError::generic_err("Cannot start emitting in the past"));
        }

        validate_admin(
            &deps.querier,
            AdminPermissions::StakingAdmin,
            info.sender.to_string(),
            &state.admin_auth.into(),
        )?;
        let decimals = token_info(&deps.querier, &Contract {
            address: token.address.clone(),
            code_hash: token.code_hash.clone(),
        })?
        .decimals;

        let total_epoches = end.sub(start);

        let reward_per_epoch = amount.multiply_ratio(Uint128::one(), total_epoches);

        let rewards_info_obj = RewardTokenInfo {
            token: token.clone(),
            decimals,
            reward_per_epoch,
            start,
            end,
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

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    Ok(to_binary("lmao")?)
}
