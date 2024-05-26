use std::ops::Add;

use crate::{
    msg::{
        status_level_to_u8, Config, ContractStatusLevel, ExecuteAnswer, ExecuteMsg,
        InProcessUnbonding, InstantiateMsg, PanicUnbond, QueryAnswer, QueryMsg, QueryWithPermit,
        ReceiverMsg, ResponseStatus::Success,
    },
    staking_interface::{transfer_staked_msg, Reward, Rewards, Token},
    state::{ContractsVksStore, REWARDED_TOKENS_LIST},
};

#[allow(unused_imports)]
use crate::staking_interface::{
    balance_query as staking_balance_query, claim_rewards_msg, compound_msg, config_query,
    rewards_query, unbond_msg, withdraw_msg, Action, RawContract, StakingConfig, UnbondResponse,
    Unbonding, WithdrawResponse,
};

use crate::state::{
    UnbondingIdsStore, UnbondingStore, CONFIG, CONTRACT_STATUS, PANIC_UNBONDS,
    PANIC_UNBOND_REPLY_ID, PANIC_WITHDRAW_REPLY_ID, PENDING_UNBONDING, RESPONSE_BLOCK_SIZE,
    UNBOND_REPLY_ID,
};
/// This contract implements SNIP-20 standard:
/// https://github.com/SecretFoundation/SNIPs/blob/master/SNIP-20.md
use cosmwasm_std::{
    entry_point, from_binary, to_binary, Addr, Binary, CosmosMsg, CustomQuery, Deps, DepsMut, Env,
    MessageInfo, QuerierWrapper, Reply, Response, StdError, StdResult, Storage, SubMsg,
    SubMsgResult, Uint128, Uint256,
};

#[allow(unused_imports)]
use secret_toolkit::{
    snip20::{
        balance_query, burn_msg, mint_msg, register_receive_msg, send_msg, set_viewing_key_msg,
        token_info_query, TokenInfo,
    },
    utils::{pad_handle_result, pad_query_result},
};

use secret_toolkit_crypto::{sha_256, ContractPrng};
use serde::de::DeserializeOwned;
use shade_protocol::query_auth::QueryPermit;

use crate::msg::{Fee, FeeInfo};
#[allow(unused_imports)]
use shade_protocol::{
    admin::{
        helpers::{validate_admin, AdminPermissions},
        ConfigResponse, QueryMsg as AdminQueryMsg,
    },
    query_auth::helpers::{authenticate_permit, authenticate_vk},
    utils::Query,
    Contract,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let token_info = get_token_info(
        deps.querier,
        RESPONSE_BLOCK_SIZE,
        msg.token.code_hash.clone(),
        msg.token.address.to_string(),
        false,
    )?;
    let derivative_info = get_token_info(
        deps.querier,
        RESPONSE_BLOCK_SIZE,
        msg.derivative.code_hash.clone(),
        msg.derivative.address.to_string(),
        true,
    )?;

    if token_info.decimals != derivative_info.decimals {
        return Err(StdError::generic_err(
            "Derivative and token contracts should have the same amount of decimals",
        ));
    }
    // Generate viewing key for staking contract
    let entropy: String = msg
        .staking
        .entropy
        .clone()
        .unwrap_or_else(|| msg.prng_seed.to_string());
    let (staking_contract_vk, new_seed) =
        new_viewing_key(&info.sender, &env, &msg.prng_seed.0, entropy.as_ref());

    // Generate viewing key for SHD contract
    let entropy: String = msg
        .token
        .entropy
        .clone()
        .unwrap_or_else(|| msg.prng_seed.to_string());
    let (token_contract_vk, _new_seed) =
        new_viewing_key(&info.sender, &env, &new_seed, entropy.as_ref());

    CONFIG.save(
        deps.storage,
        &Config {
            prng_seed: msg.prng_seed,
            staking_contract_vk: staking_contract_vk.clone(),
            token_contract_vk: token_contract_vk.clone(),
            query_auth: msg.query_auth.clone(),
            token: msg.token.clone(),
            derivative: msg.derivative.clone(),
            staking: msg.staking,
            fees: msg.fees,
            contract_address: env.contract.address.clone(),
            admin: msg.admin,
        },
    )?;
    CONTRACT_STATUS.save(deps.storage, &ContractStatusLevel::NormalRun)?;

    let msgs: Vec<CosmosMsg> = vec![
        // Register receive Derivative contract needed for Unbond functionality
        register_receive_msg(
            env.contract.code_hash.clone(),
            msg.derivative.entropy.clone(),
            RESPONSE_BLOCK_SIZE,
            msg.derivative.code_hash.clone(),
            msg.derivative.address.to_string(),
        )?,
        // Register receive SHD contract
        register_receive_msg(
            env.contract.code_hash,
            msg.token.entropy.clone(),
            RESPONSE_BLOCK_SIZE,
            msg.token.code_hash.clone(),
            msg.token.address.to_string(),
        )?,
        // Set viewing key for SHD
        set_viewing_key_msg(
            token_contract_vk,
            msg.token.entropy,
            RESPONSE_BLOCK_SIZE,
            msg.token.code_hash,
            msg.token.address.to_string(),
        )?,
        // Set viewing key for staking contract
        set_viewing_key_msg(
            staking_contract_vk,
            msg.query_auth.entropy,
            RESPONSE_BLOCK_SIZE,
            msg.query_auth.code_hash,
            msg.query_auth.address.to_string(),
        )?,
    ];

    Ok(Response::default().add_messages(msgs))
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    let response = match msg {
        // Messages always available
        ExecuteMsg::SetContractStatus { level, .. } => {
            set_contract_status(deps, info, level, ContractStatusLevel::StopAll)
        }
        // Messages available during panic mode
        ExecuteMsg::Claim {} => try_claim(deps, env, info, ContractStatusLevel::Panicked),
        ExecuteMsg::PanicUnbond { amount } => {
            try_panic_unbond(env, deps, info, amount, ContractStatusLevel::Panicked)
        }
        ExecuteMsg::PanicWithdraw {} => {
            try_panic_withdraw(deps, env, info, ContractStatusLevel::Panicked)
        }
        ExecuteMsg::UpdateFees {
            staking,
            unbonding,
            collector,
        } => update_fees(
            deps,
            info,
            staking,
            unbonding,
            collector,
            ContractStatusLevel::Panicked,
        ),

        // Messages available when status is normal
        ExecuteMsg::Receive {
            sender: _,
            from,
            amount,
            msg,
        } => receive(deps, env, info, from, amount, msg),
        ExecuteMsg::CompoundRewards {} => {
            try_compound_rewards(deps, ContractStatusLevel::NormalRun)
        }
    };

    pad_handle_result(response, RESPONSE_BLOCK_SIZE)
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    pad_query_result(
        match msg {
            QueryMsg::StakingInfo {} => query_staking_info(&deps, &env),
            QueryMsg::FeeInfo {} => query_fee_info(&deps),
            QueryMsg::ContractStatus {} => query_contract_status(deps.storage),
            QueryMsg::WithPermit { permit } => permit_queries(deps, &env, permit),
            _ => viewing_keys_queries(deps, &env, msg),
        },
        RESPONSE_BLOCK_SIZE,
    )
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match (msg.id, msg.result) {
        (UNBOND_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(x) => {
                let result: UnbondResponse = from_binary(&x)?;
                // Unbonding stored in try_unbond function
                // Because of here you can't access the sender of the TX this was stored previously
                let pending_unbonding = PENDING_UNBONDING.may_load(deps.storage)?;

                if let Some(unbonding_processing) = pending_unbonding {
                    // Set properly id for this unbonding
                    let unbond = Unbonding {
                        id: result.unbond.id,
                        amount: unbonding_processing.amount,
                        complete: unbonding_processing.complete,
                    };
                    UnbondingStore::save(deps.storage, result.unbond.id.clone().u128(), &unbond)?;

                    // Add unbonding id to user's unbondings IDs
                    let mut users_unbondings_ids =
                        UnbondingIdsStore::load(deps.storage, &unbonding_processing.owner);
                    users_unbondings_ids.push(result.unbond.id.u128());
                    UnbondingIdsStore::save(
                        deps.storage,
                        &unbonding_processing.owner,
                        users_unbondings_ids,
                    )?;

                    Ok(Response::default())
                } else {
                    Err(StdError::generic_err(
                        "Unexpected error: pending unbond storage is empty.",
                    ))
                }
            }
            None => Err(StdError::generic_err("Unknown reply id")),
        },

        (PANIC_WITHDRAW_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(x) => {
                let config = CONFIG.load(deps.storage)?;
                let result: WithdrawResponse = from_binary(&x)?;
                let withdrawn = result.withdraw.withdrawn;
                let addr = get_super_admin(&deps.querier, &config)?;

                Ok(Response::default().add_message(send_msg(
                    addr.to_string(),
                    withdrawn,
                    None,
                    Some("Panic withdraw {} tokens".to_string()),
                    config.token.entropy,
                    RESPONSE_BLOCK_SIZE,
                    config.token.code_hash,
                    config.token.address.to_string(),
                )?))
            }
            None => Err(StdError::generic_err("Unknown reply id")),
        },

        (PANIC_UNBOND_REPLY_ID, SubMsgResult::Ok(s)) => match s.data {
            Some(x) => {
                let result: UnbondResponse = from_binary(&x)?;
                let mut panic_unbonds = PANIC_UNBONDS.may_load(deps.storage)?.unwrap_or_default();
                let last = panic_unbonds.pop();

                // Validate there is at least 1 element is storage to update
                // should never happen but you never know
                if let Some(mut last_unbond) = last {
                    // Update latest panic unbond id
                    last_unbond.id = result.unbond.id;
                    panic_unbonds.push(last_unbond);
                    //Save list of panic unbonds in storage
                    PANIC_UNBONDS.save(deps.storage, &panic_unbonds)?;
                }
                Ok(Response::default())
            }
            None => Err(StdError::generic_err("Unknown reply id")),
        },
        _ => Err(StdError::generic_err("Unknown reply id")),
    }
}
/************ HANDLES ************/
/// It takes a list of unbonding ids, and if they are mature, it removes them from storage and sends the
/// tokens to the user
///
/// Arguments:
///
/// * `deps`: DepsMut - This is the dependency struct that contains all the dependencies that the
/// handler needs.
/// * `env`: Env - This is the environment that the transaction is being executed in. It contains
/// information about the block, the transaction, and the message.
/// * `info`: MessageInfo - contains the sender of the message, the sent amount, and the sent memo
///
/// Returns:
///
/// StdResult<Response>
fn try_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    priority: ContractStatusLevel,
) -> StdResult<Response> {
    check_status(deps.storage, priority)?;
    let sender = info.sender;
    let time = Uint128::from(env.block.time.seconds());
    let user_unbondings_ids = UnbondingIdsStore::load(deps.storage, &sender);
    let config = CONFIG.load(deps.storage)?;
    let mut to_claim_ids: Vec<u128> = vec![];
    let mut amount_claimed = Uint128::zero();

    for id in user_unbondings_ids.iter() {
        let opt_unbonding = UnbondingStore::may_load(deps.storage, *id);
        if let Some(unbonding) = opt_unbonding {
            // Handle mature unbondings
            if time >= unbonding.complete {
                to_claim_ids.push(unbonding.id.u128());
                amount_claimed += unbonding.amount;

                // Remove unbonding from storage
                UnbondingStore::remove(deps.storage, *id)?;
            }
        }
    }
    if to_claim_ids.is_empty() {
        return Err(StdError::generic_err("No mature unbondings to claim"));
    }
    let (fee, deposit) = get_fee(amount_claimed, &config.fees.unbonding)?;

    let users_new_pending_unbondings: Vec<u128> = user_unbondings_ids
        .into_iter()
        .filter(|id| !to_claim_ids.contains(id))
        .collect();
    UnbondingIdsStore::save(deps.storage, &sender, users_new_pending_unbondings)?;

    let to_claim_ids_uint128: Vec<Uint128> = to_claim_ids.into_iter().map(Uint128::from).collect();

    let config: Config = CONFIG.load(deps.storage)?;
    let messages: Vec<CosmosMsg> = vec![
        withdraw_msg(
            config.staking.code_hash,
            config.staking.address.to_string(),
            Some(to_claim_ids_uint128),
        )?,
        send_msg(
            config.fees.collector.to_string(),
            fee,
            None,
            Some(base64::encode(&"Payment of fee for unbonding SHD")),
            config.token.entropy.clone(),
            RESPONSE_BLOCK_SIZE,
            config.token.code_hash.clone(),
            config.token.address.to_string(),
        )?,
        send_msg(
            sender.to_string(),
            deposit,
            None,
            Some(format!("Claiming {} SHD tokens", { deposit })),
            config.token.entropy,
            RESPONSE_BLOCK_SIZE,
            config.token.code_hash,
            config.token.address.to_string(),
        )?,
    ];

    Ok(Response::default()
        .add_messages(messages)
        .set_data(to_binary(&ExecuteAnswer::Claim {
            amount_claimed: deposit,
        })?))
}

