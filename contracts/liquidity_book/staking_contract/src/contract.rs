use std::{
    ops::{Add, AddAssign},
    str::FromStr,
};

use shade_protocol::{
    c_std::{
        entry_point,
        from_binary,
        to_binary,
        Addr,
        Attribute,
        Binary,
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
    liquidity_book::{
        lb_pair::RewardsDistribution,
        staking::{EpochInfo, InvokeMsg},
    },
    snip20::{helpers::token_info, ExecuteMsg as Snip20ExecuteMsg},
    swap::staking::{ExecuteMsg, InstantiateMsg, QueryMsg, State},
    utils::{pad_handle_result, ExecuteCallback},
    BLOCK_SIZE,
};

use crate::{
    helper::{
        assert_lb_pair,
        check_if_claimable,
        create_reward_token,
        finding_total_liquidity,
        finding_user_liquidity,
        require_lp_token,
        staker_init_checker,
        store_empty_reward_set,
    },
    state::{
        EPOCH_STORE,
        REWARD_TOKENS,
        REWARD_TOKEN_INFO,
        STAKERS,
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
        lp_token: msg.lb_token.into_valid(deps.api)?,
        lb_pair: deps.api.addr_validate(&msg.amm_pair)?,
        admin_auth: msg.admin_auth.into_valid(deps.api)?,
        query_auth: None,
        total_amount_staked: Uint128::zero(),
        epoch_index: msg.epoch_index,
        epoch_durations: msg.epoch_duration,
        expiry_durations: msg.expiry_duration,
    };

    if let Some(query_auth) = msg.query_auth {
        state.query_auth = Some(query_auth.into_valid(deps.api)?);
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
        let reward_token = reward_token_update.reward_token.into_valid(deps.api)?;
        messages.push(set_viewing_key_msg(
            SHADE_STAKING_VIEWING_KEY.to_string(),
            None,
            &reward_token.clone().into(),
        )?);

        // store reward token to the list
        let now_epoch_index = state.epoch_index;
        let decimals = token_info(&deps.querier, &reward_token)?.decimals;

        let info = create_reward_token(
            deps.storage,
            now_epoch_index,
            &reward_token,
            reward_token_update.daily_reward_amount,
            reward_token_update.valid_to,
            decimals,
        )?;

        let reward_addr = if let Some(info) = info.get(0) {
            &info.token.address
        } else {
            return Err(StdError::generic_err(
                "Creation of initial reward config failed",
            ));
        };
        response = response.add_attributes(vec![
            Attribute::new("reward_token", reward_addr),
            Attribute::new(
                "daily_reward_amount",
                reward_token_update.daily_reward_amount,
            ),
        ]);
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
                receiver_callback(
                    deps,
                    env,
                    info,
                    checked_from,
                    msg.token_id,
                    msg.amount,
                    msg.msg,
                )
            }
            ExecuteMsg::ClaimRewards {} => claim_rewards(deps, env, info),
            ExecuteMsg::Unstake {
                amount,
                remove_liquidity,
                padding,
            } => todo!(),
            ExecuteMsg::UpdateRewardTokens(_) => todo!(),
            ExecuteMsg::CreateRewardTokens(_) => todo!(),
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
            } => end_epoch(deps, env, info, rewards_distribution),
        },
        BLOCK_SIZE,
    )
}

fn receiver_callback(
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

pub fn end_epoch(
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

    for reward_token in REWARD_TOKENS.load(deps.storage)?.get() {
        let reward_info = REWARD_TOKEN_INFO.load(deps.storage, reward_token)?;
        for reward in reward_info {
            if reward.valid_to <= state.epoch_index {
                reward_tokens.push(reward)
            }
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

pub fn claim_rewards(deps: DepsMut, _env: Env, info: MessageInfo) -> StdResult<Response> {
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

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    Ok(to_binary("lmao")?)
}
