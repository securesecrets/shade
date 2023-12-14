use std::{
    collections::HashMap,
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
        Storage,
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
            ExecuteAnswer,
            ExecuteMsg,
            InstantiateMsg,
            InvokeMsg,
            Liquidity,
            OwnerBalance,
            QueryAnswer,
            QueryMsg,
            QueryWithPermit,
            RewardTokenInfo,
            StakerLiquidity,
            StakerLiquiditySnapshot,
            State,
        },
    },
    s_toolkit::{
        permit::{validate, Permit, RevokedPermits, TokenPermissions},
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
pub const PREFIX_REVOKED_PERMITS: &str = "revoked_permits";

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = State {
        lb_token: msg.lb_token.valid(deps.api)?,
        lb_pair: deps.api.addr_validate(&msg.amm_pair)?,
        admin_auth: msg.admin_auth.valid(deps.api)?,
        query_auth: msg
            .query_auth
            .map(|auth| auth.valid(deps.api))
            .transpose()?,
        epoch_index: msg.epoch_index,
        epoch_durations: msg.epoch_duration,
        expiry_durations: msg.expiry_duration,
    };

    let now = env.block.time.seconds();
    EPOCH_STORE.save(deps.storage, state.epoch_index, &EpochInfo {
        rewards_distribution: None,
        start_time: now,
        end_time: now + state.epoch_durations,
        duration: state.epoch_durations,
        reward_tokens: None,
        expired_at: None,
    })?;

    let messages = vec![
        register_receive(
            env.contract.code_hash.clone(),
            None,
            &state.lb_token.clone().into(),
        )?,
        set_viewing_key_msg(
            SHADE_STAKING_VIEWING_KEY.to_string(),
            None,
            &state.lb_token.clone().into(),
        )?,
    ];

    let mut response: Response = Response::new();
    response.data = Some(env.contract.address.as_bytes().into());

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
                receiver_callback(deps, info, checked_from, msg.amount, msg.msg)
            }
            ExecuteMsg::ClaimRewards {} => try_claim_rewards(deps, env, info),
            ExecuteMsg::Unstake { token_ids, amounts } => {
                try_unstake(deps, env, info, token_ids, amounts)
            }
            ExecuteMsg::RegisterRewardTokens(tokens) => {
                try_register_reward_tokens(deps, env, info, tokens)
            }
            ExecuteMsg::UpdateConfig {
                admin_auth,
                query_auth,
                epoch_duration,
                expiry_duration,
            } => try_update_config(
                deps,
                info,
                admin_auth,
                query_auth,
                epoch_duration,
                expiry_duration,
            ),
            ExecuteMsg::RecoverFunds { .. } => todo!(), //TODO
            ExecuteMsg::EndEpoch {
                rewards_distribution,
            } => try_end_epoch(deps, env, info, rewards_distribution),

            ExecuteMsg::CreateViewingKey { entropy } => {
                try_create_viewing_key(deps, env, info, entropy)
            }
            ExecuteMsg::SetViewingKey { key } => try_set_viewing_key(deps, env, info, key),
            ExecuteMsg::RevokePermit { permit_name } => {
                try_revoke_permit(deps, env, info, permit_name)
            }
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