/// It creates a `compound_msg` and returns it as a `Response`
///
/// Arguments:
///
/// * `deps`: DepsMut - This is the dependencies object that contains the storage, querier, and other
/// useful things.
///
/// Returns:
///
/// StdResult<Response>
fn try_compound_rewards(deps: DepsMut, priority: ContractStatusLevel) -> StdResult<Response> {
    check_status(deps.storage, priority)?;
    let config = CONFIG.load(deps.storage)?;
    let staked = get_staked_shd(deps.querier, &config.contract_address, &config)?;
    let rewards = query_rewards(deps.querier, &config.contract_address, &config)?;

    let mut messages: Vec<CosmosMsg> = vec![];
    if staked > 0 {
        messages.push(compound_msg(
            config.staking.code_hash,
            config.staking.address.to_string(),
        )?);
    }
    let response = Response::default();
    let rewarded_tokens_list = REWARDED_TOKENS_LIST
        .may_load(deps.storage)?
        .unwrap_or_default();
    for addr in rewarded_tokens_list.into_iter() {
        let token = ContractsVksStore::may_load(deps.storage, &addr);
        if let Some(t) = token {
            let balance = balance_query(
                deps.querier,
                config.contract_address.to_string(),
                t.viewing_key,
                RESPONSE_BLOCK_SIZE,
                t.code_hash.clone(),
                t.address.to_string(),
            )?;

            let item = rewards
                .rewards
                .iter()
                .find(|r| r.token.address == t.address);

            let amount = if let Some(reward) = item {
                balance.amount.add(reward.amount)
            } else {
                balance.amount
            };

            if amount > Uint128::zero() {
                messages.push(send_msg(
                    config.fees.collector.to_string(),
                    amount,
                    None,
                    Some(format!(
                        "Sending {} rewards to ShadeDAO",
                        t.address.to_string()
                    )),
                    None,
                    RESPONSE_BLOCK_SIZE,
                    t.code_hash.clone(),
                    t.address.to_string(),
                )?);
            }
        }
    }

    Ok(response.add_messages(messages))
}

/// It checks if the sender is an admin, and if so, it sends a message to the staking contract to unbond
/// the given amount
///
/// Arguments:
///
/// * `deps`: DepsMut - This is the dependencies object that contains the storage, querier, and other
/// useful objects.
/// * `info`: MessageInfo - this is a struct that contains the sender, sent_funds, and sent_funds_count.
/// * `amount`: The amount of tokens to unbond.
///
/// Returns:
///
/// StdResult<Response>.
fn try_panic_unbond(
    env: Env,
    deps: DepsMut,
    info: MessageInfo,
    amount: Uint128,
    priority: ContractStatusLevel,
) -> StdResult<Response> {
    check_status(deps.storage, priority)?;
    let config = CONFIG.load(deps.storage)?;
    check_if_admin(
        &deps.querier,
        AdminPermissions::DerivativeAdmin,
        info.sender.to_string(),
        &config.admin,
    )?;
    // Store panic unbond
    let staking_config = get_staking_contract_config(deps.querier, &config)?;
    let complete: Uint128 =
        Uint128::from(env.block.time.seconds()).checked_add(staking_config.unbond_period)?;
    let mut panic_unbonds: Vec<PanicUnbond> =
        PANIC_UNBONDS.may_load(deps.storage)?.unwrap_or_default();
    panic_unbonds.push(PanicUnbond {
        id: Uint128::zero(),
        amount,
        complete,
    });
    PANIC_UNBONDS.save(deps.storage, &panic_unbonds)?;

    let msg = unbond_msg(
        amount,
        config.staking.code_hash,
        config.staking.address.to_string(),
        Some(false),
    )?;
    Ok(Response::default().add_submessage(SubMsg::reply_always(msg, PANIC_UNBOND_REPLY_ID)))
}

/// It sends a message to the staking contract to claim rewards, then sends a message to the staking contract
/// to withdraw the rewards and then sends available SHD balance to the super admin address
///
/// Arguments:
///
/// * `deps`: DepsMut,
/// * `env`: The environment of the contract.
/// * `info`: MessageInfo - contains the sender, sent_funds, and sent_funds_attachment
/// * `ids`: Option<Vec<Uint128>>
///
/// Returns:
///
/// StdResult<Response>.
fn try_panic_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    priority: ContractStatusLevel,
) -> StdResult<Response> {
    check_status(deps.storage, priority)?;
    let config = CONFIG.load(deps.storage)?;
    check_if_admin(
        &deps.querier,
        AdminPermissions::DerivativeAdmin,
        info.sender.to_string(),
        &config.admin,
    )?;
    let addr = get_super_admin(&deps.querier, &config)?;
    let rewards = get_rewards(deps.querier, &env.contract.address, &config)?;
    let balance = get_available_shd(deps.querier, &env.contract.address, &config)?;
    let amount = Uint128::from(rewards + balance);
    let mut response = Response::default().add_messages(vec![
        claim_rewards_msg(
            config.staking.code_hash.clone(),
            config.staking.address.to_string(),
        )?,
        send_msg(
            addr.to_string(),
            amount,
            None,
            Some("Panic withdraw {} tokens".to_string()),
            config.token.entropy,
            RESPONSE_BLOCK_SIZE,
            config.token.code_hash,
            config.token.address.to_string(),
        )?,
    ]);
    let panic_unbonds = PANIC_UNBONDS.may_load(deps.storage)?;
    if let Some(unbonds) = panic_unbonds {
        let time = Uint128::from(env.block.time.seconds());
        let mut to_withdraw: Vec<Uint128> = vec![];
        let mut pending_unbonds: Vec<PanicUnbond> = vec![];

        for u in unbonds.into_iter() {
            if time >= u.complete {
                to_withdraw.push(u.id);
            } else {
                pending_unbonds.push(u);
            }
        }

        PANIC_UNBONDS.save(deps.storage, &pending_unbonds)?;

        response = response.add_submessage(SubMsg::reply_on_success(
            withdraw_msg(
                config.staking.code_hash,
                config.staking.address.to_string(),
                Some(to_withdraw.clone()),
            )?,
            PANIC_WITHDRAW_REPLY_ID,
        ));
    }

    Ok(response)
}

/// `update_fees` updates the fees for staking and unbonding
///
/// Arguments:
///
/// * `deps`: DepsMut - This is the dependency object that contains the storage, querier, and logger.
/// * `info`: MessageInfo - this is the information about the message that was sent to the contract.
/// * `staking`: The fee for staking.
/// * `unbonding`: Option<Fee>
///
/// Returns:
///
/// StdResult<Response>.
fn update_fees(
    deps: DepsMut,
    info: MessageInfo,
    staking: Option<Fee>,
    unbonding: Option<Fee>,
    collector: Option<Addr>,
    priority: ContractStatusLevel,
) -> StdResult<Response> {
    check_status(deps.storage, priority)?;
    let mut config = CONFIG.load(deps.storage)?;
    check_if_admin(
        &deps.querier,
        AdminPermissions::DerivativeAdmin,
        info.sender.to_string(),
        &config.admin,
    )?;
    let fees: FeeInfo = FeeInfo {
        staking: staking.unwrap_or(config.fees.staking),
        unbonding: unbonding.unwrap_or(config.fees.unbonding),
        collector: collector.unwrap_or(config.fees.collector),
    };
    config.fees = fees.clone();
    CONFIG.save(deps.storage, &config)?;

    Ok(
        Response::default().set_data(to_binary(&ExecuteAnswer::UpdateFees {
            status: Success,
            fee: fees,
        })?),
    )
}

/// If the message is a `Stake` message, call `try_stake`, if it's an `Unbond` message, call
/// `try_unbond`, otherwise return an error
///
/// Arguments:
///
/// * `deps`: This is a struct that contains all the dependencies that the contract needs to run.
/// * `env`: The environment of the transaction.
/// * `info`: MessageInfo - contains information about the message that was sent to the contract
/// * `from`: The address of the sender
/// * `amount`: The amount of tokens sent to the contract.
///
/// Returns:
///
/// Response::default()
fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: Addr,
    amount: Uint256,
    msg: Option<Binary>,
) -> StdResult<Response> {
    if let Some(x) = msg {
        match from_binary(&x)? {
            ReceiverMsg::Stake {} => try_stake(
                deps,
                env,
                info,
                from,
                amount,
                ContractStatusLevel::NormalRun,
            ),
            ReceiverMsg::Unbond {} => try_unbond(
                deps,
                env,
                info,
                from,
                amount,
                ContractStatusLevel::NormalRun,
            ),
            ReceiverMsg::TransferStaked { receiver } => try_transfer_staked(
                deps,
                env,
                info,
                from,
                amount,
                receiver,
                ContractStatusLevel::NormalRun,
            ),
            #[allow(unreachable_patterns)]
            _ => Err(StdError::generic_err(format!(
                "Invalid msg provided, expected {} , {} or {}",
                to_binary(&ReceiverMsg::Stake {})?,
                to_binary(&ReceiverMsg::Unbond {})?,
                to_binary(&ReceiverMsg::TransferStaked { receiver: None })?
            ))),
        }
    } else {
        Ok(Response::default())
    }
}

/// `try_stake` takes a deposit of SHD and returns the equivalent mint of the derivative token
///
/// Arguments:
///
/// * `deps`: DepsMut,
/// * `env`: The environment of the contract.
/// * `info`: MessageInfo - contains information about the message that was sent to the contract
/// * `from`: The address of the staker
/// * `amt`: The amount of SHD to stake.
///
/// Returns:
///
/// The amount of tokens that were minted for the staking transaction.
fn try_stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: Addr,
    amt: Uint256,
    priority: ContractStatusLevel,
) -> StdResult<Response> {
    check_status(deps.storage, priority)?;
    let config = CONFIG.load(deps.storage)?;
    let amount = Uint128::try_from(amt)?;
    if info.sender != config.token.address {
        return Err(StdError::generic_err("Sender is not SHD contract"));
    }

    if amount == Uint128::zero() {
        return Err(StdError::generic_err("No SHD was sent for staking"));
    }

    let rewards = query_rewards(deps.querier, &config.contract_address, &config)?;

    let mut non_shd_rewards: Vec<Reward> = vec![];
    let mut shd_rewards: Option<Reward> = None;

    for r in rewards.rewards.into_iter() {
        if r.token.address == config.token.address {
            shd_rewards = Some(r);
        } else {
            non_shd_rewards.push(r)
        }
    }
    let rewards_amount = if let Some(r) = shd_rewards {
        r.amount.u128()
    } else {
        0_128
    };

    let available = get_available_shd(deps.querier, &config.contract_address, &config)?;
    let (fee, deposit) = get_fee(amount, &config.fees.staking)?;

    // get available SHD + available rewards
    let claiming = available + rewards_amount;

    // get staked SHD
    let bonded = get_staked_shd(deps.querier, &env.contract.address, &config)?;
    let starting_pool = (claiming + bonded).saturating_sub(deposit.u128() + fee.u128());

    let token_info = get_token_info(
        deps.querier,
        RESPONSE_BLOCK_SIZE,
        config.derivative.code_hash.clone(),
        config.derivative.address.to_string(),
        true,
    )?;
    let total_supply = token_info.total_supply.unwrap_or(Uint128::zero());
    // mint appropriate amount
    let mint = if starting_pool == 0 || total_supply.is_zero() {
        deposit
    } else {
        // unwrap is ok because multiplying 2 u128 ints can not overflow a u256
        let numer = Uint256::from(deposit)
            .checked_mul(Uint256::from(total_supply))
            .unwrap();
        // unwrap is ok because starting pool can not be zero
        Uint128::try_from(numer.checked_div(Uint256::from(starting_pool)).unwrap())?
    };
    if mint == Uint128::zero() {
        return Err(StdError::generic_err("The amount of SHD deposited is not enough to receive any of the derivative token at the current price"));
    }
    // Sync rewarded tokens
    let mut messages = sync_rewarded_tokens(&env, deps, info, &non_shd_rewards, &config)?;

    // Mint derivatives in exchange
    messages.push(mint_msg(
        from.to_string(),
        mint,
        Some(format!(
            "Minted {} u_{} to stake {} SHD",
            mint, token_info.symbol, deposit
        )),
        config.derivative.entropy.clone(),
        RESPONSE_BLOCK_SIZE,
        config.derivative.code_hash.clone(),
        config.derivative.address.to_string(),
    )?);

    // send fee to collector
    messages.push(send_msg(
        config.fees.collector.to_string(),
        fee,
        None,
        Some(base64::encode(format!(
            "Payment of fee for staking SHD using contract {}",
            env.contract.address.clone()
        ))),
        config.token.entropy.clone(),
        RESPONSE_BLOCK_SIZE,
        config.token.code_hash.clone(),
        config.token.address.to_string(),
    )?);

    // Stake available SHD
    if deposit > Uint128::zero() {
        messages.push(generate_stake_msg(deposit, Some(true), &config)?);
    }

    Ok(Response::new()
        .add_attribute("derivative_returned", mint)
        .set_data(to_binary(&ExecuteAnswer::Stake {
            shd_staked: deposit,
            tokens_returned: mint,
        })?)
        .add_messages(messages))
}

fn try_transfer_staked(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: Addr,
    amt: Uint256,
    receiver: Option<Addr>,
    priority: ContractStatusLevel,
) -> StdResult<Response> {
    check_status(deps.storage, priority)?;
    let config = CONFIG.load(deps.storage)?;
    let amount = Uint128::try_from(amt)?;

    let derivative_token_info = get_token_info(
        deps.querier,
        RESPONSE_BLOCK_SIZE,
        config.derivative.code_hash.clone(),
        config.derivative.address.to_string(),
        true,
    )?;

    if info.sender != config.derivative.address {
        return Err(StdError::generic_err(
            "Sender is not derivative (SNIP20) contract",
        ));
    }

    if amount == Uint128::zero() {
        return Err(StdError::generic_err("0 amount sent to unbond"));
    }

    let (_, rewards, delegatable) = get_delegatable(deps.querier, &env.contract.address, &config)?;
    let staked = get_staked_shd(deps.querier, &env.contract.address, &config)?;
    let pool = delegatable + staked;
    // unwrap is ok because multiplying 2 u128 ints can not overflow a u256
    let number = Uint256::from(amount)
        .checked_mul(Uint256::from(pool))
        .unwrap();
    // unwrap is ok because derivative token supply could not have been 0 if we were able
    // to burn
    let unbond_amount = Uint128::try_from(
        number
            .checked_div(Uint256::from(derivative_token_info.total_supply.unwrap()))
            .unwrap(),
    )?;
    // calculate the amount going to the user and fee's collector
    let (fee, deposit) = get_fee(unbond_amount, &config.fees.unbonding)?;

    if deposit.is_zero() {
        return Err(StdError::generic_err(format!(
            "Redeeming {} derivative tokens would be worth less than 1 SHD",
            amount
        )));
    }
    let recipient: String = receiver.unwrap_or(from).to_string();
    Ok(Response::default()
        .add_messages([
            // Claim rewards
            claim_rewards_msg(
                config.staking.code_hash.clone(),
                config.staking.address.to_string(),
            )?,
            // Re-stake rewards
            generate_stake_msg(
                Uint128::from(rewards).saturating_sub(fee),
                Some(true),
                &config,
            )?,
            // Burn derivatives sent
            burn_msg(
                amount,
                Some(format!(
                    "Burn {} derivatives to receive {} SHD",
                    amount, deposit
                )),
                config.derivative.entropy,
                RESPONSE_BLOCK_SIZE,
                config.derivative.code_hash.clone(),
                config.derivative.address.to_string(),
            )?,
            //Sends fee collector
            send_msg(
                config.fees.collector.to_string(),
                fee,
                None,
                Some(base64::encode(format!(
                    "Payment of fee for transfer staked SHD using contract {}",
                    env.contract.address
                ))),
                config.token.entropy.clone(),
                RESPONSE_BLOCK_SIZE,
                config.token.code_hash.clone(),
                config.token.address.to_string(),
            )?,
            // Transfer staked
            transfer_staked_msg(
                config.staking.code_hash,
                config.staking.address.to_string(),
                deposit,
                recipient,
                Some(true),
            )?,
        ])
        .set_data(to_binary(&ExecuteAnswer::TransferStaked {
            tokens_returned: deposit,
            amount_sent: amount,
        })?))
}