fn receiver_callback(
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
    if amounts.len() != token_ids.len() {
        return Err(StdError::generic_err(
            "ids array not equal to amounts array",
        ));
    }

    let state = STATE.load(deps.storage)?;
    let epoch_obj = EPOCH_STORE
        .may_load(deps.storage, state.epoch_index)?
        .ok_or_else(|| StdError::generic_err("Reward token storage does not exist"))?;

    staker_init_checker(deps.storage, &state, &info.sender)?;

    let mut actions: Vec<TransferAction> = Vec::new();

    for (token_id, amount) in token_ids.into_iter().zip(amounts.into_iter()) {
        let mut staker_liq = STAKERS_LIQUIDITY
            .load(deps.storage, (&info.sender, token_id))
            .unwrap_or_default();
        let mut staker_liq_snap = STAKERS_LIQUIDITY_SNAPSHOT
            .load(deps.storage, (&info.sender, state.epoch_index, token_id))
            .unwrap_or_default();

        if amount.is_zero() {
            return Err(StdError::generic_err("cannot request to withdraw 0"));
        }

        if staker_liq.amount_delegated < amount {
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
            token_id,
            amount,
            &mut staker_liq,
            &mut staker_liq_snap,
        )?;

        actions.push(TransferAction {
            token_id: token_id.to_string(),
            from: env.contract.address.clone(),
            recipient: info.sender.clone(),
            amount,
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

    //saves the distribution
    let mut epoch_obj = EPOCH_STORE.load(deps.storage, state.epoch_index)?;
    epoch_obj.rewards_distribution = Some(rewards_distribution);
    epoch_obj.reward_tokens = Some(reward_tokens);

    if let Some(expiry_duration) = epoch_obj.expired_at {
        epoch_obj.expired_at = Some(state.epoch_index + expiry_duration)
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

pub fn try_claim_rewards(deps: DepsMut, _env: Env, info: MessageInfo) -> StdResult<Response> {
    let state = STATE.load(deps.storage)?;
    let staker_result = STAKERS.load(deps.storage, &info.sender);
    let mut messages = Vec::new();

    let mut rewards_accumulator: HashMap<TokenKey, Uint128> = HashMap::new();

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
                let staker_liq_snap =
                    finding_user_liquidity(deps.storage, &info, &staker, round_epoch, *dis)?;
                if staker_liq_snap.liquidity.is_zero() {
                    continue;
                }
                STAKERS_LIQUIDITY_SNAPSHOT.save(
                    deps.storage,
                    (&info.sender, round_epoch, *dis),
                    &staker_liq_snap,
                )?;

                let total_liquidity_snap =
                    finding_total_liquidity(deps.storage, round_epoch, *dis)?;

                if total_liquidity_snap.liquidity.is_zero() {
                    continue;
                }

                TOTAL_LIQUIDITY_SNAPSHOT.save(
                    deps.storage,
                    (round_epoch, *dis),
                    &total_liquidity_snap,
                )?;

                // Calculate and distribute rewards
                if let Some(reward_tokens) = &epoch_info.reward_tokens {
                    for x in reward_tokens.iter() {
                        let total_bin_rewards = Uint256::from(x.reward_per_epoch.multiply_ratio(
                            rewards_distribution.weightages[i],
                            rewards_distribution.denominator,
                        ));

                        let staker_rewards = Uint128::from_str(
                            &total_bin_rewards
                                .multiply_ratio(
                                    staker_liq_snap.liquidity,
                                    total_liquidity_snap.liquidity,
                                )
                                .to_string(),
                        )?;

                        let token_key = TokenKey {
                            address: x.token.address.clone(),
                            code_hash: x.token.code_hash.clone(),
                        };
                        let entry = rewards_accumulator
                            .entry(token_key)
                            .or_insert(Uint128::zero());
                        *entry += staker_rewards;
                    }
                }
            }
        }
    }

    // Create messages for each token with the accumulated rewards
    for (token_key, amount) in rewards_accumulator {
        let contract_info = ContractInfo {
            address: token_key.address,
            code_hash: token_key.code_hash,
        };
        messages.push(
            Snip20ExecuteMsg::Send {
                recipient: info.sender.to_string(),
                recipient_code_hash: None,
                amount,
                msg: None,
                memo: None,
                padding: None,
            }
            .to_cosmos_msg(&contract_info, vec![])?,
        );
    }

    //TODO: add user rewards somewhere

    Ok(Response::default().add_messages(messages))
}

use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
struct TokenKey {
    address: Addr,
    code_hash: String,
}

impl PartialEq for TokenKey {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address && self.code_hash == other.code_hash
    }
}

impl Eq for TokenKey {}

impl Hash for TokenKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
        self.code_hash.hash(state);
    }
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
        if start >= end {
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

fn try_create_viewing_key(
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

fn try_set_viewing_key(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    key: String,
) -> StdResult<Response> {
    ViewingKey::set(deps.storage, info.sender.as_str(), key.as_str());
    Ok(Response::new())
}

fn try_revoke_permit(
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

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ContractInfo {} => query_contract_info(deps),
        QueryMsg::RegisteredTokens {} => query_registered_tokens(deps),
        QueryMsg::IdTotalBalance { id } => query_token_id_balance(deps, id),
        QueryMsg::Balance { .. }
        | QueryMsg::AllBalances { .. }
        | QueryMsg::Liquidity { .. }
        | QueryMsg::TransactionHistory { .. } => viewing_keys_queries(deps, msg),

        QueryMsg::WithPermit { permit, query } => permit_queries(deps, env, permit, query),
    }
}

fn query_contract_info(deps: Deps) -> StdResult<Binary> {
    let state: State = STATE.load(deps.storage)?;

    let response = QueryAnswer::ContractInfo {
        lb_token: state.lb_token,
        lb_pair: state.lb_pair,
        admin_auth: state.admin_auth,
        query_auth: state.query_auth,
        epoch_index: state.epoch_index,
        epoch_durations: state.epoch_durations,
        expiry_durations: state.expiry_durations,
    };
    to_binary(&response)
}

fn query_registered_tokens(deps: Deps) -> StdResult<Binary> {
    let reg_tokens = REWARD_TOKENS.load(deps.storage)?;

    let response = QueryAnswer::RegisteredTokens(reg_tokens);
    to_binary(&response)
}

fn query_token_id_balance(deps: Deps, token_id: String) -> StdResult<Binary> {
    let id = u32::from_str(&token_id)
        .map_err(|_| StdError::generic_err(format!("token_id {} cannot be parsed", token_id)))?;

    let liquidity = TOTAL_LIQUIDITY
        .load(deps.storage, id)
        .map_err(|_| StdError::generic_err(format!("token_id {} does not exist", token_id)))?;

    let response = QueryAnswer::IdTotalBalance {
        amount: liquidity.amount_delegated,
    };
    to_binary(&response)
}

fn viewing_keys_queries(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
    let (addresses, key) = msg.get_validation_params()?;

    for address in addresses {
        let result = ViewingKey::check(deps.storage, address.as_str(), key.as_str());
        if result.is_ok() {
            return match msg {
                QueryMsg::Balance {
                    owner, token_id, ..
                } => query_balance(deps, &owner, token_id),

                QueryMsg::AllBalances {
                    owner,
                    page,
                    page_size,
                    ..
                } => query_all_balances(deps, &owner, page, page_size),

                QueryMsg::Liquidity {
                    owner,
                    round_index,
                    token_ids,
                    ..
                } => query_liquidity(deps, &owner, token_ids, round_index),

                QueryMsg::WithPermit { .. } => {
                    unreachable!("This query type does not require viewing key authentication")
                }
                _ => unreachable!("This query type does not require viewing key authentication"),
            };
        }
    }

    to_binary(&QueryAnswer::ViewingKeyError {
        msg: "Wrong viewing key for this address or viewing key not set".to_string(),
    })
}

fn query_balance(deps: Deps, owner: &Addr, token_id: String) -> StdResult<Binary> {
    let id = u32::from_str(&token_id)
        .map_err(|_| StdError::generic_err(format!("token_id {} cannot be parsed", token_id)))?;

    let liquidity = STAKERS_LIQUIDITY
        .load(deps.storage, (owner, id))
        .map_err(|_| StdError::generic_err(format!("token_id {} does not exist", token_id)))?;

    let response = QueryAnswer::Balance {
        amount: liquidity.amount_delegated,
    };
    to_binary(&response)
}

fn query_all_balances(
    deps: Deps,
    owner: &Addr,
    page: Option<u32>,
    page_size: Option<u32>,
) -> StdResult<Binary> {
    let page = page.unwrap_or(0u32);
    let page_size = page_size.unwrap_or(50u32);
    let tree = STAKERS_BIN_TREE.load(deps.storage, owner)?;

    let mut token_id = 0u32;
    for _ in 0..(page * page_size) {
        token_id = tree.find_first_left(token_id); //skipping there bins_id 
    }

    let mut balances = Vec::new();

    for _ in 0..(page * page_size) {
        let liquidity = STAKERS_LIQUIDITY
            .load(deps.storage, (owner, token_id))
            .map_err(|_| StdError::generic_err(format!("token_id {} does not exist", token_id)))?;
        token_id = tree.find_first_left(token_id);
        balances.push(OwnerBalance {
            token_id: token_id.to_string(),
            amount: liquidity.amount_delegated,
        })
    }

    let response = QueryAnswer::AllBalances(balances);
    to_binary(&response)
}

pub fn query_liquidity(
    deps: Deps,
    owner: &Addr,
    token_ids: Vec<u32>,
    round_index: Option<u64>,
) -> StdResult<Binary> {
    let state = STATE.load(deps.storage)?;
    let staker_result = STAKERS.load(deps.storage, &owner);
    let mut liquidity_response = Vec::new();
    // Check if staker exists
    let staker = match staker_result {
        Ok(staker) => staker,
        Err(_) => {
            return Err(StdError::generic_err(format!(
                "No staker found for address {:?}.",
                owner
            )));
        }
    };

    for token_id in token_ids {
        let staker_liq_snap = STAKERS_LIQUIDITY_SNAPSHOT.load(
            deps.storage,
            (&owner, round_index.unwrap_or(state.epoch_index), token_id),
        )?;
        let total_liq_snap = TOTAL_LIQUIDITY_SNAPSHOT.load(
            deps.storage,
            (round_index.unwrap_or(state.epoch_index), token_id),
        )?;

        //Calculating User Liquidity to generate n tickets for the round
        let liquidity_current_round: Uint256;
        let mut legacy_bal = Uint256::zero();
        let total_liq;

        // Total Liquidity
        if total_liq_snap.liquidity.is_zero() {
            let total_liquidity = TOTAL_LIQUIDITY.load(deps.storage, token_id)?;
            total_liq = total_liquidity.amount_delegated;
        } else {
            total_liq = total_liq_snap.liquidity;
        }

        if !staker_liq_snap.liquidity.is_zero() {
            liquidity_current_round = staker_liq_snap.liquidity;
        } else {
            let mut finding_liq_round: u64 =
                if let Some(r_i) = round_index.unwrap_or(state.epoch_index).checked_sub(1) {
                    r_i
                } else {
                    return Err(StdError::generic_err("Under-flow sub error"));
                };

            let start = if staker.last_claim_rewards_round.is_some() {
                staker.last_claim_rewards_round.unwrap()
            } else {
                if staker.starting_round.is_some() {
                    staker.starting_round.unwrap()
                } else {
                    liquidity_response.push(Liquidity {
                        token_id: token_id.to_string(),
                        user_liquidity: Uint256::zero(),
                        total_liquidity: total_liq,
                    });

                    let response = QueryAnswer::Liquidity(liquidity_response);

                    return to_binary(&response);
                }
            };

            while finding_liq_round >= start {
                let staker_liq_snap_prev = STAKERS_LIQUIDITY_SNAPSHOT
                    .load(deps.storage, (&owner, finding_liq_round, token_id))?;

                if !staker_liq_snap_prev.amount_delegated.is_zero() {
                    legacy_bal = staker_liq_snap_prev.amount_delegated;
                    break;
                } else {
                    finding_liq_round = if let Some(f_liq) = finding_liq_round.checked_sub(1) {
                        f_liq
                    } else {
                        return Err(StdError::generic_err("Under-flow sub error"));
                    }
                }
            }

            liquidity_current_round = legacy_bal;
        }
        liquidity_response.push(Liquidity {
            token_id: token_id.to_string(),
            user_liquidity: liquidity_current_round,
            total_liquidity: total_liq,
        });
    }

    let response = QueryAnswer::Liquidity(liquidity_response);

    to_binary(&response)
}

fn permit_queries(
    deps: Deps,
    env: Env,
    permit: Permit,
    query: QueryWithPermit,
) -> Result<Binary, StdError> {
    // Validate permit content
    let contract_address = env.contract.address;
    let account_str = validate(
        deps,
        PREFIX_REVOKED_PERMITS,
        &permit,
        contract_address.to_string(),
        None,
    )?;
    let account = deps.api.addr_validate(&account_str)?;

    let is_owner = permit.check_permission(&TokenPermissions::Owner);

    // Permit validated! We can now execute the query.
    match query {
        QueryWithPermit::Balance { owner, token_id } => {
            if !is_owner {
                if !permit.check_permission(&TokenPermissions::Balance) {
                    return Err(StdError::generic_err(format!(
                        "`Owner` or `Balance` permit required for permit queries, got permissions {:?}",
                        permit.params.permissions
                    )));
                }
            }
            query_balance(deps, &owner, token_id)
        }
        QueryWithPermit::AllBalances { page, page_size } => {
            if !is_owner {
                if !permit.check_permission(&TokenPermissions::Balance) {
                    return Err(StdError::generic_err(format!(
                        "`Owner` or `Balance` permit required for permit queries, got permissions {:?}",
                        permit.params.permissions
                    )));
                }
            }
            query_all_balances(deps, &account, page, page_size)
        }
        _ => unreachable!("This query type does not require viewing key authentication"),
    }
}