/// `try_unbond` is called when a user sends a derivative token to the contract. The contract then
/// calculates the amount of SHD that the user will receive and unbonds it from the staking contract
/// this amount will be maturing in the unbondings for X amount of time
///
/// Arguments:
///
/// * `deps`: DepsMut,
/// * `env`: The environment in which the contract is running.
/// * `info`: MessageInfo - this is the information about the message that was sent to the contract.
/// * `from`: The address of the user who is unbonding
/// * `amt`: The amount of derivative tokens to be redeemed.
///
/// Returns:
///
/// The amount of SHD that will be received when the unbonding period is over.
fn try_unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    from: Addr,
    amt: Uint256,
    priority: ContractStatusLevel,
) -> StdResult<Response> {
    check_status(deps.storage, priority)?;
    let mut response = Response::new();
    let config = CONFIG.load(deps.storage)?;
    let derivative_token_info = get_token_info(
        deps.querier,
        RESPONSE_BLOCK_SIZE,
        config.derivative.code_hash.clone(),
        config.derivative.address.to_string(),
        true,
    )?;
    let staking_info = get_staking_contract_config(deps.querier, &config)?;
    let amount = Uint128::try_from(amt)?;
    if info.sender != config.derivative.address {
        return Err(StdError::generic_err(
            "Sender is not derivative (SNIP20) contract",
        ));
    }

    if amount == Uint128::zero() {
        return Err(StdError::generic_err("0 amount sent to unbond"));
    }

    let (_, _, delegatable) = get_delegatable(deps.querier, &env.contract.address, &config)?;

    let staked = get_staked_shd(deps.querier, &env.contract.address, &config)?;
    let pool = delegatable + staked;
    // unwrap is ok because multiplying 2 u128 ints can not overflow a u256
    let number = Uint256::from(amount)
        .checked_mul(Uint256::from(pool))
        .unwrap();
    // unwrap is ok because derivative token supply could not have been 0 if we were able
    // to burn
    let unbond_amount = Uint128::try_from(
        number
            .checked_div(Uint256::from(derivative_token_info.total_supply.unwrap()))
            .unwrap(),
    )?;
    // calculate the amount going to the user
    let (_, shd_to_be_received) = get_fee(unbond_amount, &config.fees.unbonding)?;

    if shd_to_be_received.is_zero() {
        return Err(StdError::generic_err(format!(
            "Redeeming {} derivative tokens would be worth less than 1 SHD",
            amount
        )));
    }

    // Store unbonding temporarily
    // This unbonding is used in unbond sub-message reply handler
    // Due to that in reply handler you can't access sender information
    // and this is required to store user's unbondings
    let unbonding = InProcessUnbonding {
        id: Uint128::zero(),
        amount: shd_to_be_received,
        owner: from,
        complete: Uint128::from(env.block.time.seconds())
            .checked_add(staking_info.unbond_period)?,
    };
    PENDING_UNBONDING.save(deps.storage, &unbonding)?;
    CONFIG.save(deps.storage, &config)?;

    response = response.add_submessage(SubMsg::reply_always(
        unbond_msg(
            shd_to_be_received,
            config.staking.code_hash.clone(),
            config.staking.address.to_string(),
            Some(true),
        )?,
        UNBOND_REPLY_ID,
    ));

    Ok(response
        .add_attribute("unbonded_amount", shd_to_be_received)
        .add_message(burn_msg(
            amount,
            Some(format!(
                "Burn {} derivatives to receive {} SHD",
                amount, shd_to_be_received
            )),
            config.derivative.entropy,
            RESPONSE_BLOCK_SIZE,
            config.derivative.code_hash.clone(),
            config.derivative.address.to_string(),
        )?)
        .set_data(to_binary(&ExecuteAnswer::Unbond {
            shd_to_be_received,
            tokens_redeemed: amount,
            estimated_time_of_maturity: Uint128::from(env.block.time.seconds())
                .checked_add(staking_info.unbond_period)?,
        })?))
}

/// It queries the token's balance of the contract address
///
/// Arguments:
///
/// * `querier`: The querier object that will be used to query the blockchain.
/// * `contract_addr`: The address of the contract that we want to query.
/// * `config`: The configuration object that contains the token contract address, token contract
/// verifier key, and token code hash.
///
/// Returns:
///
/// The available SHD balance of the contract.
#[cfg(not(test))]
#[allow(dead_code)]
fn get_available_shd<C: CustomQuery>(
    querier: QuerierWrapper<C>,
    contract_addr: &Addr,
    config: &Config,
) -> StdResult<u128> {
    let balance = balance_query(
        querier,
        contract_addr.to_string(),
        config.token_contract_vk.clone(),
        RESPONSE_BLOCK_SIZE,
        config.token.code_hash.to_string(),
        config.token.address.to_string(),
    )?;

    let available = balance.amount;
    Ok(available.u128())
}
#[cfg(test)]
fn get_available_shd<C: CustomQuery>(
    _: QuerierWrapper<C>,
    _: &Addr,
    _: &Config,
) -> StdResult<u128> {
    Ok(100000000_u128)
}

/// It queries the staking contract to get the amount of staked SHD for the given contract address
///
/// Arguments:
///
/// * `querier`: The querier object that will be used to query the blockchain.
/// * `contract_addr`: The address of the contract that is being queried.
/// * `config`: The configuration of the contract.
///
/// Returns:
///
/// The balance of the staking contract.
#[cfg(not(test))]
fn get_staked_shd<C: CustomQuery>(
    querier: QuerierWrapper<C>,
    contract_addr: &Addr,
    config: &Config,
) -> StdResult<u128> {
    let balance = staking_balance_query(
        contract_addr.to_string(),
        config.staking_contract_vk.clone(),
        querier,
        config.staking.code_hash.to_string(),
        config.staking.address.to_string(),
    )?;

    Ok(balance.amount.u128())
}
#[cfg(test)]
fn get_staked_shd<C: CustomQuery>(_: QuerierWrapper<C>, _: &Addr, _: &Config) -> StdResult<u128> {
    Ok(300000000)
}

/// It queries the rewards generated for this contract address.
/// Filters out to the token contract rewards and returns them as u128
///
/// Arguments:
///
/// * `querier`: The querier object that will be used to query the blockchain.
/// * `contract_addr`: The address of the contract that we want to query.
/// * `config`: The configuration file that contains the contract addresses and other parameters.
///
/// Returns:
///
/// The rewards for the staking contract.
#[cfg(not(test))]
fn get_rewards<C: CustomQuery>(
    querier: QuerierWrapper<C>,
    contract_addr: &Addr,
    config: &Config,
) -> StdResult<u128> {
    let rewards = query_rewards(querier, contract_addr, config)?;
    let item = rewards
        .rewards
        .iter()
        .find(|r| r.token.address == config.token.address);

    if let Some(reward) = item {
        Ok(reward.amount.u128())
    } else {
        Ok(0)
    }
}
#[cfg(test)]
#[allow(dead_code)]
// Allow warn code because mock queries make warnings to show up
fn get_rewards<C: CustomQuery>(_: QuerierWrapper<C>, _: &Addr, _: &Config) -> StdResult<u128> {
    Ok(100000000)
}

#[cfg(not(test))]
#[allow(dead_code)]
// Allow warn code because mock queries make warnings to show up
fn query_rewards<C: CustomQuery>(
    querier: QuerierWrapper<C>,
    contract_addr: &Addr,
    config: &Config,
) -> StdResult<Rewards> {
    rewards_query(
        contract_addr.to_string(),
        config.staking_contract_vk.clone(),
        querier,
        config.staking.code_hash.to_string(),
        config.staking.address.to_string(),
    )
}

#[cfg(test)]
#[allow(dead_code)]
// Allow warn code because mock queries make warnings to show up
fn query_rewards<C: CustomQuery>(_: QuerierWrapper<C>, _: &Addr, _: &Config) -> StdResult<Rewards> {
    use crate::staking_interface::RewardToken;

    Ok(Rewards {
        rewards: vec![Reward {
            token: RewardToken {
                address: Addr::unchecked("shade_contract_info_address"),
                code_hash: String::from("shade_contract_info_code_hash"),
            },
            amount: Uint128::from(100000000_u128),
        }],
    })
}

#[cfg(test)]
#[allow(dead_code)]
// Allow warn code because mock queries make warnings to show up
fn get_staking_contract_config<C: CustomQuery>(
    _: QuerierWrapper<C>,
    _: &Config,
) -> StdResult<StakingConfig> {
    Ok(StakingConfig {
        admin_auth: RawContract {
            address: String::from("mock_address"),
            code_hash: String::from("mock_code_hash"),
        },
        query_auth: RawContract {
            address: String::from("mock_address"),
            code_hash: String::from("mock_code_hash"),
        },
        unbond_period: Uint128::from(300_u32),
        max_user_pools: Uint128::from(5_u32),
        reward_cancel_threshold: Uint128::from(0_u32),
    })
}

/// It queries the staking contract for its configuration
///
/// Arguments:
///
/// * `querier`: The querier object that will be used to query the contract.
/// * `staking_info`: The staking contract information.
///
/// Returns:
///
/// StakingConfig
#[cfg(not(test))]
fn get_staking_contract_config<C: CustomQuery>(
    querier: QuerierWrapper<C>,
    staking_info: &Config,
) -> StdResult<StakingConfig> {
    config_query(
        querier,
        staking_info.staking.code_hash.clone(),
        staking_info.staking.address.to_string(),
    )
}

/// It gets the available and rewards balances, and returns the sum of the two
///
/// Arguments:
///
/// * `querier`: The querier object that will be used to query the contract.
/// * `contract_addr`: The address of the contract that you want to query.
/// * `config`: The configuration of the contract.
///
/// Returns:
///
/// a tuple of three values:
/// - The first value is the amount of available SHD
/// - The second value is the amount of rewards
/// - The third value is the sum of the first two values
#[cfg(not(test))]
fn get_delegatable<C: CustomQuery>(
    querier: QuerierWrapper<C>,
    contract_addr: &Addr,
    config: &Config,
) -> StdResult<(u128, u128, u128)> {
    let rewards = get_rewards(querier, contract_addr, config)?;

    let available = get_available_shd(querier, contract_addr, config)?;
    Ok((available, rewards, rewards + available))
}

#[cfg(test)]
fn get_delegatable<C: CustomQuery>(
    _: QuerierWrapper<C>,
    _: &Addr,
    _: &Config,
) -> StdResult<(u128, u128, u128)> {
    Ok((100000000, 50000000, 100000000 + 50000000))
}

#[cfg(test)]
fn get_super_admin(_: &QuerierWrapper, _: &Config) -> StdResult<Addr> {
    Ok(Addr::unchecked("super_admin"))
}
/// It queries the `admin` contract for the `super_admin` address
///
/// Arguments:
///
/// * `querier`: The querier object that will be used to query the blockchain.
/// * `config`: The configuration of the current contract.
///
/// Returns:
///
/// The address of the super admin.
#[cfg(not(test))]
fn get_super_admin(querier: &QuerierWrapper, config: &Config) -> StdResult<Addr> {
    let response: StdResult<ConfigResponse> =
        AdminQueryMsg::GetConfig {}.query(querier, &config.admin);

    match response {
        Ok(resp) => Ok(resp.super_admin),
        Err(err) => Err(err),
    }
}

/// It takes an amount and a fee config, and returns the fee and the remainder
///
/// Arguments:
///
/// * `amount`: The amount of tokens to be transferred
/// * `fee_config`: The fee configuration for the transaction.
///
/// Returns:
///
/// A tuple of two Uint128 values.
pub fn get_fee(amount: Uint128, fee_config: &Fee) -> StdResult<(Uint128, Uint128)> {
    // first unwrap is ok because multiplying a u128 by a u32 can not overflow a u256
    // second unwrap is ok because we know we aren't dividing by zero
    let _fee = Uint256::from(amount)
        .checked_mul(Uint256::from(fee_config.rate))
        .unwrap()
        .checked_div(Uint256::from(10_u32.pow(fee_config.decimal_places as u32)))
        .unwrap();
    let fee = Uint128::try_from(_fee)?;
    let remainder = amount.saturating_sub(fee);
    Ok((fee, remainder))
}
/// It queries the token contract for the token info, and
/// if the total supply is not public, it returns an error
///
/// Arguments:
///
/// * `querier`: The querier object that will be used to query the blockchain.
/// * `block_size`: The number of blocks to look back for the token's price.
/// * `callback_code_hash`: The code hash of the contract that will be called when the derivative token
/// is redeemed.
/// * `contract_addr`: The address of the contract that holds the token.
///
/// Returns:
///
/// A TokenInfo struct
#[cfg(not(test))]
fn get_token_info<C: CustomQuery>(
    querier: QuerierWrapper<C>,
    block_size: usize,
    callback_code_hash: String,
    contract_addr: String,
    check_public_supply: bool,
) -> StdResult<TokenInfo> {
    let token_info = token_info_query(querier, block_size, callback_code_hash, contract_addr)?;
    if check_public_supply && token_info.total_supply.is_none() {
        return Err(StdError::generic_err(
            "Token supply must be public on derivative token",
        ));
    }

    Ok(token_info)
}

#[cfg(test)]
fn get_token_info<C: CustomQuery>(
    _querier: QuerierWrapper<C>,
    _block_size: usize,
    _callback_code_hash: String,
    _contract_addr: String,
    _check_public_supply: bool,
) -> StdResult<TokenInfo> {
    Ok(TokenInfo {
        name: String::from("STKD-SHD"),
        symbol: String::from("STKDSHD"),
        decimals: 6,
        total_supply: Some(Uint128::from(2000_u128)),
    })
}
/// It checks if the user is an admin, and if so, it returns `Ok(())`
///
/// Arguments:
///
/// * `querier`: The querier object that can be used to query the state of the blockchain.
/// * `permission`: The permission you want to check for.
/// * `user`: The user to check if they are an admin.
/// * `admin_auth`: The contract that holds the admin permissions.
///
/// Returns:
///
/// A StdResult<()>
#[cfg(not(test))]
fn check_if_admin(
    querier: &QuerierWrapper,
    permission: AdminPermissions,
    user: String,
    admin_auth: &Contract,
) -> StdResult<()> {
    validate_admin(querier, permission, user, admin_auth)
}

#[cfg(test)]
fn check_if_admin(
    _: &QuerierWrapper,
    _: AdminPermissions,
    user: String,
    _: &Contract,
) -> StdResult<()> {
    if user != String::from("admin") {
        return Err(StdError::generic_err(
            "This is an admin command. Admin commands can only be run from admin address",
        ));
    }

    Ok(())
}

/// It takes an amount of SHD to stake, a boolean indicating whether or not to compound the stake, and a
/// configuration object, and returns a CosmosMsg object that can be used to send a transaction to the
/// staking contract
///
/// Arguments:
///
/// * `amount`: The amount of SHD to stake
/// * `compound`: Whether to compound the interest or not.
/// * `config`: The configuration file that contains the staking contract address, token address, token
/// code hash, and entropy.
///
/// Returns:
///
/// A CosmosMsg
fn generate_stake_msg(
    amount: Uint128,
    compound: Option<bool>,
    config: &Config,
) -> StdResult<CosmosMsg> {
    let memo =
        Some(to_binary(&format!("Staking {} SHD into staking contract", amount))?.to_base64());
    let msg = Some(to_binary(&Action::Stake { compound })?);
    send_msg(
        config.staking.address.to_string(),
        amount,
        msg,
        memo,
        config.token.entropy.clone(),
        RESPONSE_BLOCK_SIZE,
        config.token.code_hash.clone(),
        config.token.address.to_string(),
    )
}

/// It checks if the sender is an admin, and if so, it sets the contract status to the value passed in
///
/// Arguments:
///
/// * `deps`: DepsMut - This is the set of dependencies that the contract needs to run.
/// * `info`: MessageInfo - contains the sender, sent_funds, and sent_funds_count
/// * `status_level`: ContractStatusLevel
///
/// Returns:
///
/// The response is being returned.
fn set_contract_status(
    deps: DepsMut,
    info: MessageInfo,
    status_level: ContractStatusLevel,
    priority: ContractStatusLevel,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    check_status(deps.storage, priority)?;
    check_if_admin(
        &deps.querier,
        AdminPermissions::DerivativeAdmin,
        info.sender.to_string(),
        &config.admin,
    )?;

    CONTRACT_STATUS.save(deps.storage, &status_level)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::SetContractStatus {
            status: Success,
        })?),
    )
}

// Copied from secret-toolkit-viewing-key-0.7.0
pub fn new_viewing_key(
    sender: &Addr,
    env: &Env,
    seed: &[u8],
    entropy: &[u8],
) -> (String, [u8; 32]) {
    pub const VIEWING_KEY_PREFIX: &str = "api_key_";
    // 16 here represents the lengths in bytes of the block height and time.
    let entropy_len = 16 + sender.to_string().len() + entropy.len();
    let mut rng_entropy = Vec::with_capacity(entropy_len);
    rng_entropy.extend_from_slice(&env.block.height.to_be_bytes());
    rng_entropy.extend_from_slice(&env.block.time.seconds().to_be_bytes());
    rng_entropy.extend_from_slice(sender.as_bytes());
    rng_entropy.extend_from_slice(entropy);

    let mut rng = ContractPrng::new(seed, &rng_entropy);

    let rand_slice = rng.rand_bytes();

    let key = sha_256(&rand_slice);

    let viewing_key = VIEWING_KEY_PREFIX.to_string() + &base64::encode(key);
    (viewing_key, rand_slice)
}

pub fn sync_rewarded_tokens(
    env: &Env,
    deps: DepsMut,
    info: MessageInfo,
    rewarded_tokens: &Vec<Reward>,
    config: &Config,
) -> StdResult<Vec<CosmosMsg>> {
    let mut messages = vec![];
    let mut no_registered_tokens: Vec<Addr> = vec![];

    for r in rewarded_tokens.into_iter() {
        let is_registered = ContractsVksStore::may_load(deps.storage, &r.token.address);

        if is_registered.is_none() {
            // This contract isn't registered.
            no_registered_tokens.push(r.token.address.clone());
            // Generated a vk for this token and store it
            let (new_vk, _) = new_viewing_key(
                &info.sender,
                &env,
                &config.prng_seed,
                config.staking.entropy.clone().unwrap_or_default().as_ref(),
            );

            let token: Token = Token {
                address: r.token.address.clone(),
                code_hash: r.token.code_hash.clone(),
                viewing_key: new_vk.clone(),
            };
            ContractsVksStore::save(deps.storage, &r.token.address, &token)?;

            messages.push(set_viewing_key_msg(
                new_vk,
                None,
                RESPONSE_BLOCK_SIZE,
                r.token.code_hash.clone(),
                r.token.address.to_string(),
            )?)
        }
    }

    if no_registered_tokens.len() > 0 {
        let registered_tokens = REWARDED_TOKENS_LIST
            .may_load(deps.storage)?
            .unwrap_or_default();
        let new_rewarded_tokens = [registered_tokens, no_registered_tokens].concat();

        REWARDED_TOKENS_LIST.save(deps.storage, &new_rewarded_tokens)?;
    }

    Ok(messages)
}

/// If the contract admin has disabled the contract, then this function will return an error
///
/// Arguments:
///
/// * `storage`: The storage object that is passed to the contract.
/// * `priority`: The priority of the action being performed.
///
/// Returns:
///
/// Ok is messages is allowed.
fn check_status(storage: &dyn Storage, priority: ContractStatusLevel) -> StdResult<()> {
    let contract_status = CONTRACT_STATUS.load(storage)?;

    if status_level_to_u8(priority) < status_level_to_u8(contract_status) {
        return Err(StdError::generic_err(
            "The contract admin has temporarily disabled this action",
        ));
    }
    Ok(())
}
/************ QUERIES ************/

fn permit_queries(deps: Deps, env: &Env, permit: QueryPermit) -> Result<Binary, StdError> {
    // Validate permit content
    let config = CONFIG.load(deps.storage)?;
    let (addr, query) = validate_permit::<QueryWithPermit>(&deps.querier, &config, permit)?;

    // Permit validated! We can now execute the query.
    match query {
        QueryWithPermit::Unbondings {} => query_unbondings(&deps, addr),
        QueryWithPermit::Holdings {} => query_holdings(&deps, env, addr),
        #[allow(unreachable_patterns)]
        _ => Err(StdError::generic_err("Invalid query message")),
    }
}

pub fn validate_permit<T: DeserializeOwned>(
    querier: &QuerierWrapper,
    config: &Config,
    permit: QueryPermit,
) -> StdResult<(Addr, T)> {
    let authenticator = Contract {
        address: config.query_auth.address.clone(),
        code_hash: config.query_auth.code_hash.clone(),
    };
    let response = authenticate_permit::<T>(permit, querier, authenticator)?;

    if response.revoked {
        return Err(StdError::generic_err("Permit was revoked"));
    }

    Ok((response.sender, response.data))
}
#[cfg(not(test))]
pub fn validate_viewing_key(
    querier: &QuerierWrapper,
    config: &Config,
    address: Addr,
    key: String,
) -> StdResult<()> {
    let authenticator = Contract {
        address: config.query_auth.address.clone(),
        code_hash: config.query_auth.code_hash.clone(),
    };
    let is_valid = authenticate_vk(address, key, querier, &authenticator)?;

    if !is_valid {
        return Err(StdError::generic_err("Invalid viewing key"));
    }

    Ok(())
}
#[cfg(test)]
pub fn validate_viewing_key(
    _querier: &QuerierWrapper,
    _config: &Config,
    _address: Addr,
    key: String,
) -> StdResult<()> {
    if key != "password".to_string() {
        return Err(StdError::generic_err("Invalid viewing key"));
    }

    Ok(())
}

pub fn viewing_keys_queries(deps: Deps, env: &Env, msg: QueryMsg) -> StdResult<Binary> {
    let (addresses, key) = msg.get_validation_params(deps.api)?;
    let config = CONFIG.load(deps.storage)?;
    for address in addresses {
        validate_viewing_key(&deps.querier, &config, address, key)?;

        return match msg {
            QueryMsg::Unbondings { address, .. } => query_unbondings(&deps, address),
            QueryMsg::Holdings { address, .. } => query_holdings(&deps, env, address),
            _ => Err(StdError::generic_err(
                "This query type does not require authentication",
            )),
        };
    }

    to_binary(&QueryAnswer::ViewingKeyError {
        msg: "Wrong viewing key for this address or viewing key not set".to_string(),
    })
}

/// It loads all the unbonding ids for the given address, then for each unbonding id, it loads the
/// unbonding, and if the unbonding is complete, it adds the amount to the claimable amount, otherwise
/// it adds the amount to the unbonding amount
///
/// Arguments:
///
/// * `deps`: &Deps - this is the dependencies object that contains the storage, querier, and logger.
/// * `env`: The environment of the current transaction.
/// * `addr`: The address of the account to query
///
/// Returns:
///
/// The holdings of the address.
fn query_holdings(deps: &Deps, env: &Env, addr: Addr) -> StdResult<Binary> {
    let mut derivative_claimable = Uint128::zero();
    let mut derivative_unbonding = Uint128::zero();

    let time = Uint128::from(env.block.time.seconds());

    let unbondings_ids = UnbondingIdsStore::load(deps.storage, &addr);

    for id in unbondings_ids.into_iter() {
        let opt_unbonding = UnbondingStore::may_load(deps.storage, id);
        if let Some(unbonding) = opt_unbonding {
            if time >= unbonding.complete {
                derivative_claimable += unbonding.amount;
            } else {
                derivative_unbonding += unbonding.amount;
            }
        }
    }
    to_binary(&QueryAnswer::Holdings {
        derivative_claimable,
        derivative_unbonding,
    })
}

/// It loads all unbonding ids for a given address, then loads all unbonding structs for those ids, and
/// returns the result
///
/// Arguments:
///
/// * `deps`: &Deps - this is the dependencies object that contains the storage, querier, and logger.
/// * `addr`: The address of the user whose unbondings we want to query.
///
/// Returns:
///
/// A vector of unbonding structs.
fn query_unbondings(deps: &Deps, addr: Addr) -> StdResult<Binary> {
    let user_unbonds_ids = UnbondingIdsStore::load(deps.storage, &addr);
    let mut unbonds: Vec<Unbonding> = vec![];

    for id in user_unbonds_ids.into_iter() {
        let opt_unbonding = UnbondingStore::may_load(deps.storage, id);

        if let Some(unbond) = opt_unbonding {
            unbonds.push(unbond)
        }
    }

    to_binary(&QueryAnswer::Unbondings { unbonds })
}

/// It loads the contract status from the storage and returns it as a query answer
///
/// Arguments:
///
/// * `storage`: &dyn Storage - this is the storage that the contract will use to store and retrieve
/// data.
///
/// Returns:
///
/// A QueryAnswer::ContractStatus
fn query_contract_status(storage: &dyn Storage) -> StdResult<Binary> {
    let contract_status = CONTRACT_STATUS.load(storage)?;

    to_binary(&QueryAnswer::ContractStatus {
        status: contract_status,
    })
}

/// It queries the staking contract for the amount of SHD bonded, the amount of SHD available, the
/// amount of SHD in rewards, the total supply of the derivative token, and the price of the derivative
/// token
///
/// Arguments:
///
/// * `deps`: &Deps - this is a struct that contains all the dependencies that the contract needs to
/// run.
/// * `env`: The environment of the contract.
///
/// Returns:
///
/// The query_staking_info function returns the staking information of the contract.
fn query_staking_info(deps: &Deps, env: &Env) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;

    let derivative_info = get_token_info(
        deps.querier,
        RESPONSE_BLOCK_SIZE,
        config.derivative.code_hash.clone(),
        config.derivative.address.to_string(),
        true,
    )?;
    let bonded = get_staked_shd(deps.querier, &env.contract.address, &config)?;
    let rewards = get_rewards(deps.querier, &env.contract.address, &config)?;
    let available = get_available_shd(deps.querier, &env.contract.address, &config)?;

    let total_supply = derivative_info.total_supply.unwrap_or(Uint128::zero());

    let pool = bonded + rewards + available;
    let price = if total_supply == Uint128::zero() || pool == 0 {
        Uint128::from(10_u128.pow(derivative_info.decimals as u32))
    } else {
        // unwrap is ok because multiplying a u128 by 1 mill can not overflow u256
        let number = Uint256::from(pool)
            .checked_mul(Uint256::from(10_u128.pow(derivative_info.decimals as u32)))
            .unwrap();
        // unwrap is ok because we already checked if the total supply is 0
        Uint128::try_from(number.checked_div(Uint256::from(total_supply)).unwrap())?
    };

    let staking_contract_config = get_staking_contract_config(deps.querier, &config)?;

    to_binary(&QueryAnswer::StakingInfo {
        unbonding_time: staking_contract_config.unbond_period,
        bonded_shd: Uint128::from(bonded),
        available_shd: Uint128::from(available),
        rewards: Uint128::from(rewards),
        total_derivative_token_supply: total_supply,
        price,
    })
}

/// It loads the fee configuration from the storage, and returns it as a binary
///
/// Arguments:
///
/// * `deps`: &Deps
///
/// Returns:
///
/// The fee information for staking and unbonding.
fn query_fee_info(deps: &Deps) -> StdResult<Binary> {
    let config = CONFIG.load(deps.storage)?;

    to_binary(&QueryAnswer::FeeInfo {
        staking: config.fees.staking,
        unbonding: config.fees.unbonding,
        collector: config.fees.collector,
    })
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use cosmwasm_std::testing::*;
    use cosmwasm_std::{from_binary, OwnedDeps, QueryResponse};
    use shade_protocol::Contract;

    use crate::msg::{ContractInfo as CustomContractInfo, Fee, FeeInfo};

    use super::*;

    fn init_helper() -> (
        StdResult<Response>,
        OwnedDeps<MockStorage, MockApi, MockQuerier>,
    ) {
        let mut deps = mock_dependencies_with_balance(&[]);
        let env = mock_env();
        let info = mock_info("instantiator", &[]);

        let init_msg = InstantiateMsg {
            prng_seed: Binary::from("lolz fun yay".as_bytes()),
            derivative: CustomContractInfo {
                address: Addr::unchecked("derivative_snip20_info_address"),
                code_hash: String::from("derivative_snip20_info_codehash"),
                entropy: Some(String::from("4359o74nd8dnkjerjrh")),
            },
            staking: CustomContractInfo {
                address: Addr::unchecked("staking_contract_info_address"),
                code_hash: String::from("staking_contract_info_code_hash"),
                entropy: Some(String::from("4359o74nd8dnkjerjrh")),
            },
            query_auth: CustomContractInfo {
                address: Addr::unchecked("authentication_contract_info_address"),
                code_hash: String::from("authentication_contract_info_code_hash"),
                entropy: Some(String::from("ljkdsfgh9548605874easfnd")),
            },
            token: CustomContractInfo {
                address: Addr::unchecked("shade_contract_info_address"),
                code_hash: String::from("shade_contract_info_code_hash"),
                entropy: Some(String::from("5sa4d6aweg473g87766h7712")),
            },
            admin: Contract {
                address: Addr::unchecked("shade_contract_info_address"),
                code_hash: String::from("shade_contract_info_code_hash"),
            },
            fees: FeeInfo {
                staking: Fee {
                    rate: 5,
                    decimal_places: 2_u8,
                },
                unbonding: Fee {
                    rate: 5,
                    decimal_places: 2_u8,
                },
                collector: Addr::unchecked("collector_address"),
            },
        };

        (instantiate(deps.as_mut(), env, info, init_msg), deps)
    }
    fn extract_error_msg<T: Any>(error: StdResult<T>) -> String {
        match error {
            Ok(response) => {
                let bin_err = (&response as &dyn Any)
                    .downcast_ref::<QueryResponse>()
                    .expect("An error was expected, but no error could be extracted");
                match from_binary(bin_err).unwrap() {
                    QueryAnswer::ViewingKeyError { msg } => msg,
                    _ => panic!("Unexpected query answer"),
                }
            }
            Err(err) => match err {
                StdError::GenericErr { msg, .. } => msg,
                _ => panic!("Unexpected result from init"),
            },
        }
    }

    #[test]
    fn test_init_sanity() {
        let (init_result, deps) = init_helper();
        let env = mock_env();
        let info = mock_info("instantiator", &[]);
        let prnd = Binary::from("lolz fun yay".as_bytes());
        let staking = CustomContractInfo {
            address: Addr::unchecked("staking_contract_info_address"),
            code_hash: String::from("staking_contract_info_code_hash"),
            entropy: Some(String::from("4359o74nd8dnkjerjrh")),
        };

        let authentication_contract = CustomContractInfo {
            address: Addr::unchecked("authentication_contract_info_address"),
            code_hash: String::from("authentication_contract_info_code_hash"),
            entropy: Some(String::from("ljkdsfgh9548605874easfnd")),
        };
        let token = CustomContractInfo {
            address: Addr::unchecked("shade_contract_info_address"),
            code_hash: String::from("shade_contract_info_code_hash"),
            entropy: Some(String::from("5sa4d6aweg473g87766h7712")),
        };
        let derivative = CustomContractInfo {
            address: Addr::unchecked("derivative_snip20_info_address"),
            code_hash: String::from("derivative_snip20_info_codehash"),
            entropy: Some(String::from("4359o74nd8dnkjerjrh")),
        };

        // Generate viewing key for staking contract
        let entropy: String = staking.entropy.clone().unwrap();
        let (staking_contract_vk, new_seed) =
            new_viewing_key(&info.sender.clone(), &env, &prnd.0, entropy.as_ref());

        // Generate viewing key for SHD contract
        let entropy: String = token.entropy.clone().unwrap();
        let (token_contract_vk, _new_seed) =
            new_viewing_key(&info.sender.clone(), &env, &new_seed, entropy.as_ref());

        let msgs: Vec<CosmosMsg> = vec![
            // Register receive Derivative contract
            register_receive_msg(
                env.contract.code_hash.clone(),
                derivative.entropy,
                RESPONSE_BLOCK_SIZE,
                derivative.code_hash,
                derivative.address.to_string(),
            )
            .unwrap(),
            // Register receive SHD contract
            register_receive_msg(
                env.contract.code_hash,
                token.entropy.clone(),
                RESPONSE_BLOCK_SIZE,
                token.code_hash.clone(),
                token.address.to_string(),
            )
            .unwrap(),
            // Set viewing key for SHD
            set_viewing_key_msg(
                token_contract_vk,
                token.entropy,
                RESPONSE_BLOCK_SIZE,
                token.code_hash,
                token.address.to_string(),
            )
            .unwrap(),
            // Set viewing key for staking contract
            set_viewing_key_msg(
                staking_contract_vk,
                authentication_contract.entropy,
                RESPONSE_BLOCK_SIZE,
                authentication_contract.code_hash,
                authentication_contract.address.to_string(),
            )
            .unwrap(),
        ];
        assert_eq!(init_result.unwrap(), Response::default().add_messages(msgs));

        assert_eq!(
            CONTRACT_STATUS.load(&deps.storage).unwrap(),
            ContractStatusLevel::NormalRun
        );
    }

    #[test]
    fn test_panic_messages_when_contract_panicked() {
        let (init_result, mut deps) = init_helper();
        let info = mock_info("admin", &[]);
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::SetContractStatus {
            level: ContractStatusLevel::Panicked,
            padding: None,
        };
        let handle_result = execute(deps.as_mut(), mock_env(), info.clone(), handle_msg);
        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::PanicWithdraw {};
        let handle_result = execute(deps.as_mut(), mock_env(), info, handle_msg);
        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );
    }

    #[test]
    fn test_handle_set_contract_status() {
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::SetContractStatus {
            level: ContractStatusLevel::StopAll,
            padding: None,
        };
        let info = mock_info("admin", &[]);

        let handle_result = execute(deps.as_mut(), mock_env(), info, handle_msg);

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );

        let contract_status = CONTRACT_STATUS.load(&deps.storage).unwrap();
        assert!(matches!(
            contract_status,
            ContractStatusLevel::StopAll { .. }
        ));
    }
    #[test]
    fn test_receive_msg_sender_is_not_shd_contract() {
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked(""),
            from: Addr::unchecked(""),
            amount: Uint256::from(100000000 as u32),
            msg: Some(to_binary(&ReceiverMsg::Stake {}).unwrap()),
        };
        let info = mock_info("giannis", &[]);

        let handle_result = execute(deps.as_mut(), mock_env(), info, handle_msg);

        assert!(handle_result.is_err());
        let error = extract_error_msg(handle_result);

        assert_eq!(error, "Sender is not SHD contract");
    }

    #[test]
    fn test_receive_stake_msg_successfully() {
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked(""),
            from: Addr::unchecked(""),
            amount: Uint256::from(100000000 as u32),
            msg: Some(to_binary(&ReceiverMsg::Stake {}).unwrap()),
        };
        let info = mock_info("shade_contract_info_address", &[]);

        let handle_result = execute(deps.as_mut(), mock_env(), info, handle_msg);

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );
    }

    #[test]
    fn test_receive_unbond_msg_sender_is_not_derivative_contract() {
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked(""),
            from: Addr::unchecked(""),
            amount: Uint256::from(100000000 as u32),
            msg: Some(to_binary(&ReceiverMsg::Unbond {}).unwrap()),
        };
        let info = mock_info("giannis", &[]);

        let handle_result = execute(deps.as_mut(), mock_env(), info, handle_msg);

        assert!(handle_result.is_err());
        let error = extract_error_msg(handle_result);

        assert_eq!(error, "Sender is not derivative (SNIP20) contract");
    }

    #[test]
    fn test_receive_unbond_msg_successfully() {
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked(""),
            from: Addr::unchecked(""),
            amount: Uint256::from(100000000 as u32),
            msg: Some(to_binary(&ReceiverMsg::Unbond {}).unwrap()),
        };
        let info = mock_info("derivative_snip20_info_address", &[]);

        let handle_result = execute(deps.as_mut(), mock_env(), info, handle_msg);

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );
    }

    #[test]
    fn test_receive_transfer_staked_msg_successfully() {
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked(""),
            from: Addr::unchecked(""),
            amount: Uint256::from(100000000 as u32),
            msg: Some(to_binary(&ReceiverMsg::TransferStaked { receiver: None }).unwrap()),
        };
        let info = mock_info("derivative_snip20_info_address", &[]);

        let handle_result = execute(deps.as_mut(), mock_env(), info, handle_msg);

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );
    }

    #[test]
    fn test_receive_transfer_staked_msg_sender_is_not_derivative_contract() {
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked(""),
            from: Addr::unchecked(""),
            amount: Uint256::from(100000000 as u32),
            msg: Some(to_binary(&ReceiverMsg::TransferStaked { receiver: None }).unwrap()),
        };
        let info = mock_info("giannis", &[]);

        let handle_result = execute(deps.as_mut(), mock_env(), info, handle_msg);

        assert!(handle_result.is_err());
        let error = extract_error_msg(handle_result);

        assert_eq!(error, "Sender is not derivative (SNIP20) contract");
    }
    #[test]
    fn test_unbonding_query_not_unbonds() {
        let (init_result, deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );
        // Query unbondings
        let query_msg = QueryMsg::Unbondings {
            address: Addr::unchecked("david"),
            viewing_key: String::from("password"),
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg);
        let unbonds = match from_binary(&query_result.unwrap()).unwrap() {
            QueryAnswer::Unbondings { unbonds } => unbonds,
            other => panic!("Unexpected: {:?}", other),
        };

        assert_eq!(unbonds, vec![]);
    }

    #[test]
    fn test_holdings_query_not_funds() {
        let (init_result, deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );
        // Query unbondings
        let query_msg = QueryMsg::Holdings {
            address: Addr::unchecked("david"),
            viewing_key: String::from("password"),
        };
        let query_result = query(deps.as_ref(), mock_env(), query_msg);
        let (derivative_claimable, derivative_unbonding) =
            match from_binary(&query_result.unwrap()).unwrap() {
                QueryAnswer::Holdings {
                    derivative_claimable,
                    derivative_unbonding,
                } => (derivative_claimable, derivative_unbonding),
                other => panic!("Unexpected: {:?}", other),
            };

        assert_eq!(derivative_claimable, Uint128::zero());
        assert_eq!(derivative_unbonding, Uint128::zero());
    }

    #[test]
    fn test_sanity_unbonding_processing_storage() {
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );
        let env = mock_env();
        // Unbond from bob account
        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked(""),
            from: Addr::unchecked("bob"),
            amount: Uint256::from(100000000 as u32),
            msg: Some(to_binary(&ReceiverMsg::Unbond {}).unwrap()),
        };
        let info = mock_info("derivative_snip20_info_address", &[]);

        let handle_result = execute(deps.as_mut(), mock_env(), info, handle_msg);

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );

        let unbonding_processing = PENDING_UNBONDING.load(&deps.storage).unwrap();

        assert_eq!(
            unbonding_processing,
            InProcessUnbonding {
                id: Uint128::zero(),
                owner: Addr::unchecked("bob"),
                amount: Uint128::from(21375000000000_u128),
                complete: Uint128::from(env.block.time.seconds())
                    .checked_add(Uint128::from(300_u32))
                    .unwrap(),
            }
        )
    }

    #[test]
    fn test_update_fees_should_fail_no_admin_sender() {
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::UpdateFees {
            staking: None,
            unbonding: None,
            collector: None,
        };
        let info = mock_info("not_admin", &[]);

        let handle_result = execute(deps.as_mut(), mock_env(), info, handle_msg);

        assert!(handle_result.is_err());
        let error = extract_error_msg(handle_result);

        assert_eq!(
            error,
            "This is an admin command. Admin commands can only be run from admin address"
        );
    }

    #[test]
    fn test_update_fees_successfully_sender_is_admin_no_new_config_provided() {
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );
        let config_before_tx = CONFIG.load(&deps.storage).unwrap();
        let handle_msg = ExecuteMsg::UpdateFees {
            staking: None,
            unbonding: None,
            collector: None,
        };
        let info = mock_info("admin", &[]);

        let handle_result = execute(deps.as_mut(), mock_env(), info, handle_msg);

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );
        let config_after_tx = CONFIG.load(&deps.storage).unwrap();

        assert_eq!(config_before_tx.fees, config_after_tx.fees);
    }

    #[test]
    fn test_update_fees_successfully_sender_is_admin() {
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );
        let config_before_tx = CONFIG.load(&deps.storage).unwrap();
        let handle_msg = ExecuteMsg::UpdateFees {
            staking: Some(Fee {
                rate: 5_u32,
                decimal_places: 2_u8,
            }),
            collector: Some(Addr::unchecked("new_collector")),
            unbonding: None,
        };
        let info = mock_info("admin", &[]);

        let handle_result = execute(deps.as_mut(), mock_env(), info, handle_msg);

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );
        let config_after_tx = CONFIG.load(&deps.storage).unwrap();

        assert_ne!(config_before_tx.fees, config_after_tx.fees);

        let answer: ExecuteAnswer = from_binary(&handle_result.unwrap().data.unwrap()).unwrap();
        let fee_info_returned = match answer {
            ExecuteAnswer::UpdateFees { fee, status: _ } => fee,
            _ => panic!("NOPE"),
        };
        let fees = CONFIG.load(&deps.storage).unwrap().fees;

        assert_eq!(fee_info_returned, fees)
    }

    #[test]
    fn test_staking_returned_tokens() {
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::Receive {
            sender: Addr::unchecked(""),
            from: Addr::unchecked("bob"),
            amount: Uint256::from(300000000 as u32),
            msg: Some(to_binary(&ReceiverMsg::Stake {}).unwrap()),
        };
        let info = mock_info("shade_contract_info_address", &[]);

        let handle_result = execute(deps.as_mut(), mock_env(), info, handle_msg);

        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );
        let expected_tokens_return = Uint128::from(2850_u128);
        let (_, tokens_returned) = match from_binary(&handle_result.unwrap().data.unwrap()).unwrap()
        {
            ExecuteAnswer::Stake {
                shd_staked,
                tokens_returned,
            } => (shd_staked, tokens_returned),
            other => panic!("Unexpected: {:?}", other),
        };
        assert_eq!(tokens_returned, expected_tokens_return)
    }

    #[test]
    fn test_staking_info_query() {
        let (init_result, deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );
        let query_msg = QueryMsg::StakingInfo {};
        let query_result = query(deps.as_ref(), mock_env(), query_msg);
        let (
            unbonding_time,
            bonded_shd,
            available_shd,
            rewards,
            total_derivative_token_supply,
            price,
        ) = match from_binary(&query_result.unwrap()).unwrap() {
            QueryAnswer::StakingInfo {
                unbonding_time,
                bonded_shd,
                available_shd,
                rewards,
                total_derivative_token_supply,
                price,
            } => (
                unbonding_time,
                bonded_shd,
                available_shd,
                rewards,
                total_derivative_token_supply,
                price,
            ),
            other => panic!("Unexpected: {:?}", other),
        };

        assert_eq!(unbonding_time, Uint128::from(300_u32));
        assert_eq!(bonded_shd, Uint128::from(300000000_u128));
        assert_eq!(available_shd, Uint128::from(100000000_u128));
        assert_eq!(rewards, Uint128::from(100000000_u128));
        assert_eq!(total_derivative_token_supply, Uint128::from(2000_u128));
        assert_eq!(price, Uint128::from(250000000000_u128));
    }

    #[test]
    fn test_fee_info() {
        let (init_result, deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );
        let query_msg = QueryMsg::FeeInfo {};
        let query_result = query(deps.as_ref(), mock_env(), query_msg);
        let (staking, unbonding, collector) = match from_binary(&query_result.unwrap()).unwrap() {
            QueryAnswer::FeeInfo {
                staking,
                unbonding,
                collector,
            } => (staking, unbonding, collector),
            other => panic!("Unexpected: {:?}", other),
        };

        assert_eq!(
            staking,
            Fee {
                rate: 5,
                decimal_places: 2_u8,
            }
        );
        assert_eq!(
            unbonding,
            Fee {
                rate: 5,
                decimal_places: 2_u8,
            }
        );

        assert_eq!(collector, Addr::unchecked("collector_address"));
    }
    #[test]
    fn test_handle_claim_not_mature_unbonds() {
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::Claim {};
        let info = mock_info("x", &[]);

        let handle_result = execute(deps.as_mut(), mock_env(), info, handle_msg);
        assert!(handle_result.is_err());
        let error = extract_error_msg(handle_result);

        assert_eq!(error, "No mature unbondings to claim");
    }

    #[test]
    fn test_handle_compound_rewards_msg() {
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::CompoundRewards {};
        let info = mock_info("x", &[]);

        let handle_result = execute(deps.as_mut(), mock_env(), info, handle_msg);
        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );

        let config = CONFIG.load(&deps.storage).unwrap();

        let msgs = vec![
            compound_msg(config.staking.code_hash, config.staking.address.to_string()).unwrap(),
        ];

        assert_eq!(
            handle_result.unwrap(),
            Response::default().add_messages(msgs)
        );
    }

    #[test]
    fn test_handle_panic_unbond_not_admin_user() {
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );
        let handle_msg = ExecuteMsg::PanicUnbond {
            amount: Uint128::from(100000000_u128),
        };
        let info = mock_info("bob", &[]);

        let handle_result = execute(deps.as_mut(), mock_env(), info, handle_msg);

        assert!(handle_result.is_err());
        let error = extract_error_msg(handle_result);

        assert_eq!(
            error,
            "This is an admin command. Admin commands can only be run from admin address"
        );
    }

    #[test]
    fn test_handle_panic_unbond_msg() {
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::PanicUnbond {
            amount: Uint128::from(100000000_u128),
        };
        let info = mock_info("admin", &[]);

        let handle_result = execute(deps.as_mut(), mock_env(), info, handle_msg);
        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );

        let config = CONFIG.load(&deps.storage).unwrap();

        let msg = SubMsg::reply_always(
            unbond_msg(
                Uint128::from(100000000_u128),
                config.staking.code_hash,
                config.staking.address.to_string(),
                Some(false),
            )
            .unwrap(),
            PANIC_UNBOND_REPLY_ID,
        );

        assert_eq!(
            handle_result.unwrap(),
            Response::default().add_submessage(msg)
        );
    }

    #[test]
    fn test_handle_panic_withdraw_not_admin_user() {
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );
        let handle_msg = ExecuteMsg::PanicWithdraw {};
        let info = mock_info("bob", &[]);

        let handle_result = execute(deps.as_mut(), mock_env(), info, handle_msg);

        assert!(handle_result.is_err());
        let error = extract_error_msg(handle_result);

        assert_eq!(
            error,
            "This is an admin command. Admin commands can only be run from admin address"
        );
    }

    #[test]
    fn test_handle_panic_withdraw_msg() {
        let (init_result, mut deps) = init_helper();
        assert!(
            init_result.is_ok(),
            "Init failed: {}",
            init_result.err().unwrap()
        );

        let handle_msg = ExecuteMsg::PanicWithdraw {};
        let info = mock_info("admin", &[]);

        let handle_result = execute(deps.as_mut(), mock_env(), info, handle_msg);
        assert!(
            handle_result.is_ok(),
            "handle() failed: {}",
            handle_result.err().unwrap()
        );

        let config = CONFIG.load(&deps.storage).unwrap();
        let rewards = 100000000_u128;
        let balance = 100000000_u128;
        let amount = Uint128::from(rewards + balance);
        assert_eq!(
            handle_result.unwrap(),
            Response::default().add_messages(vec![
                claim_rewards_msg(
                    config.staking.code_hash.clone(),
                    config.staking.address.to_string(),
                )
                .unwrap(),
                send_msg(
                    Addr::unchecked("super_admin").to_string(),
                    amount,
                    None,
                    Some("Panic withdraw {} tokens".to_string()),
                    config.token.entropy,
                    RESPONSE_BLOCK_SIZE,
                    config.token.code_hash,
                    config.token.address.to_string(),
                )
                .unwrap(),
            ])
        );
    }
}
