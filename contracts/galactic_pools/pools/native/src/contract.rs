//Crate Import
use crate::{
    constants::*,
    helper::*,
    msg::{
        space_pad,
        ContractConfigResponse,
        ContractStatus,
        ContractStatusResponse,
        CurrentRewardsResponse,
        DelegatedResponse,
        ExpContract,
        GalacticPoolsPermissions,
        HandleAnswer,
        HandleMsg,
        InstantiateMsg,
        LiquidityResponse,
        PoolStateInfoResponse,
        PoolStateLiquidityStatsResponse,
        QueryMsg,
        QueryWithPermit,
        RecordsResponse,
        RemoveSponsorCredentialsDecisions,
        RequestWithdrawQueryResponse,
        ResponseStatus::Success,
        Review,
        RewardStatsResponse,
        RoundResponse,
        SponsorDisplayInfo,
        SponsorInfoResponse,
        SponsorMessageRequestResponse,
        SponsorsResponse,
        UnbondingsResponse,
        UserInfoResponse,
        ValidatorInfo,
        ViewingKeyErrorResponse,
        WithdrawablelResponse,
    },
    rand::sha_256,
    staking::{
        // get_exp,
        get_rewards,
        redelegate,
        stake,
        undelegate,
        withdraw,
    },
    state::{
        AdminShareInfo,
        ConfigInfo,
        DigitsInfo,
        GlobalSponsorDisplayRequestListState,
        GlobalSponsorState,
        PoolLiqState,
        PoolState,
        RewardsClaimed,
        RewardsDistInfo,
        RewardsPerTierInfo,
        RewardsState,
        RoundInfo,
        SponsorInfo,
        TierCounter,
        TierLog,
        TierState,
        UnbondingBatch,
        UnclaimedDistInfo,
        UserInfo,
        UserLiqState,
        UserRewardsLog,
        Validator,
        WinningSequence,
    },
    viewing_key::{ViewingKey, VIEWING_KEY_SIZE},
};

use shade_protocol::{
    c_std::{
        entry_point,
        to_binary,
        Addr,
        BankMsg,
        Binary,
        Coin,
        CosmosMsg,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
        Storage,
        Uint128,
    },
    s_toolkit::permit::{validate, Permit, RevokedPermits},
};

//Cosmwasm import

//Secret toolkit Import

//Rust functions
use rand::{distributions::Uniform, prelude::*, SeedableRng};
use rand_chacha::ChaChaRng;
use sha2::{Digest, Sha256};
use std::ops::{Add, AddAssign};

/// pad handle responses and log attributes to blocks of 256 bytes to prevent leaking info based on
/// We make sure that responses from `handle` are padded to a multiple of this size.
pub const BLOCK_SIZE: usize = 256;

////////////////////////////////////// Instantiate ///////////////////////////////////////
/// Returns StdResult<Response>
///
/// Initializes the contract
///
/// # Arguments
///
/// * `deps` - Mutable reference to Extern containing all the contract's external dependencies
/// * `env`  - Env of contract's environment
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `msg`  - InitMsg passed in with the instantiation message
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // Initialize state
    // Ensuring that the validator is registered
    let queried_vals = deps.querier.query_all_validators()?;

    let val_struct_vec: Vec<Validator> = msg
        .validator
        .into_iter()
        .filter_map(|validator| {
            if queried_vals
                .iter()
                .any(|v| v.address.as_str() == validator.address.as_str())
            {
                Some(Validator {
                    address: validator.address,
                    delegated: Uint128::zero(),
                    weightage: validator.weightage,
                    percentage_filled: 0,
                })
            } else {
                None
            }
        })
        .collect();

    //Storing CanonicalAddr only

    let mut admins = Vec::new();

    if let Some(ad) = msg.admins {
        for admin in ad {
            admins.push(admin)
        }
    } else {
        let ad = info.sender.clone();
        admins.push(ad)
    }

    let mut triggerers = Vec::new();
    if let Some(tri) = msg.triggerers {
        for triggerer in tri {
            triggerers.push(triggerer)
        }
    } else {
        let tr = info.sender.clone();
        triggerers.push(tr)
    }

    let mut reviewers = Vec::new();
    if let Some(revs) = msg.reviewers {
        for reviewer in revs {
            let rv = reviewer;
            reviewers.push(rv)
        }
    } else {
        let rv = info.sender.clone();
        reviewers.push(rv)
    }

    let prng_seed_hashed = sha_256(&msg.prng_seed.0);
    let prng_seed_hashed_twice = sha_256(&prng_seed_hashed);
    let config_obj = ConfigInfo {
        admins,
        triggerers,
        common_divisor: msg.common_divisor,
        denom: msg.denom,
        prng_seed: prng_seed_hashed.to_vec(),
        contract_address: env.contract.address,
        validators: val_struct_vec,
        next_validator_for_delegation: 0,
        next_validator_for_unbonding: 0,
        next_unbonding_batch_time: env.block.time.seconds().add(msg.unbonding_batch_duration),
        next_unbonding_batch_amount: Uint128::zero(),
        next_unbonding_batch_index: 1u64,
        unbonding_duration: msg.unbonding_duration,
        unbonding_batch_duration: msg.unbonding_batch_duration,
        minimum_deposit_amount: msg.minimum_deposit_amount,
        status: ContractStatus::Normal.to_u8(),
        reviewers,
        sponsor_msg_edit_fee: msg.sponsor_msg_edit_fee,
        exp_contract: msg.exp_contract.clone(),
    };

    let mut message: Vec<CosmosMsg> = Vec::new();
    //TODO: check this
    if msg.exp_contract.is_some() {
        // if let Some(exp_contract) = msg.exp_contract {
        //     let set_vk_msg;
        //     set_vk_msg = experience_contract::msg::ExecuteMsg::SetViewingKey {
        //         key: exp_contract.vk,
        //     };
        //     message.push(set_vk_msg.to_cosmos_msg(
        //         BLOCK_SIZE,
        //         exp_contract.contract.hash,
        //         exp_contract.contract.address,
        //         None,
        //     )?);
        // } else {
        //     return Err(StdError::generic_err(
        //         "No viewing key provided for the experience contract",
        //     ));
        // }
    }
    CONFIG_STORE.save(deps.storage, &config_obj)?;

    //Starting first round
    // the entropy is hashed again to obtain the seed field value. As you can see, the entropy and seed fields have different values, enhancing the security of the random number generation process.
    let round_obj = RoundInfo {
        entropy: prng_seed_hashed_twice.to_vec(),
        seed: prng_seed_hashed.to_vec(),
        duration: msg.round_duration,
        start_time: env.block.time.seconds(),
        end_time: env.block.time.seconds().add(msg.round_duration),
        rewards_distribution: msg.rewards_distribution,
        current_round_index: 1u64,
        ticket_price: msg.ticket_price,
        rewards_expiry_duration: msg.rewards_expiry_duration,
        admin_share: AdminShareInfo {
            total_percentage_share: msg.total_admin_share,
            shade_percentage_share: msg.shade_percentage_share,
            galactic_pools_percentage_share: msg.galactic_pools_percentage_share,
        },
        triggerer_share_percentage: msg.triggerer_share_percentage,
        shade_rewards_address: msg.shade_rewards_address,
        galactic_pools_rewards_address: msg.galactic_pools_rewards_address,
        unclaimed_rewards_last_claimed_round: None,
        unclaimed_distribution: UnclaimedDistInfo {
            reserves_percentage: msg.reserve_percentage,
            propagate_percentage: msg
                .common_divisor
                .checked_sub(msg.reserve_percentage)
                .ok_or_else(|| StdError::generic_err("Under-flow sub error"))?,
        },
        grand_prize_address: msg.grand_prize_address,
        number_of_tickers_per_transaction: msg.number_of_tickers_per_transaction,
    };
    ROUND_STORE.save(deps.storage, &round_obj)?;

    let pool_state_obj = PoolState {
        total_delegated: Uint128::zero(),
        rewards_returned_to_contract: Uint128::zero(),
        total_reserves: Uint128::zero(),
        total_sponsored: Uint128::zero(),
        unbonding_batches: Vec::new(),
    };
    POOL_STATE_STORE.save(deps.storage, &pool_state_obj)?;

    let sponsor_stats_obj = GlobalSponsorState {
        offset: 0,
        empty_slots: Vec::new(),
    };
    SPONSOR_STATS_STORE.save(deps.storage, &sponsor_stats_obj)?;

    Ok(Response::new()
        .add_messages(message)
        .set_data(to_binary(&HandleAnswer::Initialize { status: Success })?))
}

fn pad_response(response: StdResult<Response>) -> StdResult<Response> {
    response.map(|mut response| {
        response.data = response.data.map(|mut data| {
            space_pad(BLOCK_SIZE, &mut data.0);
            data
        });
        response
    })
}

///////////////////////////////////// Handle //////////////////////////////////////
/// Returns StdResult<Response>
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `msg` - HandleMsg passed in with the execute message
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: HandleMsg) -> StdResult<Response> {
    let mut config: ConfigInfo = config_helper_read_only(deps.storage)?;

    let response = match msg {
        // User
        HandleMsg::Deposit { .. } => {
            try_deposit(deps, env, info, &mut config, ContractStatus::Normal.to_u8())
        }
        HandleMsg::RequestWithdraw { amount, .. } => try_request_withdraw(
            deps,
            env,
            info,
            &mut config,
            amount,
            ContractStatus::StopTransactions.to_u8(),
        ),
        HandleMsg::Withdraw { amount } => try_withdraw(
            deps,
            env,
            info,
            &mut config,
            amount,
            ContractStatus::StopTransactions.to_u8(),
        ),
        HandleMsg::ClaimRewards {} => try_claim_rewards(
            deps,
            env,
            info,
            &mut config,
            ContractStatus::StopTransactions.to_u8(),
        ),
        HandleMsg::CreateViewingKey { entropy, .. } => try_create_viewing_key(
            deps,
            env,
            info,
            &mut config,
            entropy,
            ContractStatus::Normal.to_u8(),
        ),
        HandleMsg::SetViewingKey { key, .. } => {
            try_set_viewing_key(deps, info, &mut config, key, ContractStatus::Normal.to_u8())
        }

        HandleMsg::RevokePermit { permit_name, .. } => try_revoke_permit(
            deps.storage,
            &info.sender.as_str(),
            &permit_name,
            ContractStatus::StopTransactions.to_u8(),
            &config,
        ),

        // Sponsor
        HandleMsg::Sponsor { title, message, .. } => try_sponsor(
            deps,
            env,
            info,
            title,
            message,
            &mut config,
            ContractStatus::Normal.to_u8(),
        ),
        HandleMsg::SponsorRequestWithdraw { amount, .. } => try_sponsor_request_withdraw(
            deps,
            info,
            &mut config,
            amount,
            ContractStatus::StopTransactions.to_u8(),
        ),
        HandleMsg::SponsorWithdraw { amount } => try_sponsor_withdraw(
            deps,
            env,
            info,
            &mut config,
            amount,
            ContractStatus::StopTransactions.to_u8(),
        ),
        HandleMsg::SponsorMessageEdit {
            title,
            message,
            delete_title,
            delete_message,
            ..
        } => try_sponsor_message_edit(
            deps,
            info,
            title,
            message,
            delete_title,
            delete_message,
            &mut config,
            ContractStatus::StopTransactions.to_u8(),
        ),

        // Triggerer
        HandleMsg::EndRound {} => {
            try_end_round(deps, env, info, &mut config, ContractStatus::Normal.to_u8())
        }

        // Admin
        HandleMsg::UpdateConfig {
            unbonding_batch_duration,
            unbonding_duration,
            minimum_deposit_amount,
            exp_contract,
        } => try_update_config(
            deps,
            info,
            env,
            &mut config,
            unbonding_batch_duration,
            unbonding_duration,
            minimum_deposit_amount,
            exp_contract,
        ),

        HandleMsg::UpdateRound {
            duration,
            rewards_distribution,
            ticket_price,
            rewards_expiry_duration,
            admin_share,
            triggerer_share_percentage,
            shade_rewards_address,
            galactic_pools_rewards_address,
            grand_prize_address,
            unclaimed_distribution,
        } => try_update_round(
            deps,
            info,
            &mut config,
            duration,
            rewards_distribution,
            ticket_price,
            rewards_expiry_duration,
            admin_share,
            triggerer_share_percentage,
            shade_rewards_address,
            galactic_pools_rewards_address,
            grand_prize_address,
            unclaimed_distribution,
        ),
        HandleMsg::AddAdmin { admin } => try_add_admin(deps, info, &mut config, admin),
        HandleMsg::RemoveAdmin { admin } => try_remove_admin(deps, info, &mut config, admin),

        HandleMsg::AddTriggerer { triggerer } => {
            try_add_triggerer(deps, info, &mut config, triggerer)
        }
        HandleMsg::RemoveTriggerer { triggerer } => {
            try_remove_triggerer(deps, info, &mut config, triggerer)
        }

        HandleMsg::AddReviewer { reviewer } => try_add_reviewer(deps, info, &mut config, reviewer),
        HandleMsg::RemoveReviewer { reviewer } => {
            try_remove_reviewer(deps, info, &mut config, reviewer)
        }

        HandleMsg::UpdateValidatorSet {
            updated_validator_set,
        } => try_update_validator_set(deps, env, info, &mut config, updated_validator_set),
        HandleMsg::RebalanceValidatorSet {} => {
            try_rebalance_validator_set(deps, env, info, &mut config)
        }
        HandleMsg::SetContractStatus { level, .. } => {
            try_set_contract_status(deps, info, &mut config, level)
        }

        HandleMsg::RequestReservesWithdraw { amount, .. } => {
            try_request_reserves_withdraw(deps, info, &mut config, amount)
        }
        HandleMsg::ReservesWithdraw { amount, .. } => {
            try_reserve_withdraw(deps, env, info, &mut config, amount)
        }
        HandleMsg::ReviewSponsors { decisions, .. } => {
            try_review_sponsor_messages(deps, info, decisions, &config)
        }

        HandleMsg::RemoveSponsorCredentials { decisions, .. } => {
            try_remove_sponsor_credentials(deps, info, decisions, &mut config)
        }

        HandleMsg::UnbondBatch { .. } => try_unbond_batch(deps, env, info, &mut config),

        _ => Err(StdError::generic_err("Unavailable or unknown action")),
    };

    pad_response(response)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let response = match msg {
        QueryMsg::ContractConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::ContractStatus {} => to_binary(&query_contract_status(deps)?),
        QueryMsg::CurrentRewards {} => to_binary(&query_current_rewards(deps, _env)?),
        QueryMsg::Round {} => to_binary(&query_round(deps)?),
        QueryMsg::PoolState {} => to_binary(&query_pool_state_info(deps)?),
        QueryMsg::PoolStateLiquidityStats {} => to_binary(&query_pool_state_liquidity_stats(deps)?),
        QueryMsg::RewardsStats {} => to_binary(&query_reward_stats(deps)?),
        QueryMsg::PoolStateLiquidityStatsSpecific { round_index } => to_binary(
            &query_pool_state_liquidity_stats_specific(deps, round_index)?,
        ),
        QueryMsg::SponsorMessageRequestCheck {
            start_page,
            page_size,
        } => to_binary(&query_sponsor_message_req_check(
            deps, start_page, page_size,
        )?),
        QueryMsg::Sponsors {
            page_size,
            start_page,
        } => to_binary(&query_sponsors(deps, start_page, page_size)?),

        QueryMsg::WithPermit { permit, query } => permit_queries(deps, permit, query, _env),
        // QueryMsg::PastRecords {} => query_past_records(&deps),
        // QueryMsg::PastAllRecords {} => query_all_past_records(&deps),
        _ => authenticated_queries(deps, _env, msg),
    };
    response
}

///////////////////////////////////////// User Functions //////////////////////////////////////

/// Returns StdResult<Response>
///
/// User deposit their tokens here
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a mutable reference to the Config
/// * `priority` - u8 representation of highest ContractStatus level this action is permitted
fn try_deposit(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut config: &mut ConfigInfo,
    priority: u8,
) -> StdResult<Response> {
    // Checking if the contract is in the correct status to perform this handle function.
    check_status(config.status, priority)?;

    let deposit_amount: Uint128;

    // Checking if the deposited amount is valid
    deposit_amount = check_if_valid_amount(&info, &config)?;

    // Loading general use read-only stores
    let round_obj: RoundInfo = round_helper_read_only(deps.storage)?;
    // 1) Updating user_info and user_liquidity_snapshot objects
    //Loading readonly stores
    let mut user_info_obj: UserInfo = user_info_helper_read_only(deps.storage, &info.sender)?;

    user_info_obj.starting_round = Some(round_obj.current_round_index);
    let mut user_liquidity_state: UserLiqState = user_liquidity_snapshot_stats_helper_read_only(
        deps.storage,
        round_obj.current_round_index,
        &info.sender,
    )?;

    // Fetching liquidity
    let liquidity = user_liquidity_state
        .liquidity
        .unwrap_or(user_info_obj.amount_delegated);

    // Only adding to liquidity if the round has not ended yet
    if env.block.time.seconds() >= round_obj.end_time {
        user_liquidity_state.liquidity = Some(liquidity);
    } else {
        let time_remaining_in_round = round_obj
            .end_time
            .checked_sub(env.block.time.seconds())
            .ok_or_else(|| StdError::generic_err("Under-flow sub error"))?;
        user_liquidity_state.liquidity = Some(
            liquidity
                .add(deposit_amount.multiply_ratio(time_remaining_in_round, round_obj.duration)),
        );
    }

    //Storing user_info_obj and user_liquidity_snapshot
    user_info_obj.amount_delegated.add_assign(deposit_amount);
    user_info_helper_store(deps.storage, &info.sender, &user_info_obj)?;

    user_liquidity_state.amount_delegated = Some(user_info_obj.amount_delegated);
    user_liquidity_snapshot_stats_helper_store(
        deps.storage,
        round_obj.current_round_index,
        &info.sender,
        user_liquidity_state,
    )?;

    //2) Updating PoolState, PoolStateLiquidityStats, and Config
    //Loading readonly stores
    let mut pool_state_obj: PoolState = pool_state_helper_read_only(deps.storage)?;
    let mut pool_liquidity_state: PoolLiqState =
        pool_state_liquidity_helper_read_only(deps.storage, round_obj.current_round_index)?;

    //Updating PoolStateLiquidityStats
    //Fetching Liquidity
    let pool_liquidity = pool_liquidity_state
        .total_liquidity
        .unwrap_or(pool_state_obj.total_delegated);

    //Adding to liquidity if round has not ended yet
    if env.block.time.seconds() >= round_obj.end_time {
        pool_liquidity_state.total_liquidity = Some(pool_liquidity);
    } else {
        let time_remaining_in_round = round_obj
            .end_time
            .checked_sub(env.block.time.seconds())
            .ok_or_else(|| StdError::generic_err("Under-flow sub error"))?;
        pool_liquidity_state.total_liquidity = Some(
            pool_liquidity
                .add(deposit_amount.multiply_ratio(time_remaining_in_round, round_obj.duration)),
        );
    }

    let total_delegated = pool_state_obj.total_delegated.add(deposit_amount);
    pool_liquidity_state.total_delegated = Some(total_delegated);
    //Storing pool_state_liquidity_snapshot
    pool_state_liquidity_helper_store(
        deps.storage,
        round_obj.current_round_index,
        pool_liquidity_state,
    )?;

    //Selecting Validator for deposit and updating the validator information
    let selected_val_index = config.next_validator_for_delegation as usize;
    config.validators[selected_val_index]
        .delegated
        .add_assign(deposit_amount);

    if selected_val_index == config.validators.len() - 1 {
        config.next_validator_for_delegation = 0;
    } else {
        config.next_validator_for_delegation += 1;
    }
    //Updating validators stats
    config_helper_store(deps.storage, &config)?;

    //Querying pending rewards sent back from the validator
    let rewards = get_rewards(deps.as_ref(), &env.contract.address, &config)?;
    let mut rewards_amount = Uint128::zero();

    for reward in rewards {
        if reward.validator_address.as_str()
            == config.validators[selected_val_index].address.as_str()
        {
            rewards_amount.add_assign(reward.reward);
            break;
        }
    }

    //Updating: PoolState
    pool_state_obj.total_delegated = total_delegated;
    pool_state_obj
        .rewards_returned_to_contract
        .add_assign(rewards_amount);

    //Storing PoolState
    pool_state_helper_store(deps.storage, &pool_state_obj)?;

    //Staking to the selected validator
    let mut messages = Vec::new();
    messages.push(stake(
        &config.validators[selected_val_index].address,
        deposit_amount,
        &config.denom,
    ));

    Ok(Response::new()
        .add_messages(messages)
        .set_data(to_binary(&HandleAnswer::Deposit { status: Success })?))
}

/// Returns StdResult<Response>
///
/// User request to withdraw their funds. It take 21 days to unbond the funds.
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a mutable reference to the Config
/// * `request_withdraw_amount` - amount requested to withdraw
/// * `priority` - u8 representation of highest ContractStatus level this action is permitted
fn try_request_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: &mut ConfigInfo,
    request_withdraw_amount: Uint128,
    priority: u8,
) -> StdResult<Response> {
    //CHECKING status
    check_status(config.status, priority)?;

    //Loading data
    let round_obj = round_helper_read_only(deps.storage)?;
    let mut user_info = user_info_helper_read_only(deps.storage, &info.sender)?;
    let mut user_liquidity_snapshot_obj = user_liquidity_snapshot_stats_helper_read_only(
        deps.storage,
        round_obj.current_round_index,
        &info.sender,
    )?;
    let mut pool_state: PoolState = pool_state_helper_read_only(deps.storage)?;
    let mut pool_state_liquidity_snapshot_obj: PoolLiqState =
        pool_state_liquidity_helper_read_only(deps.storage, round_obj.current_round_index)?;

    // Checking: If the amount unbonded is not greater than amount delegated
    if user_info.amount_delegated < request_withdraw_amount {
        return Err(StdError::generic_err(format!(
            "insufficient funds to redeem: balance={}, required={}",
            user_info.amount_delegated, request_withdraw_amount
        )));
    }

    if request_withdraw_amount == Uint128::zero() {
        return Err(StdError::generic_err(format!(
            "cannot request to withdraw 0 {}. Please specify a non-zero amount",
            config.denom
        )));
    }
    //STORING USER UNBONDING
    if !user_info
        .unbonding_batches
        .contains(&config.next_unbonding_batch_index)
    {
        user_info
            .unbonding_batches
            .push(config.next_unbonding_batch_index);
    }

    let mut unbonding_amount = user_unbond_helper_read_only(
        deps.storage,
        config.next_unbonding_batch_index,
        &info.sender,
    )?;

    unbonding_amount.add_assign(request_withdraw_amount);

    user_unbond_helper_store(
        deps.storage,
        config.next_unbonding_batch_index,
        &info.sender,
        unbonding_amount,
    )?;

    //If the liquidity struct for that round is not generated then we create one and store
    let user_liquidity: Uint128;
    if user_liquidity_snapshot_obj.liquidity.is_none() {
        user_liquidity = user_info.amount_delegated;
    } else {
        user_liquidity = user_liquidity_snapshot_obj.liquidity.unwrap();
    }

    if env.block.time.seconds() >= round_obj.end_time {
        user_liquidity_snapshot_obj.liquidity = Some(user_liquidity);
    } else {
        let time_remaining_in_round = round_obj
            .end_time
            .checked_sub(env.block.time.seconds())
            .ok_or_else(|| StdError::generic_err("Under-flow sub error 1"))?;

        user_liquidity_snapshot_obj.liquidity = Some(
            if let Ok(liq) = user_liquidity.checked_sub(
                request_withdraw_amount.multiply_ratio(time_remaining_in_round, round_obj.duration),
            ) {
                liq
            } else {
                return Err(StdError::generic_err("Under-flow sub error 2"));
            },
        );
    }

    user_info.amount_delegated = if let Ok(a_d) = user_info
        .amount_delegated
        .checked_sub(request_withdraw_amount)
    {
        a_d
    } else {
        return Err(StdError::generic_err("Under-flow sub error 3"));
    };

    user_info
        .amount_unbonding
        .add_assign(request_withdraw_amount);

    //Snapshot of the current amount delegated
    user_liquidity_snapshot_obj.amount_delegated = Some(user_info.amount_delegated);
    //Storing the User and User liquidity
    user_liquidity_snapshot_stats_helper_store(
        deps.storage,
        round_obj.current_round_index,
        &info.sender,
        user_liquidity_snapshot_obj,
    )?;
    user_info_helper_store(deps.storage, &info.sender, &user_info)?;

    //Querying pending_rewards send back from validator
    //Updating the reward pool
    let total_liquidity: Uint128;
    if pool_state_liquidity_snapshot_obj.total_liquidity.is_none() {
        total_liquidity = pool_state.total_delegated;
    } else {
        total_liquidity = pool_state_liquidity_snapshot_obj.total_liquidity.unwrap();
    }

    if env.block.time.seconds() >= round_obj.end_time {
        pool_state_liquidity_snapshot_obj.total_liquidity = Some(total_liquidity);
    } else {
        let time_remaining_in_round = round_obj
            .end_time
            .checked_sub(env.block.time.seconds())
            .ok_or_else(|| StdError::generic_err("Under-flow sub error 4"))?;
        pool_state_liquidity_snapshot_obj.total_liquidity = Some(
            if let Ok(val) = total_liquidity.checked_sub(
                request_withdraw_amount.multiply_ratio(time_remaining_in_round, round_obj.duration),
            ) {
                val
            } else {
                return Err(StdError::generic_err("Under-flow sub error 5"));
            },
        );
    }

    pool_state.total_delegated = if let Ok(t_d) = pool_state
        .total_delegated
        .checked_sub(request_withdraw_amount)
    {
        t_d
    } else {
        return Err(StdError::generic_err("Under-flow sub error 6"));
    };

    pool_state_liquidity_snapshot_obj.total_delegated = Some(pool_state.total_delegated);
    pool_state_liquidity_helper_store(
        deps.storage,
        round_obj.current_round_index,
        pool_state_liquidity_snapshot_obj,
    )?;

    pool_state_helper_store(deps.storage, &pool_state)?;
    config
        .next_unbonding_batch_amount
        .add_assign(request_withdraw_amount);

    config_helper_store(deps.storage, &config)?;

    Ok(
        Response::new().set_data(to_binary(&HandleAnswer::RequestWithdraw {
            status: Success,
        })?),
    )
}

/// Returns StdResult<Response>
///
/// User withdraw their requested/unbonded funds.
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a mutable reference to the Config
/// * `withdraw_amount` - amount to withdraw
/// * `priority` - u8 representation of highest ContractStatus level this action is permitted
fn try_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: &mut ConfigInfo,
    withdraw_amount: Uint128,
    priority: u8,
) -> StdResult<Response> {
    //Loading Data from storage
    check_status(config.status, priority)?;

    let mut user_info_obj = user_info_helper_read_only(deps.storage, &info.sender)?;
    let pool_state_obj: PoolState = pool_state_helper_read_only(deps.storage)?;

    //Calculating amount available for withdraw
    //USER UNBONDING
    let mut pop_front_counter: Vec<u64> = vec![];

    for i in 0..user_info_obj.unbonding_batches.len() {
        let unbond_batch_index = user_info_obj.unbonding_batches[i];
        let unbonding_batch_obj =
            unbonding_batch_helper_read_only(deps.storage, unbond_batch_index)?;

        if unbonding_batch_obj.unbonding_time.is_some() {
            if env.block.time.seconds() >= unbonding_batch_obj.unbonding_time.unwrap() {
                let unbonding_amount =
                    user_unbond_helper_read_only(deps.storage, unbond_batch_index, &info.sender)?;

                user_info_obj
                    .amount_withdrawable
                    .add_assign(unbonding_amount);

                pop_front_counter.push(unbond_batch_index);
            }
        }
    }

    //only retaining the unclaimed batches
    user_info_obj
        .unbonding_batches
        .retain(|val| !pop_front_counter.contains(val));

    //Checking if amount available is greater than withdraw_amount
    if withdraw_amount > user_info_obj.amount_withdrawable {
        return Err(StdError::generic_err(
            "Trying to withdraw more than available",
        ));
    }

    //Updating user's
    user_info_obj.amount_withdrawable = if let Ok(a_w) = user_info_obj
        .amount_withdrawable
        .checked_sub(withdraw_amount)
    {
        a_w
    } else {
        return Err(StdError::generic_err("Under-flow sub error 1"));
    };

    user_info_obj.amount_unbonding =
        if let Ok(a_u) = user_info_obj.amount_unbonding.checked_sub(withdraw_amount) {
            a_u
        } else {
            return Err(StdError::generic_err("Under-flow sub error 2"));
        };
    user_info_helper_store(deps.storage, &info.sender, &user_info_obj)?;

    //Updating pool state
    pool_state_helper_store(deps.storage, &pool_state_obj)?;

    //Sending funcds to reciepient
    let mut messages = Vec::new();
    if withdraw_amount > Uint128::zero() {
        let withdraw_coins: Vec<Coin> = vec![Coin {
            denom: config.denom.to_string(),
            amount: withdraw_amount,
        }];
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.into_string(),
            amount: withdraw_coins,
        }));
    }

    let res = Response::new()
        .add_messages(messages)
        .set_data(to_binary(&HandleAnswer::Withdraw { status: Success })?);

    Ok(res)
}

/// Returns StdResult<Response>
///
/// User can win prizes here. User will generate a 'n' random sequence of numbers and this sequence will be checked against winning sequence to win prizes.
/// Number of tickets to win will be calculated based on the liquidity provided.
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a mutable reference to the Config
/// * `priority` - u8 representation of highest ContractStatus level this action is permitted
fn try_claim_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: &mut ConfigInfo,
    priority: u8,
) -> StdResult<Response> {
    check_status(config.status, priority)?;

    // Loading dependencies
    let round_obj: RoundInfo = round_helper_read_only(deps.storage)?;
    let mut user_info_obj: UserInfo = user_info_helper_read_only(deps.storage, &info.sender)?;
    //Some variables
    let mut liquidity_current_round: Uint128;
    // let mut legacy_bal: Uint128 = Uint128::zero();
    let mut total_winning_amount: Uint128 = Uint128::zero();
    let mut total_exp: Uint128 = Uint128::zero();
    let mut txn_ticket_count: u128 = 0u128;
    // let mut last_round_claimed: Option<u64> = None;

    //Finding the starting and ending point to start the round
    //Starting round
    let starting_round: u64;
    if user_info_obj.last_claim_rewards_round.is_some() {
        //ERROR CHECK: If user have already claimed the prizes
        if user_info_obj.last_claim_rewards_round.unwrap()
            == (if let Some(val) = round_obj.current_round_index.checked_sub(1) {
                val
            } else {
                return Err(StdError::generic_err("Under-flow sub error 1"));
            })
        {
            return Err(StdError::generic_err(format!(
                "You claimed recently!. Wait for this round to end"
            )));
        } else {
            starting_round = user_info_obj.last_claim_rewards_round.unwrap().add(1);
        }
    } else {
        //1.2)ERROR CHECK: If liquidity provided is less than current round then return error
        check_if_claimable(user_info_obj.starting_round, round_obj.current_round_index)?;
        starting_round = user_info_obj.starting_round.unwrap();
    }

    //Ending round
    let ending_round = round_obj.current_round_index;

    //Main loop
    for round_index in starting_round..ending_round {
        // println!("{}", round_index);
        let mut num_of_tickets: u128;
        let display_tickets: u128;
        let exp: Uint128;
        //Loading dependencies
        let user_liq_obj = user_liquidity_snapshot_stats_helper_read_only(
            deps.storage,
            round_index,
            &info.sender,
        )?;
        let mut reward_stats =
            reward_stats_for_nth_round_helper_read_only(deps.storage, round_index)?;
        let tkt = reward_stats
            .total_rewards
            .checked_sub(reward_stats.total_claimed);
        if tkt.is_ok() {
            if tkt?.is_zero() {
                (_, _, _, _, exp) = finding_user_liquidity(
                    config,
                    user_liq_obj,
                    round_index,
                    &mut user_info_obj,
                    &round_obj,
                    &reward_stats,
                    txn_ticket_count,
                    deps.storage,
                    &info.sender,
                )?;
                user_info_obj.last_claim_rewards_round = Some(round_index);
                if reward_stats.total_exp_claimed.is_some() {
                    reward_stats.total_exp_claimed =
                        Some(reward_stats.total_exp_claimed.unwrap().add(exp));
                } else {
                    reward_stats.total_exp_claimed = Some(exp);
                }

                total_exp.add_assign(exp);

                reward_stats_for_nth_round_helper_store(deps.storage, round_index, &reward_stats);
                continue;
            }
        } else {
            return Err(StdError::generic_err("Under-flow sub error 2"));
        }

        //Checking if the round just expired.
        if reward_stats.rewards_expiration_date.is_some() {
            if env.block.time.seconds() > reward_stats.rewards_expiration_date.unwrap() {
                (_, _, _, _, exp) = finding_user_liquidity(
                    config,
                    user_liq_obj,
                    round_index,
                    &mut user_info_obj,
                    &round_obj,
                    &reward_stats,
                    txn_ticket_count,
                    deps.storage,
                    &info.sender,
                )?;
                user_info_obj.last_claim_rewards_round = Some(round_index);
                total_exp.add_assign(exp);
                if reward_stats.total_exp_claimed.is_some() {
                    reward_stats.total_exp_claimed =
                        Some(reward_stats.total_exp_claimed.unwrap().add(exp));
                } else {
                    reward_stats.total_exp_claimed = Some(exp);
                }
                reward_stats_for_nth_round_helper_store(deps.storage, round_index, &reward_stats);

                continue;
            }
        }

        //Calculating Liquidity/tickets

        (
            liquidity_current_round,
            num_of_tickets,
            user_info_obj,
            txn_ticket_count,
            exp,
        ) = finding_user_liquidity(
            config,
            user_liq_obj,
            round_index,
            &mut user_info_obj,
            &round_obj,
            &reward_stats,
            txn_ticket_count,
            deps.storage,
            &info.sender,
        )?;
        display_tickets = num_of_tickets;

        if num_of_tickets == 0 {
            total_exp.add_assign(exp);
            if reward_stats.total_exp_claimed.is_some() {
                reward_stats.total_exp_claimed =
                    Some(reward_stats.total_exp_claimed.unwrap().add(exp));
            } else {
                reward_stats.total_exp_claimed = Some(exp);
            }
            reward_stats_for_nth_round_helper_store(deps.storage, round_index, &reward_stats);
            continue;
        }

        //Now fetch the winning combination of round
        //*Generate a sequence of random number between the range defined in round_obj
        let mut hasher = Sha256::new();
        hasher.update(&config.prng_seed);
        hasher.update(&round_obj.entropy);
        hasher.update(&info.sender.as_bytes());

        //**generate a random number between 0 and ending_ticket
        let seed: [u8; 32] = hasher.finalize().into();
        let rng = ChaChaRng::from_seed(seed);

        let mut claimed_counter = TierCounter {
            tier_5: Uint128::zero(),
            tier_4: Uint128::zero(),
            tier_3: Uint128::zero(),
            tier_2: Uint128::zero(),
            tier_1: Uint128::zero(),
            tier_0: Uint128::zero(),
        };
        //Iters for Tier 0,1,2,3,4,5
        for tier_number in (0..6).rev() {
            let winning_number: Option<Uint128> = match tier_number {
                0 => Some(reward_stats.winning_sequence.tier_0.winning_number),
                1 => Some(reward_stats.winning_sequence.tier_1.winning_number),
                2 => Some(reward_stats.winning_sequence.tier_2.winning_number),
                3 => Some(reward_stats.winning_sequence.tier_3.winning_number),
                4 => Some(reward_stats.winning_sequence.tier_4.winning_number),
                5 => Some(reward_stats.winning_sequence.tier_5.winning_number),
                _ => None,
            };
            let range_finder = match tier_number {
                0 => reward_stats.winning_sequence.tier_0.range,
                1 => reward_stats.winning_sequence.tier_1.range,
                2 => reward_stats.winning_sequence.tier_2.range,
                3 => reward_stats.winning_sequence.tier_3.range,
                4 => reward_stats.winning_sequence.tier_4.range,
                5 => reward_stats.winning_sequence.tier_5.range,
                _ => Uint128::zero(),
            };
            let mut digit_range = 0u128;
            if range_finder.u128() > 0u128 {
                if let Some(d_r) = range_finder.u128().checked_sub(1) {
                    digit_range = d_r;
                } else {
                    return Err(StdError::generic_err("Under-flow sub error 7"));
                }
            }
            let range = Uniform::new_inclusive(0, digit_range);
            let mut digit_generator = rng.clone().sample_iter(&range);
            // println!("no. of tickets{}  Loops{}", num_of_tickets, loops);

            for _ in 0u128..num_of_tickets {
                //**We need to draft 6 digits individually
                let drafted_number = digit_generator.next().unwrap();

                if drafted_number == winning_number.unwrap().u128() {
                    match tier_number {
                        5 => {
                            claimed_counter.tier_5.add_assign(Uint128::one());
                        }
                        4 => {
                            claimed_counter.tier_4.add_assign(Uint128::one());
                            let val = claimed_counter.tier_5.checked_sub(Uint128::one());
                            if val.is_ok() {
                                claimed_counter.tier_5 = val?
                            } else {
                                return Err(StdError::generic_err("Under-flow sub error 8"));
                            }
                        }
                        3 => {
                            claimed_counter.tier_3.add_assign(Uint128::one());
                            let val = claimed_counter.tier_4.checked_sub(Uint128::one());
                            if val.is_ok() {
                                claimed_counter.tier_4 = val?
                            } else {
                                return Err(StdError::generic_err("Under-flow sub error 9"));
                            }
                        }
                        2 => {
                            claimed_counter.tier_2.add_assign(Uint128::one());
                            let val = claimed_counter.tier_3.checked_sub(Uint128::one());
                            if val.is_ok() {
                                claimed_counter.tier_3 = val?
                            } else {
                                return Err(StdError::generic_err("Under-flow sub error 10"));
                            }
                        }
                        1 => {
                            claimed_counter.tier_1.add_assign(Uint128::one());
                            let val = claimed_counter.tier_2.checked_sub(Uint128::one());
                            if val.is_ok() {
                                claimed_counter.tier_2 = val?
                            } else {
                                return Err(StdError::generic_err("Under-flow sub error 11"));
                            }
                        }
                        0 => {
                            claimed_counter.tier_0.add_assign(Uint128::one());
                            let val = claimed_counter.tier_1.checked_sub(Uint128::one());
                            if val.is_ok() {
                                claimed_counter.tier_1 = val?
                            } else {
                                return Err(StdError::generic_err("Under-flow sub error 12"));
                            }
                        }
                        _ => {}
                    }
                }
            }
            //Making sure that if the previous number is matched only then go for to check for next tier.
            match tier_number {
                5 => num_of_tickets = claimed_counter.tier_5.u128(),
                4 => num_of_tickets = claimed_counter.tier_4.u128(),
                3 => num_of_tickets = claimed_counter.tier_3.u128(),
                2 => num_of_tickets = claimed_counter.tier_2.u128(),
                1 => num_of_tickets = claimed_counter.tier_1.u128(),
                0 => num_of_tickets = claimed_counter.tier_0.u128(),
                _ => {}
            }
        }

        let mut tier_state_obj = reward_stats.distribution_per_tiers;

        if tier_state_obj
            .tier_5
            .claimed
            .num_of_rewards_claimed
            .add(claimed_counter.tier_5)
            .gt(&tier_state_obj.tier_5.num_of_rewards)
        {
            let val = tier_state_obj
                .tier_5
                .num_of_rewards
                .checked_sub(tier_state_obj.tier_5.claimed.num_of_rewards_claimed);
            if val.is_ok() {
                claimed_counter.tier_5 = val?
            } else {
                return Err(StdError::generic_err("Under-flow sub error 13"));
            }
        };

        if tier_state_obj
            .tier_4
            .claimed
            .num_of_rewards_claimed
            .add(claimed_counter.tier_4)
            .gt(&tier_state_obj.tier_4.num_of_rewards)
        {
            let val = tier_state_obj
                .tier_4
                .num_of_rewards
                .checked_sub(tier_state_obj.tier_4.claimed.num_of_rewards_claimed);
            if val.is_ok() {
                claimed_counter.tier_4 = val?
            } else {
                return Err(StdError::generic_err("Under-flow sub error 14"));
            }
        };

        if tier_state_obj
            .tier_3
            .claimed
            .num_of_rewards_claimed
            .add(claimed_counter.tier_3)
            .gt(&tier_state_obj.tier_3.num_of_rewards)
        {
            let val = tier_state_obj
                .tier_3
                .num_of_rewards
                .checked_sub(tier_state_obj.tier_3.claimed.num_of_rewards_claimed);
            if val.is_ok() {
                claimed_counter.tier_3 = val?
            } else {
                return Err(StdError::generic_err("Under-flow sub error 15"));
            }
        };

        if tier_state_obj
            .tier_2
            .claimed
            .num_of_rewards_claimed
            .add(claimed_counter.tier_2)
            .gt(&tier_state_obj.tier_2.num_of_rewards)
        {
            let val = tier_state_obj
                .tier_2
                .num_of_rewards
                .checked_sub(tier_state_obj.tier_2.claimed.num_of_rewards_claimed);
            if val.is_ok() {
                claimed_counter.tier_2 = val?
            } else {
                return Err(StdError::generic_err("Under-flow sub error 16"));
            }
        };

        if tier_state_obj
            .tier_1
            .claimed
            .num_of_rewards_claimed
            .add(claimed_counter.tier_1)
            .gt(&tier_state_obj.tier_1.num_of_rewards)
        {
            let val = tier_state_obj
                .tier_1
                .num_of_rewards
                .checked_sub(tier_state_obj.tier_1.claimed.num_of_rewards_claimed);
            if val.is_ok() {
                claimed_counter.tier_1 = val?
            } else {
                return Err(StdError::generic_err("Under-flow sub error 17"));
            }
        };

        if tier_state_obj
            .tier_0
            .claimed
            .num_of_rewards_claimed
            .add(claimed_counter.tier_0)
            .gt(&tier_state_obj.tier_0.num_of_rewards)
        {
            let val = tier_state_obj
                .tier_0
                .num_of_rewards
                .checked_sub(tier_state_obj.tier_0.claimed.num_of_rewards_claimed);
            if val.is_ok() {
                claimed_counter.tier_0 = val?
            } else {
                return Err(StdError::generic_err("Under-flow sub error 18"));
            }
        };

        tier_state_obj
            .tier_5
            .claimed
            .num_of_rewards_claimed
            .add_assign(claimed_counter.tier_5);

        tier_state_obj
            .tier_4
            .claimed
            .num_of_rewards_claimed
            .add_assign(claimed_counter.tier_4);

        tier_state_obj
            .tier_3
            .claimed
            .num_of_rewards_claimed
            .add_assign(claimed_counter.tier_3);

        tier_state_obj
            .tier_2
            .claimed
            .num_of_rewards_claimed
            .add_assign(claimed_counter.tier_2);

        tier_state_obj
            .tier_1
            .claimed
            .num_of_rewards_claimed
            .add_assign(claimed_counter.tier_1);

        tier_state_obj
            .tier_0
            .claimed
            .num_of_rewards_claimed
            .add_assign(claimed_counter.tier_0);

        //Calculate Total rewards for each tier

        let user_rewards_per_tier_log = TierLog {
            tier_5: RewardsPerTierInfo {
                num_of_rewards_claimed: (claimed_counter
                    .tier_5
                    .multiply_ratio(tier_state_obj.tier_5.claimed.reward_per_match.u128(), 1u128)),
                reward_per_match: tier_state_obj.tier_5.claimed.reward_per_match,
            },
            tier_4: RewardsPerTierInfo {
                num_of_rewards_claimed: (claimed_counter
                    .tier_4
                    .multiply_ratio(tier_state_obj.tier_4.claimed.reward_per_match.u128(), 1u128)),
                reward_per_match: tier_state_obj.tier_4.claimed.reward_per_match,
            },
            tier_3: RewardsPerTierInfo {
                num_of_rewards_claimed: (claimed_counter
                    .tier_3
                    .multiply_ratio(tier_state_obj.tier_3.claimed.reward_per_match.u128(), 1u128)),
                reward_per_match: tier_state_obj.tier_3.claimed.reward_per_match,
            },
            tier_2: RewardsPerTierInfo {
                num_of_rewards_claimed: (claimed_counter
                    .tier_2
                    .multiply_ratio(tier_state_obj.tier_2.claimed.reward_per_match.u128(), 1u128)),
                reward_per_match: tier_state_obj.tier_2.claimed.reward_per_match,
            },
            tier_1: RewardsPerTierInfo {
                num_of_rewards_claimed: (claimed_counter
                    .tier_1
                    .multiply_ratio(tier_state_obj.tier_1.claimed.reward_per_match.u128(), 1u128)),
                reward_per_match: tier_state_obj.tier_1.claimed.reward_per_match,
            },
            tier_0: RewardsPerTierInfo {
                num_of_rewards_claimed: (claimed_counter
                    .tier_0
                    .multiply_ratio(tier_state_obj.tier_0.claimed.reward_per_match.u128(), 1u128)),
                reward_per_match: tier_state_obj.tier_0.claimed.reward_per_match,
            },
        };

        let amount_won = user_rewards_per_tier_log
            .tier_5
            .num_of_rewards_claimed
            .add(user_rewards_per_tier_log.tier_4.num_of_rewards_claimed)
            .add(user_rewards_per_tier_log.tier_3.num_of_rewards_claimed)
            .add(user_rewards_per_tier_log.tier_2.num_of_rewards_claimed)
            .add(user_rewards_per_tier_log.tier_1.num_of_rewards_claimed)
            .add(user_rewards_per_tier_log.tier_0.num_of_rewards_claimed);

        if amount_won > Uint128::zero() {
            let user_rewards_log: UserRewardsLog = UserRewardsLog {
                round: (round_index),

                liquidity: Some(Uint128::from(liquidity_current_round)),
                rewards_per_tier: Some(user_rewards_per_tier_log),
                total_amount_won: Some(amount_won),
                tickets: Uint128::from(display_tickets),
                ticket_price: reward_stats.ticket_price,
                total_exp_gained: Some(exp),
            };
            reward_stats.total_claimed.add_assign(amount_won);
            if reward_stats.total_exp_claimed.is_some() {
                reward_stats.total_exp_claimed =
                    Some(reward_stats.total_exp_claimed.unwrap().add(exp));
            } else {
                reward_stats.total_exp_claimed = Some(exp);
            }
            reward_stats.distribution_per_tiers = tier_state_obj;

            total_winning_amount
                .add_assign(user_rewards_log.total_amount_won.unwrap_or(Uint128::zero()));
            total_exp.add_assign(exp);

            user_rewards_log_helper_store(deps.storage, &info.sender, &user_rewards_log)?;
            reward_stats_for_nth_round_helper_store(deps.storage, round_index, &reward_stats);
        }

        if txn_ticket_count.eq(&round_obj.number_of_tickers_per_transaction.u128()) {
            break;
        }
    }

    // if last_round_claimed.is_some() {
    //     user_info_obj.last_claim_rewards_round = last_round_claimed;
    if total_winning_amount > Uint128::zero() {
        user_info_obj.total_won.add_assign(total_winning_amount);
    }
    user_info_helper_store(deps.storage, &info.sender, &user_info_obj)?;
    // }

    let mut messages: Vec<CosmosMsg> = vec![];
    if total_winning_amount.u128() > 0u128 {
        let winning_coins: Vec<Coin> = vec![Coin {
            denom: config.denom.as_mut().into(),
            amount: total_winning_amount,
        }];

        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: winning_coins,
        }));
    }

    //TODO: check this
    // //Updating last claimed and claiming xp
    // let add_exp = experience_contract::msg::ExecuteMsg::AddExp {
    //     address: info.sender.clone().to_string(),
    //     exp: total_exp,
    // };

    // if config.exp_contract.is_some() {
    //     messages.push(add_exp.to_cosmos_msg(
    //         BLOCK_SIZE,
    //         config.exp_contract.clone().unwrap().contract.hash,
    //         config.exp_contract.clone().unwrap().contract.address,
    //         None,
    //     )?);
    // }

    let res =
        Response::new()
            .add_messages(messages)
            .set_data(to_binary(&HandleAnswer::ClaimRewards {
                status: Success,
                winning_amount: total_winning_amount,
            })?);

    Ok(res)
}

fn finding_user_liquidity(
    config: &ConfigInfo,
    mut user_liq_obj: UserLiqState,
    round_index: u64,
    user_info_obj: &mut UserInfo,
    round_obj: &RoundInfo,
    reward_stats: &RewardsState,
    mut txn_ticket_count: u128,
    storage: &mut dyn Storage,
    sender: &Addr,
) -> StdResult<(Uint128, u128, UserInfo, u128, Uint128)> {
    let liquidity;
    let mut legacy_bal: Uint128 = Uint128::zero();
    if user_liq_obj.liquidity.is_some() {
        liquidity = user_liq_obj.liquidity.unwrap();
    } else {
        let mut finding_liq_round: u64 = if let Some(rn) = round_index.checked_sub(1) {
            rn
        } else {
            return Err(StdError::generic_err("Under-flow sub error 3"));
        };
        let start = if user_info_obj.last_claim_rewards_round.is_some() {
            user_info_obj.last_claim_rewards_round.unwrap()
        } else {
            user_info_obj.starting_round.unwrap()
        };
        while finding_liq_round >= start {
            // println!("Finding liquidity {}", finding_liq_round);
            let user_liq_obj_prev_round =
                user_liquidity_snapshot_stats_helper_read_only(storage, finding_liq_round, sender)?;
            if user_liq_obj_prev_round.amount_delegated.is_some() {
                legacy_bal = user_liq_obj_prev_round.amount_delegated.unwrap();
                break;
            } else {
                finding_liq_round = if let Some(f_liq_round) = finding_liq_round.checked_sub(1) {
                    f_liq_round
                } else {
                    return Err(StdError::generic_err("Under-flow sub error 4"));
                };
            }
        }

        user_liq_obj.liquidity = Some(legacy_bal);
        user_liq_obj.amount_delegated = Some(legacy_bal);
        // user_liquidity_snapshot_stats_helper_store(storage, round_index, sender, user_liq_obj)?;

        liquidity = legacy_bal;
    }
    let pool_state_liquidity_snapshot_obj: PoolLiqState =
        pool_state_liquidity_helper_read_only(storage, round_index)?;

    //TODO config check here
    let mut exp = Uint128::zero();
    if config.exp_contract.is_some() {
        exp = reward_stats.total_exp.unwrap().multiply_ratio(
            liquidity,
            pool_state_liquidity_snapshot_obj.total_liquidity.unwrap(),
        );
    }

    //Calculating total tickets
    let mut num_of_tickets: u128 = liquidity
        .multiply_ratio(1u128, reward_stats.ticket_price.u128())
        .u128();

    if user_liq_obj.tickets_used.is_some() {
        if let Some(tickets) = num_of_tickets.checked_sub(user_liq_obj.tickets_used.unwrap().u128())
        {
            num_of_tickets = tickets;
        } else {
            return Err(StdError::generic_err("Under-flow sub error 5"));
        }
    }

    if txn_ticket_count.add(num_of_tickets) > round_obj.number_of_tickers_per_transaction.u128() {
        if let Some(tickets) = round_obj
            .number_of_tickers_per_transaction
            .u128()
            .checked_sub(txn_ticket_count)
        {
            num_of_tickets = tickets;
            txn_ticket_count.add_assign(tickets);
        } else {
            return Err(StdError::generic_err("Under-flow sub error 6"));
        };
    } else {
        user_info_obj.last_claim_rewards_round = Some(round_index);
        txn_ticket_count.add_assign(num_of_tickets);
    }

    if user_liq_obj.tickets_used.is_some() {
        user_liq_obj.tickets_used =
            Some(Uint128::from(num_of_tickets) + user_liq_obj.tickets_used.unwrap());
    } else {
        user_liq_obj.tickets_used = Some(Uint128::from(num_of_tickets));
    }

    user_liquidity_snapshot_stats_helper_store(storage, round_index, sender, user_liq_obj)?;

    Ok((
        liquidity,
        num_of_tickets,
        user_info_obj.clone(),
        txn_ticket_count,
        exp,
    ))
}

/// Returns StdResult<Response>
///
/// creates a viewing key
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a reference to the Config
/// * `entropy` - string slice of the input String to be used as entropy in randomization
/// * `priority` - u8 representation of highest ContractStatus level this action is permitted
fn try_create_viewing_key(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: &mut ConfigInfo,
    entropy: String,
    priority: u8,
) -> StdResult<Response> {
    check_status(config.status, priority)?;

    let prng_seed = &config.prng_seed;
    let key = ViewingKey::new(&env, info.clone(), &prng_seed, (&entropy).as_ref());
    write_viewing_key_helper(deps.storage, &info.sender, &key)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::CreateViewingKey { key })?))
}

/// Returns StdResult<Response>
///
/// sets the viewing key
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a reference to the Config
/// * `key` - String to be used as the viewing key
/// * `priority` - u8 representation of highest ContractStatus level this action is permitted
fn try_set_viewing_key(
    deps: DepsMut,
    info: MessageInfo,
    config: &mut ConfigInfo,
    key: String,
    priority: u8,
) -> StdResult<Response> {
    check_status(config.status, priority)?;

    let vk = ViewingKey(key);
    write_viewing_key_helper(deps.storage, &info.sender, &vk)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetViewingKey { status: Success })?))
}

/// Returns StdResult<Response>
///
/// revoke the ability to use a specified permit
///
/// # Arguments
///
/// * `storage` - mutable reference to the contract's storage
/// * `sender` - a reference to the message sender
/// * `permit_name` - string slice of the name of the permit to revoke
/// * `priority` - u8 representing the highest status level this action may execute at
/// * `config` - a reference to the Config
fn try_revoke_permit(
    storage: &mut dyn Storage,
    sender: &str,
    permit_name: &str,
    priority: u8,
    config: &ConfigInfo,
) -> StdResult<Response> {
    check_status(config.status, priority)?;

    RevokedPermits::revoke_permit(storage, PREFIX_REVOKED_PERMITS, sender, permit_name);

    Ok(Response::new().set_data(to_binary(&HandleAnswer::RevokePermit { status: Success })?))
}

///////////////////////////////////////// Sponsor Functions //////////////////////////////////////

/// Returns StdResult<Response>
///
/// Sponsors can delegate to the contract.
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `title` - The title of the sponsor
/// * `message` - The message by the sponsor
/// * `config` - a mutable reference to the Config
/// * `priority` - u8 representation of highest ContractStatus level this action is permitted
fn try_sponsor(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: Option<String>,
    message: Option<String>,
    config: &mut ConfigInfo,
    priority: u8,
) -> StdResult<Response> {
    check_status(config.status, priority)?;

    //CHECKING: If sponsor amount is valid
    let deposit_amount: Uint128 = check_if_valid_amount(&info, &config)?;

    //Loading sponsor info
    let mut sponsor_info_obj = sponsor_info_helper_read_only(deps.storage, &info.sender)?;
    //Updating sponsor info
    sponsor_info_obj.amount_sponsored.add_assign(deposit_amount);
    //addr_list_index is assigned when sponsor deposits.
    if sponsor_info_obj.addr_list_index.is_none() {
        //find index and give sponsor this index to recognize the order in the list.
        let mut sponsor_stats_obj = sponsor_stats_helper_read_only(deps.storage)?;
        let index: Option<u32>;
        if sponsor_stats_obj.empty_slots.len() > 0 {
            index = Some(sponsor_stats_obj.empty_slots.pop().unwrap());
        } else {
            index = Some(sponsor_stats_obj.offset);
            sponsor_stats_obj.offset.add_assign(1u32);
        }
        sponsor_info_obj.addr_list_index = index;
        sponsor_addr_list_helper_store(deps.storage, index.unwrap(), &info.sender)?;
        sponsor_stats_helper_store(deps.storage, &sponsor_stats_obj)?;

        // You can only add title and message through sponsor function when sponsoring for the first time
        if sponsor_info_obj.has_requested == false {
            if title.is_some() || message.is_some() {
                sponsor_info_obj.has_requested = true;
                sponsor_display_request_deque_push_back_helper(
                    deps.storage,
                    &GlobalSponsorDisplayRequestListState {
                        addr: info.sender.to_string(),
                        index: sponsor_info_obj.addr_list_index,
                        title,
                        message,
                        deque_store_index: None,
                    },
                )?;
            }
        }
    };

    sponsor_info_helper_store(deps.storage, &info.sender, &sponsor_info_obj);

    //3)Fetching Pool Store data and updating it
    let mut pool_state: PoolState = pool_state_helper_read_only(deps.storage)?;
    pool_state.total_sponsored.add_assign(deposit_amount);

    //4) Choosing Validator for deposit and updating the validator information
    let selected_val_index = config.next_validator_for_delegation as usize;
    config.validators[selected_val_index]
        .delegated
        .add_assign(deposit_amount);

    if selected_val_index == config.validators.len() - 1 {
        config.next_validator_for_delegation = 0;
    } else {
        config.next_validator_for_delegation += 1;
    }

    //4.3) Storing validator
    config_helper_store(deps.storage, &config)?;

    //5) Querying pending_rewards send back from validator
    let rewards = get_rewards(deps.as_ref(), &env.contract.address, &config).unwrap();
    let mut rewards_amount = Uint128::zero();
    for reward in rewards {
        if reward.validator_address.as_str()
            == config.validators[selected_val_index].address.as_str()
        {
            rewards_amount = rewards_amount.add(reward.reward);
        }
    }

    //Updating PoolState
    pool_state
        .rewards_returned_to_contract
        .add_assign(rewards_amount);
    pool_state_helper_store(deps.storage, &pool_state)?;

    //Sending staking message
    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(stake(
        &config.validators[selected_val_index].address,
        deposit_amount,
        &config.denom,
    ));

    Ok(Response::new()
        .add_messages(messages)
        .set_data(to_binary(&HandleAnswer::Sponsor { status: Success })?))
}

/// Returns StdResult<Response>
///
/// Sponsors can request to withdraw their funds. It take 21 days to withdraw the funds.
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a mutable reference to the Config
/// * `request_withdraw_amount` - amount requested to withdraw
/// * `priority` - u8 representation of highest ContractStatus level this action is permitted
fn try_sponsor_request_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    config: &mut ConfigInfo,
    request_withdraw_amount: Uint128,
    priority: u8,
) -> StdResult<Response> {
    //Checking status
    check_status(config.status, priority)?;
    let mut sponsor_info_obj = sponsor_info_helper_read_only(deps.storage, &info.sender)?;

    let mut pool_state: PoolState = pool_state_helper_read_only(deps.storage)?;
    // Checking: If the amount unbonded is not greater than amount delegated
    if sponsor_info_obj.amount_sponsored < request_withdraw_amount {
        return Err(StdError::generic_err(format!(
            "insufficient funds to redeem: balance={}, required={}",
            sponsor_info_obj.amount_sponsored, request_withdraw_amount
        )));
    }
    // Checking: If the amount unbonded is not equal to zero
    if request_withdraw_amount == Uint128::zero() {
        return Err(StdError::generic_err(format!(
            "Cannot withdraw 0 {}",
            config.denom
        )));
    }
    //If the liquidity struct for that round is not generated then we create one and store
    pool_state.total_sponsored = if let Ok(t_s) = pool_state
        .total_sponsored
        .checked_sub(request_withdraw_amount)
    {
        t_s
    } else {
        return Err(StdError::generic_err("Under-flow sub error 1"));
    };

    sponsor_info_obj.amount_sponsored = if let Ok(a_s) = sponsor_info_obj
        .amount_sponsored
        .checked_sub(request_withdraw_amount)
    {
        a_s
    } else {
        return Err(StdError::generic_err("Under-flow sub error 2"));
    };

    sponsor_info_obj
        .amount_unbonding
        .add_assign(request_withdraw_amount);

    // if amount delegated == 0 then remove the name from sponsors list
    // Also remove to sponsors address list.
    if sponsor_info_obj.amount_sponsored.u128() == 0 {
        //Removing sponsor from global sponsors list
        sponsor_addr_list_remove_helper_store(
            deps.storage,
            sponsor_info_obj.addr_list_index.unwrap(),
        )?;

        //Updating sponsor stat store
        let mut sponsor_stats_obj = sponsor_stats_helper_read_only(deps.storage)?;
        if sponsor_info_obj.addr_list_index.unwrap() == sponsor_stats_obj.offset - 1 {
            // sponsor_stats_obj.offset.sub_assign(1);

            sponsor_stats_obj.offset = if let Some(off) = sponsor_stats_obj.offset.checked_sub(1) {
                off
            } else {
                return Err(StdError::generic_err("Under-flow sub error 3"));
            }
        } else {
            sponsor_stats_obj
                .empty_slots
                .push(sponsor_info_obj.addr_list_index.unwrap());
        }
        sponsor_stats_helper_store(deps.storage, &sponsor_stats_obj)?;

        //Removing any pending message requests made by the user
        let len = SPONSOR_DISPLAY_REQ_STORE.get_len(deps.storage)?;
        if len > 0 {
            for i in 0..(len - 1) {
                let req_obj = SPONSOR_DISPLAY_REQ_STORE.get_at(deps.storage, i)?;

                if req_obj.index == sponsor_info_obj.addr_list_index {
                    sponsor_display_request_deque_helper_remove(deps.storage, i)?;
                }
            }
        }

        //Updating sponsor info
        sponsor_info_obj.has_requested = false;
        sponsor_info_obj.addr_list_index = None;
    }

    //Checking if sponsor has already made unbonding request this current round.
    if !sponsor_info_obj
        .unbonding_batches
        .contains(&config.next_unbonding_batch_index)
    {
        sponsor_info_obj
            .unbonding_batches
            .push(config.next_unbonding_batch_index);
    }

    //Adding to amount unbonding for next unbonding round
    let mut unbonding_amount = sponsor_unbond_helper_read_only(
        deps.storage,
        config.next_unbonding_batch_index,
        &info.sender,
    )?;
    unbonding_amount.add_assign(request_withdraw_amount);
    sponsor_unbond_helper_store(
        deps.storage,
        config.next_unbonding_batch_index,
        &info.sender,
        unbonding_amount,
    )?;

    sponsor_info_helper_store(deps.storage, &info.sender, &sponsor_info_obj);

    //Asking the validator to undelegate the funds
    pool_state_helper_store(deps.storage, &pool_state)?;
    config
        .next_unbonding_batch_amount
        .add_assign(request_withdraw_amount);

    config_helper_store(deps.storage, &config)?;

    let res = Response::new().set_data(to_binary(&HandleAnswer::RequestWithdrawSponsor {
        status: Success,
    })?);

    Ok(res)
}

/// Returns StdResult<Response>
///
/// Sponsors withdraw their requested funds. It take 21 days to withdraw the funds after the request is made.
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a mutable reference to the Config
/// * `withdraw_amount` - amount to withdraw
/// * `priority` - u8 representation of highest ContractStatus level this action is permitted
fn try_sponsor_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: &mut ConfigInfo,
    withdraw_amount: Uint128,
    priority: u8,
) -> StdResult<Response> {
    //loading Data from storage
    check_status(config.status, priority)?;

    let mut sponsor_info_obj = sponsor_info_helper_read_only(deps.storage, &info.sender)?;

    //Checking amount available for withdraw
    let mut amount_av_for_withdraw = Uint128::zero();
    //STORING USER UNBONDING
    let mut pop_front_counter: Vec<u64> = vec![];

    for i in 0..sponsor_info_obj.unbonding_batches.len() {
        let unbond_batch_index = sponsor_info_obj.unbonding_batches[i];
        let unbonding_batch_obj =
            unbonding_batch_helper_read_only(deps.storage, unbond_batch_index)?;

        if unbonding_batch_obj.unbonding_time.is_some() {
            if env.block.time.seconds() >= unbonding_batch_obj.unbonding_time.unwrap() {
                let unbonding_amount = sponsor_unbond_helper_read_only(
                    deps.storage,
                    unbond_batch_index,
                    &info.sender,
                )?;

                amount_av_for_withdraw.add_assign(unbonding_amount);

                pop_front_counter.push(unbond_batch_index);
            }
        }
    }

    sponsor_info_obj
        .unbonding_batches
        .retain(|val| !pop_front_counter.contains(val));

    sponsor_info_obj
        .amount_withdrawable
        .add_assign(amount_av_for_withdraw);

    //ERROR Check
    if withdraw_amount
        > sponsor_info_obj
            .amount_withdrawable
            .add(amount_av_for_withdraw)
    {
        return Err(StdError::generic_err(
            "Trying to withdraw more than available",
        ));
    }

    //Updating user and   pool state

    sponsor_info_obj
        .amount_withdrawable
        .add_assign(amount_av_for_withdraw);
    // sponsor_info_obj
    //     .amount_unbonding
    //     .sub_assign(withdraw_amount);

    sponsor_info_obj.amount_unbonding = if let Ok(a_u) = sponsor_info_obj
        .amount_unbonding
        .checked_sub(withdraw_amount)
    {
        a_u
    } else {
        return Err(StdError::generic_err("Under-flow sub error"));
    };

    // sponsor_info_obj
    //     .amount_withdrawable
    //     .sub_assign(withdraw_amount);
    sponsor_info_obj.amount_withdrawable = if let Ok(a_w) = sponsor_info_obj
        .amount_withdrawable
        .checked_sub(withdraw_amount)
    {
        a_w
    } else {
        return Err(StdError::generic_err("Under-flow sub error"));
    };
    sponsor_info_helper_store(deps.storage, &info.sender, &sponsor_info_obj);

    let mut messages: Vec<CosmosMsg> = vec![];

    if withdraw_amount > Uint128::zero() {
        let withdraw_coins: Vec<Coin> = vec![Coin {
            denom: config.denom.to_string(),
            amount: withdraw_amount,
        }];
        //Sending a message to withdraw
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: withdraw_coins,
        }));
    }

    let res = Response::new().add_messages(messages).set_data(to_binary(
        &HandleAnswer::SponsorWithdraw { status: Success },
    )?);

    Ok(res)
}

/// Returns StdResult<Response>
///
/// Sponsors can edit the message they send before.
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `title` - The title of the sponsor
/// * `message` - The message by the sponsor
/// * `config` - a mutable reference to the Config
/// * `priority` - u8 representation of highest ContractStatus level this action is permitted
fn try_sponsor_message_edit(
    deps: DepsMut,
    info: MessageInfo,
    title: Option<String>,
    message: Option<String>,
    delete_title: bool,
    delete_message: bool,
    config: &mut ConfigInfo,
    priority: u8,
) -> StdResult<Response> {
    check_status(config.status, priority)?;

    let mut sponsor_info_obj = sponsor_info_helper_read_only(deps.storage, &info.sender)?;
    if sponsor_info_obj.amount_sponsored.u128() == 0 {
        return Err(StdError::generic_err("Sponsor to avail this option"));
    }

    if delete_title || delete_message {
        if delete_title {
            sponsor_info_obj.title = None;
        }
        if delete_message {
            sponsor_info_obj.message = None;
        }
    }

    if config.sponsor_msg_edit_fee.is_some() {
        let sponsor_fee_amount: Uint128 = check_if_valid_amount(&info, &config)?;

        if sponsor_fee_amount < config.sponsor_msg_edit_fee.unwrap() {
            return Err(StdError::generic_err(
                "Please pay fee to allow title/message edit",
            ));
        }
    }

    // Add name/message request
    if sponsor_info_obj.has_requested == false {
        if title.is_some() || message.is_some() {
            sponsor_info_obj.has_requested = true;
            sponsor_display_request_deque_push_back_helper(
                deps.storage,
                &GlobalSponsorDisplayRequestListState {
                    addr: info.sender.to_string(),
                    index: sponsor_info_obj.addr_list_index,
                    title,
                    message,
                    deque_store_index: None,
                },
            )?;
        }
    } else {
        let len = SPONSOR_DISPLAY_REQ_STORE.get_len(deps.storage)?;

        for i in 0..(len) {
            let req_obj = SPONSOR_DISPLAY_REQ_STORE.get_at(deps.storage, i)?;

            if req_obj.index == sponsor_info_obj.addr_list_index {
                // sponsor_display_request_deque_helper_remove(deps.storage, i)?;
                SPONSOR_DISPLAY_REQ_STORE.set_at(
                    deps.storage,
                    i,
                    &GlobalSponsorDisplayRequestListState {
                        addr: info.sender.to_string(),
                        index: sponsor_info_obj.addr_list_index,
                        title,
                        message,
                        deque_store_index: None,
                    },
                )?;
                break;
            }
        }
        sponsor_info_obj.has_requested = true;
    }

    sponsor_info_helper_store(deps.storage, &info.sender, &sponsor_info_obj);

    Ok(
        Response::new().set_data(to_binary(&HandleAnswer::SponsorMessageEdit {
            status: Success,
        })?),
    )
}

///////////////////////////////////////// Admin Functions //////////////////////////////////////

fn try_review_sponsor_messages(
    deps: DepsMut,
    info: MessageInfo,
    mut decisions: Vec<Review>,
    config: &ConfigInfo,
) -> StdResult<Response> {
    check_if_reviewer(&config, &info.sender)?;

    decisions.sort_by(|a, b| a.index.cmp(&b.index));

    // Get reqs and
    // let len = SPONSOR_DISPLAY_REQ_STORE.get_len(deps.storage)?;
    let mut off = 0;
    for decision in decisions {
        let sponsor_disp_obj =
            SPONSOR_DISPLAY_REQ_STORE.get_at(deps.storage, decision.index - off)?;
        let mut sponsor_info_obj = sponsor_info_helper_read_only(
            deps.storage,
            &deps.api.addr_validate(sponsor_disp_obj.addr.as_str())?,
        )?;

        if decision.is_accpeted {
            if sponsor_disp_obj.message.is_some() {
                sponsor_info_obj.message = sponsor_disp_obj.message;
            }
            if sponsor_disp_obj.title.is_some() {
                sponsor_info_obj.title = sponsor_disp_obj.title;
            }
        }
        sponsor_info_obj.has_requested = false;
        sponsor_info_helper_store(
            deps.storage,
            &deps.api.addr_validate(sponsor_disp_obj.addr.as_str())?,
            &sponsor_info_obj,
        );

        sponsor_display_request_deque_helper_remove(deps.storage, decision.index - off)?;
        off += 1;
    }

    Ok(
        Response::new().set_data(to_binary(&HandleAnswer::ReviewSponsorMessages {
            status: Success,
        })?),
    )
}

fn try_remove_sponsor_credentials(
    deps: DepsMut,
    info: MessageInfo,
    decisions: Vec<RemoveSponsorCredentialsDecisions>,
    config: &ConfigInfo,
) -> StdResult<Response> {
    check_if_reviewer(&config, &info.sender)?;

    for decision in decisions {
        let sponsor_address = sponsor_addr_list_helper_read_only(deps.storage, decision.index)?;

        let mut sponsor_info_obj = sponsor_info_helper_read_only(deps.storage, &sponsor_address)?;

        if decision.remove_sponsor_title {
            sponsor_info_obj.title = None;
        }
        if decision.remove_sponsor_message {
            sponsor_info_obj.message = None;
        }

        sponsor_info_helper_store(deps.storage, &sponsor_address, &sponsor_info_obj)
    }

    Ok(
        Response::new().set_data(to_binary(&HandleAnswer::RemoveSponsorCredentials {
            status: Success,
        })?),
    )
}

/// Returns StdResult<Response>
///
/// End of the round - winning sequence and difficulty is calculated.
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a mutable reference to the Config
/// * `priority` - u8 representation of highest ContractStatus level this action is permitted
fn try_end_round(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: &mut ConfigInfo,
    priority: u8,
) -> StdResult<Response> {
    //Checking status
    check_status(config.status, priority)?;

    //Checking if this function is called by Triggerer
    check_if_triggerer(&config, &info.sender)?;

    //Loading dependencies and data
    let mut round_obj: RoundInfo = round_helper_read_only(deps.storage)?;

    //Checking if the round can be closed or there still some time left
    validate_end_time(round_obj.end_time, env.block.time.seconds())?;

    //Validate_start_time(round_obj.start_time, env.block.time)?;
    //Extending the entropy using block height and time.
    round_obj.entropy.extend(&env.block.height.to_be_bytes());
    round_obj
        .entropy
        .extend(&env.block.time.seconds().to_be_bytes());
    round_obj.entropy.extend(&config.prng_seed);

    //Querying pending_rewards and add them to winning prizes.
    let rewards_obj = get_rewards(deps.as_ref(), &env.contract.address, &config)?;
    let mut total_rewards: Uint128 = Uint128::zero();
    for reward_obj in rewards_obj.clone() {
        total_rewards.add_assign(reward_obj.reward);
    }

    //Calculating total rewards recieved
    let mut pool_state: PoolState = pool_state_helper_read_only(deps.storage)?;
    let mut total_rewards = pool_state.rewards_returned_to_contract.add(total_rewards);
    let mut unclaimed_exp = None;
    let mut messages = Vec::new();

    //Checking when rewards are zero
    if total_rewards == Uint128::zero() {
        if config.exp_contract.is_some() {
            // unclaimed_exp = Some(get_exp(deps.as_ref(), config)?);
            // TODO: check this
            // messages.push(
            //     experience_contract::msg::ExecuteMsg::UpdateLastClaimed {}.to_cosmos_msg(
            //         BLOCK_SIZE,
            //         config.exp_contract.clone().unwrap().contract.hash,
            //         config.exp_contract.clone().unwrap().contract.address,
            //         None,
            //     )?,
            // );
        }
        let reward_stats_for_nth_round: RewardsState = RewardsState {
            //Using a helper function to create rewards distribution
            distribution_per_tiers: Default::default(),
            ticket_price: round_obj.ticket_price,
            winning_sequence: Default::default(),
            rewards_expiration_date: Some(
                env.block
                    .time
                    .seconds()
                    .add(round_obj.rewards_expiry_duration),
            ),
            total_rewards: Uint128::zero(),
            total_claimed: Uint128::zero(),
            total_exp: unclaimed_exp,
            total_exp_claimed: Some(Uint128::zero()),
        };
        //*Saving the rewards_tickets of current round
        reward_stats_for_nth_round_helper_store(
            deps.storage,
            round_obj.current_round_index,
            &reward_stats_for_nth_round,
        );
        round_obj.start_time = env.block.time.seconds();
        round_obj.end_time = env.block.time.seconds().add(round_obj.duration);
        round_obj.current_round_index.add_assign(1u64);
        round_helper_store(deps.storage, &round_obj)?;

        return Ok(Response::new()
            .set_data(to_binary(&HandleAnswer::EndRound { status: Success })?)
            .add_messages(messages));
    }

    //Trigger get share from rewards
    let trigger_share =
        total_rewards.multiply_ratio(round_obj.triggerer_share_percentage, config.common_divisor);

    total_rewards = if let Ok(t_r) = total_rewards.checked_sub(trigger_share) {
        t_r
    } else {
        return Err(StdError::generic_err("Under-flow sub error 1"));
    };

    //Shade and PoolParty Share of the rewards
    let admin_share = total_rewards.multiply_ratio(
        round_obj.admin_share.total_percentage_share,
        config.common_divisor,
    );
    let shade_share = admin_share.multiply_ratio(
        round_obj.admin_share.shade_percentage_share,
        config.common_divisor,
    );

    let galactic_pools_share = if let Ok(g_p_s) = admin_share.checked_sub(shade_share) {
        g_p_s
    } else {
        return Err(StdError::generic_err("Under-flow sub error 2"));
    };

    let mut winning_amount = if let Ok(w_a) = total_rewards.checked_sub(admin_share) {
        w_a
    } else {
        return Err(StdError::generic_err("Under-flow sub error 3"));
    };

    //Claim the unclaimed rewards that have been expired
    let when_last_redeemed_the_unclaimed_obj = round_obj
        .unclaimed_rewards_last_claimed_round
        .unwrap_or(0)
        .add(1);
    let mut reserve: Uint128 = Uint128::zero();
    let mut propagate: Uint128 = Uint128::zero();

    for round in when_last_redeemed_the_unclaimed_obj..round_obj.current_round_index {
        let rewards_expiry_check_obj =
            reward_stats_for_nth_round_helper_read_only(deps.storage, round)?;

        if rewards_expiry_check_obj.rewards_expiration_date.is_some() {
            if rewards_expiry_check_obj.rewards_expiration_date.unwrap() <= env.block.time.seconds()
            {
                round_obj.unclaimed_rewards_last_claimed_round = Some(round);
                //Fetch unclaimed
                let total_unclaimed = if let Ok(t_c) = rewards_expiry_check_obj
                    .total_rewards
                    .checked_sub(rewards_expiry_check_obj.total_claimed)
                {
                    t_c
                } else {
                    return Err(StdError::generic_err("Under-flow sub error 4"));
                };

                //Reserve vs Propagation
                let reserve_share = total_unclaimed.multiply_ratio(
                    round_obj.unclaimed_distribution.reserves_percentage as u128,
                    config.common_divisor as u128,
                );

                reserve.add_assign(reserve_share);
                let remaining = total_unclaimed.checked_sub(reserve_share);
                if remaining.is_ok() {
                    propagate = propagate.add(remaining?);
                } else {
                    return Err(StdError::generic_err("Under-flow sub error 5"));
                }
            } else {
                break;
            }
        }
    }

    // Fetching validator with the lowest percentage filled
    // Choosing Validator for deposit and updating the validator information
    let index = config.next_validator_for_delegation as usize;

    if reserve.u128() > 0 {
        config.validators[index].delegated.add_assign(reserve);
        pool_state.total_reserves.add_assign(reserve);
        messages.push(stake(
            &config.validators[index].address,
            reserve,
            &config.denom,
        ));

        if index == config.validators.len() - 1 {
            config.next_validator_for_delegation = 0;
        } else {
            config.next_validator_for_delegation += 1;
        }
    }

    if propagate.u128() > 0 {
        winning_amount.add_assign(propagate);
    }

    //Withdraw rewards from all validators
    for validator in &config.validators {
        if validator.delegated.u128() > 0 {
            messages.push(withdraw(&validator.address));
        }
    }
    config_helper_store(deps.storage, &config)?;

    pool_state.rewards_returned_to_contract = Uint128::zero();
    pool_state_helper_store(deps.storage, &pool_state)?;

    //Calculate range for each tier
    //*Getting pool liquidity
    let mut pool_state_liquidity_snapshot_obj: PoolLiqState =
        pool_state_liquidity_helper_read_only(deps.storage, round_obj.current_round_index)?;
    if pool_state_liquidity_snapshot_obj.total_liquidity.is_none() {
        pool_state_liquidity_snapshot_obj.total_liquidity = Some(pool_state.total_delegated);
        pool_state_liquidity_snapshot_obj.total_delegated = Some(pool_state.total_delegated);
        pool_state_liquidity_helper_store(
            deps.storage,
            round_obj.current_round_index,
            pool_state_liquidity_snapshot_obj,
        )?;
    }
    //Calculting total number of winners
    let total_number_of_winners: Uint128 = round_obj
        .rewards_distribution
        .tier_5
        .total_number_of_winners
        .add(
            round_obj
                .rewards_distribution
                .tier_4
                .total_number_of_winners,
        )
        .add(
            round_obj
                .rewards_distribution
                .tier_3
                .total_number_of_winners,
        )
        .add(
            round_obj
                .rewards_distribution
                .tier_2
                .total_number_of_winners,
        )
        .add(
            round_obj
                .rewards_distribution
                .tier_1
                .total_number_of_winners,
        )
        .add(
            round_obj
                .rewards_distribution
                .tier_0
                .total_number_of_winners,
        );

    //Calculating the range of each tier using total winners for each tier
    let range_tier_5 = pool_state_liquidity_snapshot_obj
        .total_liquidity
        .unwrap()
        .multiply_ratio(1u128, round_obj.ticket_price.u128())
        .multiply_ratio(1u128, total_number_of_winners);

    let range_tier_4 = round_obj
        .rewards_distribution
        .tier_5
        .total_number_of_winners
        .multiply_ratio(
            1u128,
            round_obj
                .rewards_distribution
                .tier_4
                .total_number_of_winners,
        );
    let range_tier_3 = round_obj
        .rewards_distribution
        .tier_4
        .total_number_of_winners
        .multiply_ratio(
            1u128,
            round_obj
                .rewards_distribution
                .tier_3
                .total_number_of_winners,
        );
    let range_tier_2 = round_obj
        .rewards_distribution
        .tier_3
        .total_number_of_winners
        .multiply_ratio(
            1u128,
            round_obj
                .rewards_distribution
                .tier_2
                .total_number_of_winners,
        );
    let range_tier_1 = round_obj
        .rewards_distribution
        .tier_2
        .total_number_of_winners
        .multiply_ratio(
            1u128,
            round_obj
                .rewards_distribution
                .tier_1
                .total_number_of_winners,
        );
    let range_tier_0 = round_obj
        .rewards_distribution
        .tier_1
        .total_number_of_winners
        .multiply_ratio(
            1u128,
            round_obj
                .rewards_distribution
                .tier_0
                .total_number_of_winners,
        );

    //Generate a sequence of random number between the range defined in round_obj
    let mut hasher = Sha256::new();
    hasher.update(&config.prng_seed);
    hasher.update(&round_obj.entropy);

    let mut winning_sequence_data: WinningSequence = WinningSequence {
        tier_0: DigitsInfo {
            range: range_tier_0,
            winning_number: Default::default(),
        },
        tier_1: DigitsInfo {
            range: range_tier_1,
            winning_number: Default::default(),
        },
        tier_2: DigitsInfo {
            range: range_tier_2,
            winning_number: Default::default(),
        },
        tier_3: DigitsInfo {
            range: range_tier_3,
            winning_number: Default::default(),
        },
        tier_4: DigitsInfo {
            range: range_tier_4,
            winning_number: Default::default(),
        },
        tier_5: DigitsInfo {
            range: range_tier_5,
            winning_number: Default::default(),
        },
    };

    for i in 0u8..6u8 {
        let range_finder = match i {
            0 => range_tier_0,
            1 => range_tier_1,
            2 => range_tier_2,
            3 => range_tier_3,
            4 => range_tier_4,
            5 => range_tier_5,
            _ => Uint128::zero(),
        };

        //**generate a random number between 0 and range
        let mut digit_range = range_finder.u128();
        if digit_range > 0 {
            let d_n = range_finder.checked_sub(Uint128::one());

            if d_n.is_ok() {
                digit_range = d_n?.u128()
            } else {
                return Err(StdError::generic_err("Under-flow sub error 6"));
            }
        }
        let range = Uniform::new_inclusive(0, digit_range);

        let mut final_hasher = hasher.clone();
        final_hasher.update(&vec![i]);
        let seed: [u8; 32] = final_hasher.finalize().into();
        let rng = ChaChaRng::from_seed(seed);
        let mut digit_generator = rng.clone().sample_iter(&range);
        //**We need to draft 6 digits individually

        let drafted_ticket = digit_generator.next().unwrap_or(0u128);
        match i {
            0 => winning_sequence_data.tier_0.winning_number = Uint128::from(drafted_ticket),
            1 => winning_sequence_data.tier_1.winning_number = Uint128::from(drafted_ticket),
            2 => winning_sequence_data.tier_2.winning_number = Uint128::from(drafted_ticket),
            3 => winning_sequence_data.tier_3.winning_number = Uint128::from(drafted_ticket),
            4 => winning_sequence_data.tier_4.winning_number = Uint128::from(drafted_ticket),
            5 => winning_sequence_data.tier_5.winning_number = Uint128::from(drafted_ticket),
            _ => {}
        };
    }

    //* Divide the winning amount among the distribution and save it as a RewardStatsPerRound
    //** Fetching rewards distribution
    let rewards_distribution_obj = &round_obj.rewards_distribution;
    //** Now RewardStats for this round and Saving the reward_stats_for_current_round

    // if config.exp_contract.is_some() {
    //     unclaimed_exp = Some(get_exp(deps.as_ref(), config)?);
    // }

    let reward_stats_for_nth_round: RewardsState = RewardsState {
        //using a helper function to create rewards distribution
        distribution_per_tiers: reward_distribution_per_tier_helper(
            winning_amount,
            &rewards_distribution_obj,
            &config,
        )?,
        ticket_price: round_obj.ticket_price,
        winning_sequence: winning_sequence_data,
        rewards_expiration_date: Some(
            env.block
                .time
                .seconds()
                .add(round_obj.rewards_expiry_duration),
        ),
        total_rewards: winning_amount,
        total_claimed: Uint128::zero(),
        total_exp: unclaimed_exp,
        total_exp_claimed: Some(Uint128::zero()),
    };

    //*Saving the rewards_tickets of current round_obj
    reward_stats_for_nth_round_helper_store(
        deps.storage,
        round_obj.current_round_index,
        &reward_stats_for_nth_round,
    );

    if round_obj.entropy.len() > 1024 {
        round_obj.entropy = round_obj.entropy
            [round_obj.entropy.len() - 1024..(round_obj.entropy.len() - 1)]
            .to_vec();
    }

    round_obj.start_time = env.block.time.seconds();
    round_obj.end_time = env.block.time.seconds().add(round_obj.duration);
    round_obj.current_round_index.add_assign(1u64);
    round_helper_store(deps.storage, &round_obj)?;

    //Sending amount to users
    if shade_share.u128() > 0 {
        let shade_coins: Vec<Coin> = vec![Coin {
            denom: config.denom.to_string(),
            amount: shade_share,
        }];
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: round_obj.shade_rewards_address.to_string(),
            amount: shade_coins,
        }));
    }

    if galactic_pools_share.u128() > 0 {
        let galactic_pools_coins: Vec<Coin> = vec![Coin {
            denom: config.denom.to_string(),
            amount: galactic_pools_share,
        }];
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: round_obj.galactic_pools_rewards_address.to_string(),
            amount: galactic_pools_coins,
        }));
    }

    if trigger_share.u128() > 0 {
        let triggerer_coins: Vec<Coin> = vec![Coin {
            denom: config.denom.to_string(),
            amount: trigger_share,
        }];
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: config.triggerers[0].to_string(),
            amount: triggerer_coins,
        }));
    }

    //TODO: check this
    // if config.exp_contract.is_some() {
    //     messages.push(
    //         experience_contract::msg::ExecuteMsg::UpdateLastClaimed {}.to_cosmos_msg(
    //             BLOCK_SIZE,
    //             config.exp_contract.clone().unwrap().contract.hash,
    //             config.exp_contract.clone().unwrap().contract.address,
    //             None,
    //         )?,
    //     );
    // }

    let res = Response::new()
        .add_messages(messages)
        .set_data(to_binary(&HandleAnswer::EndRound { status: Success })?);

    Ok(res)
}

/// Returns StdResult<Response>
///
/// Adds admin to the vec of admins
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a reference to the Config
/// * `admin` - String for the admin
fn try_add_admin(
    deps: DepsMut,
    info: MessageInfo,
    config: &mut ConfigInfo,
    admin: Addr,
) -> StdResult<Response> {
    //Only admin(s) can access this functionality

    check_if_admin(&config, &info.sender)?;

    if config.admins.contains(&admin) {
        return Err(StdError::generic_err("This address already exisits"));
    } else {
        config.admins.push(admin);
    }

    config_helper_store(deps.storage, config)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::AddAdmin { status: Success })?))
}

/// Returns StdResult<Response>
///
/// Removes admin to the vec of admins
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a reference to the Config
/// * `admin` - String for the admin
fn try_remove_admin(
    deps: DepsMut,
    info: MessageInfo,
    config: &mut ConfigInfo,
    admin: Addr,
) -> StdResult<Response> {
    let _ = check_if_admin(&config, &info.sender)?;

    if config.admins.contains(&admin) {
        config.admins.retain(|ad| ad != &admin);
    } else {
        return Err(StdError::generic_err("This address doesn't exisits"));
    }
    config_helper_store(deps.storage, config)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::RemoveAdmin { status: Success })?))
}

/// Returns StdResult<Response>
///
/// Adds triggerer to the vec of triggerers
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a reference to the Config
/// * `triggerer` - String for the triggerer
fn try_add_triggerer(
    deps: DepsMut,
    info: MessageInfo,
    config: &mut ConfigInfo,
    triggerer: Addr,
) -> StdResult<Response> {
    //Only admin(s) can access this functionality
    check_if_admin(&config, &info.sender)?;

    if config.triggerers.contains(&triggerer) {
        return Err(StdError::generic_err("This address already exisits"));
    } else {
        config.triggerers.push(triggerer);
    }

    config_helper_store(deps.storage, config)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::AddTriggerer { status: Success })?))
}

/// Returns StdResult<Response>
///
/// Removes triggerer to the vec of triggerers
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a reference to the Config
/// * `triggerer` - String for the triggerer
fn try_remove_triggerer(
    deps: DepsMut,
    info: MessageInfo,
    config: &mut ConfigInfo,
    triggerer: Addr,
) -> StdResult<Response> {
    check_if_admin(&config, &info.sender)?;

    if config.triggerers.contains(&triggerer) {
        config.triggerers.retain(|ad| ad != &triggerer);
    } else {
        return Err(StdError::generic_err("This address doesn't exisits"));
    }
    config_helper_store(deps.storage, config)?;

    Ok(
        Response::new().set_data(to_binary(&HandleAnswer::RemoveTriggerer {
            status: Success,
        })?),
    )
}

/// Returns StdResult<Response>
///
/// Adds reviewer to the vec of reviewers
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a reference to the Config
/// * `reviewer` - String for the reviewer
fn try_add_reviewer(
    deps: DepsMut,
    info: MessageInfo,
    config: &mut ConfigInfo,
    reviewer: Addr,
) -> StdResult<Response> {
    //Only admin(s) can access this functionality
    check_if_admin(&config, &info.sender)?;

    if config.reviewers.contains(&reviewer) {
        return Err(StdError::generic_err("This address already exisits"));
    } else {
        config.reviewers.push(reviewer);
    }

    config_helper_store(deps.storage, config)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::AddReviewer { status: Success })?))
}

/// Returns StdResult<Response>
///
/// Removes reviewer to the vec of reviewers
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a reference to the Config
/// * `reviewer` - String for the reviewer
fn try_remove_reviewer(
    deps: DepsMut,
    info: MessageInfo,
    config: &mut ConfigInfo,
    reviewer: Addr,
) -> StdResult<Response> {
    let _ = check_if_admin(&config, &info.sender)?;

    if config.reviewers.contains(&reviewer) {
        config.reviewers.retain(|ad| ad != &reviewer);
    } else {
        return Err(StdError::generic_err("This address doesn't exisits"));
    }
    config_helper_store(deps.storage, config)?;

    Ok(
        Response::new().set_data(to_binary(&HandleAnswer::RemoveReviewer {
            status: Success,
        })?),
    )
}

/// Returns StdResult<Response>
///
/// Changes the configuration of the contract.
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a reference to the Config
/// * `admin` - String for the admin
/// * `triggerer` - String for the triggerer
/// * `reviewer` - String for the reviewer
/// * `sscrt` - contract details of sscrt contract
/// * `unbonding_batch_duration` - time in seconds it takes before next batch is unbonded
/// * `unbonding_duration` - time in seconds taken by this chain to unbond the tokens delegated
/// * `minimum_deposit_amount` - Optional time in seconds taken by this chain to unbond the tokens delegated
fn try_update_config(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    config: &mut ConfigInfo,
    unbonding_batch_duration: Option<u64>,
    unbonding_duration: Option<u64>,
    minimum_deposit_amount: Option<Uint128>,
    exp_contract: Option<ExpContract>,
) -> StdResult<Response> {
    let _ = check_if_admin(&config, &info.sender)?;

    if let Some(duration) = unbonding_batch_duration {
        config.unbonding_batch_duration = duration;
    }

    if let Some(duration) = unbonding_duration {
        config.unbonding_duration = duration;
    }

    if let Some(amount) = minimum_deposit_amount {
        config.minimum_deposit_amount = Some(amount);
    }

    let mut msgs = Vec::<CosmosMsg>::new();

    // TODO: check this
    // if exp_contract.is_some() {
    //     if let Some(exp_c) = exp_contract.clone() {
    //         let set_vk_msg = experience_contract::msg::ExecuteMsg::SetViewingKey { key: exp_c.vk };

    //         msgs.push(set_vk_msg.to_cosmos_msg(
    //             BLOCK_SIZE,
    //             exp_c.contract.hash,
    //             exp_c.contract.address,
    //             None,
    //         )?);
    //     }
    // }

    config.exp_contract = exp_contract;

    config_helper_store(deps.storage, &config)?;

    Ok(Response::new()
        .set_data(to_binary(&HandleAnswer::UpdateConfig { status: Success })?)
        .add_messages(msgs))
}

/// Returns StdResult<Response>
///
/// Changes the configuration of the contract.
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a reference to the Config
/// * `duration` - duration of the round
/// * `rewards_distribution` - rewards distribution of rewards between each tier
/// * `ticket_price` - price per one ticket
/// * `rewards_expiry_duration` -  duration after round ends after which prizes are expired.
/// * `triggerer_share_percentage` - % of rewards for triggerer to maintain this contract
/// * `shade_rewards_address` - shade's dao address
/// * `galactic_pools_rewards_address` - galacticpool's dao address
/// * `grand_prize_address` - grand-prize contract address
/// * `unclaimed_distribution` - distribution of unclaimed rewards
fn try_update_round(
    deps: DepsMut,
    info: MessageInfo,
    config: &mut ConfigInfo,
    duration: Option<u64>,
    rewards_distribution: Option<RewardsDistInfo>,
    ticket_price: Option<Uint128>,
    rewards_expiry_duration: Option<u64>,
    admin_share: Option<AdminShareInfo>,
    triggerer_share_percentage: Option<u64>,
    shade_rewards_address: Option<Addr>,
    galactic_pools_rewards_address: Option<Addr>,
    grand_prize_address: Option<Addr>,
    unclaimed_distribution: Option<UnclaimedDistInfo>,
) -> StdResult<Response> {
    let _ = check_if_admin(&config, &info.sender)?;

    let mut round_obj = round_helper_read_only(deps.storage)?;

    if duration.is_some() {
        round_obj.duration = duration.unwrap();
    }
    if rewards_distribution.is_some() {
        round_obj.rewards_distribution = rewards_distribution.unwrap();
    }
    if ticket_price.is_some() {
        round_obj.ticket_price = ticket_price.unwrap();
    }
    if rewards_expiry_duration.is_some() {
        round_obj.rewards_expiry_duration = rewards_expiry_duration.unwrap();
    }
    if admin_share.is_some() {
        if admin_share
            .unwrap()
            .shade_percentage_share
            .add(admin_share.unwrap().galactic_pools_percentage_share)
            != config.common_divisor as u64
        {
            return Err(StdError::generic_err(
                "Total percentage shares don't add up to 100%",
            ));
        }
        round_obj.admin_share = admin_share.unwrap();
    }
    if triggerer_share_percentage.is_some() {
        round_obj.triggerer_share_percentage = triggerer_share_percentage.unwrap();
    }
    if shade_rewards_address.is_some() {
        round_obj.shade_rewards_address = shade_rewards_address.unwrap();
    }
    if galactic_pools_rewards_address.is_some() {
        round_obj.galactic_pools_rewards_address = galactic_pools_rewards_address.unwrap();
    }
    if grand_prize_address.is_some() {
        round_obj.grand_prize_address = grand_prize_address.unwrap();
    }
    if unclaimed_distribution.is_some() {
        round_obj.unclaimed_distribution = unclaimed_distribution.unwrap();
    }

    round_helper_store(deps.storage, &round_obj)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::UpdateRound { status: Success })?))
}

/// Returns StdResult<Response>
///
/// PoolParty community can request to withdraw the funds from the contract.
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a mutable reference to the Config
/// * `request_withdraw_amount` - amount requested to withdraw
fn try_request_reserves_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    config: &mut ConfigInfo,
    request_withdraw_amount: Uint128,
) -> StdResult<Response> {
    //Loading Structs
    let _ = check_if_admin(&config, &info.sender)?;
    let mut pool_state = pool_state_helper_read_only(deps.storage)?;

    // Checking if requested amount is more than amount available
    if pool_state.total_reserves < request_withdraw_amount {
        return Err(StdError::generic_err(format!(
            "Insufficient funds to redeem: balance={}, required={}",
            pool_state.total_reserves, request_withdraw_amount
        )));
    }
    if request_withdraw_amount == Uint128::zero() {
        return Err(StdError::generic_err(format!(
            "Cannot withdraw 0 {}",
            config.denom
        )));
    }
    // Putting in request withdraw
    // pool_state
    //     .total_reserves
    //     .sub_assign(request_withdraw_amount);

    pool_state.total_reserves = if let Ok(t_r) = pool_state
        .total_reserves
        .checked_sub(request_withdraw_amount)
    {
        t_r
    } else {
        return Err(StdError::generic_err("Under-flow sub error"));
    };

    //STORING USER UNBONDING
    if !pool_state
        .unbonding_batches
        .contains(&config.next_unbonding_batch_index)
    {
        pool_state
            .unbonding_batches
            .push(config.next_unbonding_batch_index);
    }

    let mut unbonding_amount =
        admin_unbond_helper_read_only(deps.storage, config.next_unbonding_batch_index)?;

    unbonding_amount.add_assign(request_withdraw_amount);

    admin_unbond_helper_store(
        deps.storage,
        config.next_unbonding_batch_index,
        unbonding_amount,
    )?;

    //Asking the validator to undelegate the funds when unbond the batch
    pool_state_helper_store(deps.storage, &pool_state)?;
    config
        .next_unbonding_batch_amount
        .add_assign(request_withdraw_amount);
    config_helper_store(deps.storage, &config)?;

    let res = Response::new().set_data(to_binary(&HandleAnswer::RequestAdminWithdraw {
        status: Success,
    })?);

    Ok(res)
}

/// Returns StdResult<Response>
///
/// GalacticPools community withdraw their requested funds. It take 21 days to withdraw the funds after the request is made.
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a mutable reference to the Config
/// * `withdraw_amount` - amount to withdraw
fn try_reserve_withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: &mut ConfigInfo,
    withdraw_amount: Uint128,
) -> StdResult<Response> {
    //loading Data from storage
    let mut pool_state_obj: PoolState = pool_state_helper_read_only(deps.storage)?;
    let _ = check_if_admin(&config, &info.sender)?;

    let mut admin_withdraw_obj = admin_withdraw_helper_read_only(deps.storage)?;

    //Checking amount available for withdraw
    let mut amount_av_for_withdraw = Uint128::zero();

    //STORING USER UNBONDING

    let mut pop_front_counter: Vec<u64> = vec![];

    for i in 0..pool_state_obj.unbonding_batches.len() {
        let unbond_batch_index = pool_state_obj.unbonding_batches[i];
        let unbonding_batch_obj =
            unbonding_batch_helper_read_only(deps.storage, unbond_batch_index)?;

        if unbonding_batch_obj.unbonding_time.is_some() {
            if env.block.time.seconds() >= unbonding_batch_obj.unbonding_time.unwrap() {
                let unbonding_amount =
                    admin_unbond_helper_read_only(deps.storage, unbond_batch_index)?;

                amount_av_for_withdraw.add_assign(unbonding_amount);
                pop_front_counter.push(unbond_batch_index);
            }
        }
    }

    pool_state_obj
        .unbonding_batches
        .retain(|val| !pop_front_counter.contains(val));

    //ERROR Check
    if withdraw_amount > admin_withdraw_obj.add(amount_av_for_withdraw) {
        return Err(StdError::generic_err(
            "Trying to withdraw more than available",
        ));
    }

    //Updating user and   pool state
    pool_state_helper_store(deps.storage, &pool_state_obj)?;

    admin_withdraw_obj.add_assign(amount_av_for_withdraw);
    // admin_withdraw_obj.sub_assign(withdraw_amount);
    admin_withdraw_obj = if let Ok(a_w_o) = admin_withdraw_obj.checked_sub(withdraw_amount) {
        a_w_o
    } else {
        return Err(StdError::generic_err("Under-flow sub error"));
    };
    admin_withdraw_helper_store(deps.storage, &admin_withdraw_obj)?;

    let mut messages: Vec<CosmosMsg> = vec![];
    let withdraw_coins: Vec<Coin> = vec![Coin {
        denom: config.denom.to_string(),
        amount: withdraw_amount,
    }];
    //Sending a message to withdraw
    let round_obj = round_helper_read_only(deps.storage)?;

    if withdraw_amount > Uint128::zero() {
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: (&round_obj.grand_prize_address).to_string(),
            amount: withdraw_coins,
        }));
    }

    let res = Response::new().add_messages(messages).set_data(to_binary(
        &HandleAnswer::ReservesWithdraw { status: Success },
    )?);

    Ok(res)
}

/// Returns StdResult<Response>
///
/// User unbonding requests are done in batches since only 7 unbondings are allowed to a single address at an instance.
/// It take 21 days to withdraw the funds after the request is made.
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a mutable reference to the Config
fn try_unbond_batch(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: &mut ConfigInfo,
) -> StdResult<Response> {
    check_if_triggerer(&config, &info.sender)?;
    //Loading data from storage
    let mut pool_state = pool_state_helper_read_only(deps.storage)?;

    //Checking if triggerer can batch unbond
    if env.block.time.seconds() < config.next_unbonding_batch_time {
        return Err(StdError::generic_err(format!(
            "Cannot unbond right now. You can unbond at {}",
            config.next_unbonding_batch_time
        )));
    }

    if config.next_unbonding_batch_amount == Uint128::zero() {
        config.next_unbonding_batch_index.add_assign(1u64);
        config.next_unbonding_batch_time = env
            .block
            .time
            .seconds()
            .add(config.unbonding_batch_duration);
        config_helper_store(deps.storage, &config)?;

        let res =
            Response::new().set_data(to_binary(&HandleAnswer::UnbondBatch { status: Success })?);

        return Ok(res);
    }

    //Creating undelegate messages from validators list
    let mut remaining_withdraw_amount = config.next_unbonding_batch_amount;
    let mut validators_used: Vec<Validator> = Vec::new();
    let mut messages: Vec<CosmosMsg> = vec![];

    for _ in 0..config.validators.len() {
        let index = config.next_validator_for_unbonding as usize;
        let withdraw_amount: Uint128;
        if config.validators[index].delegated.u128() >= remaining_withdraw_amount.u128() {
            withdraw_amount = remaining_withdraw_amount;
        } else {
            withdraw_amount = config.validators[index].delegated;
        }

        //Unbonding message
        if withdraw_amount > Uint128::zero() {
            config.validators[index].delegated = if let Ok(del) = config.validators[index]
                .delegated
                .checked_sub(withdraw_amount)
            {
                del
            } else {
                return Err(StdError::generic_err("Under-flow sub error"));
            };
            remaining_withdraw_amount =
                if let Ok(r_w_a) = remaining_withdraw_amount.checked_sub(withdraw_amount) {
                    r_w_a
                } else {
                    return Err(StdError::generic_err("Under-flow sub error"));
                };
            validators_used.push(config.validators[index].clone());

            messages.push(undelegate(
                &config.validators[index].address,
                withdraw_amount,
                &config.denom,
            ));
            if index == config.validators.len() - 1 {
                config.next_validator_for_unbonding = 0;
            } else {
                config.next_validator_for_unbonding += 1;
            }
        }

        if remaining_withdraw_amount.is_zero() {
            break;
        }
    }

    //Querying pending_rewards send back from validator
    let rewards = get_rewards(deps.as_ref(), &env.contract.address, &config)?;
    let mut rewards_amount = Uint128::zero();
    for val in &validators_used {
        for reward in &rewards {
            if val.address.as_str().eq(reward.validator_address.as_str()) {
                rewards_amount.add_assign(reward.reward);
            }
        }
    }

    //Adding pending_rewards to total rewards returned
    if rewards_amount > Uint128::zero() {
        pool_state
            .rewards_returned_to_contract
            .add_assign(rewards_amount);
    }
    pool_state_helper_store(deps.storage, &pool_state)?;

    //Storing this_unbonding_batch
    let unbonding_batch_obj = UnbondingBatch {
        unbonding_time: Some(env.block.time.seconds().add(config.unbonding_duration)),
        amount: Some(config.next_unbonding_batch_amount),
    };
    unbonding_batch_helper_store(
        deps.storage,
        config.next_unbonding_batch_index,
        &unbonding_batch_obj,
    )?;

    config.next_unbonding_batch_amount = Uint128::zero();
    config.next_unbonding_batch_index.add_assign(1u64);
    config.next_unbonding_batch_time = env
        .block
        .time
        .seconds()
        .add(config.unbonding_batch_duration);
    config_helper_store(deps.storage, &config)?;

    let res = Response::new()
        .add_messages(messages)
        .set_data(to_binary(&HandleAnswer::UnbondBatch { status: Success })?);

    Ok(res)
}

/// Returns StdResult<Response>
///
/// Helps rebalance amount delegated to the validators
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a reference to the Config
fn try_rebalance_validator_set(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: &mut ConfigInfo,
) -> StdResult<Response> {
    let mut pool_state = pool_state_helper_read_only(deps.storage)?;
    check_if_admin(&config, &info.sender)?;

    //Re-balance
    //Create a combined vector

    let mut messages: Vec<CosmosMsg> = vec![];
    let mut excess: Vec<(String, Uint128)> = Vec::new();
    let mut less: Vec<(String, Uint128)> = Vec::new();
    let mut rewards_obj = get_rewards(deps.as_ref(), &env.contract.address, &config).unwrap();

    //Separating validators with more than ideal and less than ideal % of amount.
    for validator in &mut config.validators {
        let ideal_delegated_amount = pool_state
            .total_delegated
            .multiply_ratio(validator.weightage, config.common_divisor);
        if validator.delegated > ideal_delegated_amount {
            let difference =
                if let Ok(diff) = validator.delegated.checked_sub(ideal_delegated_amount) {
                    diff
                } else {
                    return Err(StdError::generic_err("Under-flow sub error"));
                };
            validator.delegated = if let Ok(del) = validator.delegated.checked_sub(difference) {
                del
            } else {
                return Err(StdError::generic_err("Under-flow sub error"));
            };

            let per_filled: u64;
            if validator.weightage == 0 {
                per_filled = 0;
            } else {
                per_filled = ((validator
                    .delegated
                    .multiply_ratio(config.common_divisor, 1u128))
                .multiply_ratio(
                    1u128,
                    pool_state
                        .total_delegated
                        .add(pool_state.total_sponsored)
                        .add(pool_state.total_reserves)
                        .multiply_ratio(validator.weightage as u128, config.common_divisor as u128)
                        .u128(),
                ))
                .u128() as u64;
            }

            validator.percentage_filled = per_filled;
            excess.push((validator.address.as_mut().into(), difference));
        } else if validator.delegated == ideal_delegated_amount {
            //Do nothing
        } else {
            let difference =
                if let Ok(diff) = ideal_delegated_amount.checked_sub(validator.delegated) {
                    diff
                } else {
                    return Err(StdError::generic_err("Under-flow sub error"));
                };

            less.push((validator.address.as_mut().into(), difference));
        }
    }

    for mut excess_val in &mut excess {
        for less_val in &mut less {
            //restake and update the total_delegated and percentage_filled
            let restake_amount: Uint128;
            //Calculating amount to restake
            if less_val.1 <= excess_val.1 {
                restake_amount = less_val.1;
            } else {
                restake_amount = excess_val.1;
            }
            excess_val.1 = if let Ok(e_v) = excess_val.1.checked_sub(restake_amount) {
                e_v
            } else {
                return Err(StdError::generic_err("Under-flow sub error"));
            };
            less_val.1 = if let Ok(l_v) = less_val.1.checked_sub(restake_amount) {
                l_v
            } else {
                return Err(StdError::generic_err("Under-flow sub error"));
            };
            if restake_amount > Uint128::zero() {
                messages.push(redelegate(
                    &excess_val.0,
                    &less_val.0,
                    restake_amount,
                    &config.denom,
                ));
            }

            for r in &mut rewards_obj {
                if r.reward.u128() > 0_u128 {
                    if r.validator_address.as_str() == excess_val.0.as_str() {
                        pool_state.rewards_returned_to_contract.add_assign(r.reward);
                        r.reward = Uint128::zero();
                    }
                    if r.validator_address.as_str() == less_val.0.as_str() {
                        pool_state.rewards_returned_to_contract.add_assign(r.reward);
                        r.reward = Uint128::zero();
                    }
                }
            }

            //Update the total_staking and percentage_filled
            for validator in &mut config.validators {
                if validator.address.as_str() == less_val.0.as_str() {
                    validator.delegated.add_assign(restake_amount);

                    let per_filled;
                    if validator.weightage == 0 {
                        per_filled = 0;
                    } else {
                        if pool_state
                            .total_delegated
                            .add(pool_state.total_sponsored)
                            .add(pool_state.total_reserves)
                            .u128()
                            > 0
                        {
                            per_filled = ((validator
                                .delegated
                                .multiply_ratio(config.common_divisor, 1u128))
                            .multiply_ratio(
                                1u128,
                                pool_state
                                    .total_delegated
                                    .add(pool_state.total_sponsored)
                                    .add(pool_state.total_reserves)
                                    .multiply_ratio(
                                        validator.weightage as u128,
                                        config.common_divisor as u128,
                                    )
                                    .u128(),
                            ))
                            .u128() as u64;
                        } else {
                            per_filled = 0u64;
                        }
                    }
                    validator.percentage_filled = per_filled;
                    break;
                }
            }
            if excess_val.1.is_zero() {
                break;
            }
        }
    }

    config.next_validator_for_delegation = 0;
    config.next_validator_for_unbonding = 0;
    config_helper_store(deps.storage, &config)?;
    pool_state_helper_store(deps.storage, &pool_state)?;

    let res = Response::new().set_data(to_binary(&HandleAnswer::RebalanceValidatorSet {
        status: Success,
    })?);

    Ok(res)
}

/// Returns StdResult<Response>
///
/// Helps add, remove new validators. Plus options to change the % weight of the validators.
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a reference to the Config
/// * `updated_val_set` - updated validator set with removed validators % weightage set to zero.
fn try_update_validator_set(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: &mut ConfigInfo,
    updated_val_set: Vec<ValidatorInfo>,
) -> StdResult<Response> {
    let mut pool_state = pool_state_helper_read_only(deps.storage)?;
    check_if_admin(&config, &info.sender)?;

    //Re-balance
    let mut final_validator_set: Vec<Validator> = Vec::new();

    //Create a combined vector

    for local_val in updated_val_set {
        //Get total delegated
        let index = config
            .validators
            .iter()
            .position(|val| val.address.as_str().eq(local_val.address.as_str()));
        if index.is_some() {
            let per_filled: u64;
            if local_val.weightage == 0 {
                per_filled = 0;
            } else {
                per_filled = ((config.validators[index.unwrap()]
                    .delegated
                    .multiply_ratio(config.common_divisor, 1u128))
                .multiply_ratio(
                    1u128,
                    pool_state
                        .total_delegated
                        .add(pool_state.total_sponsored)
                        .multiply_ratio(local_val.weightage as u128, config.common_divisor as u128)
                        .u128(),
                ))
                .u128() as u64;
            }

            final_validator_set.push(Validator {
                address: local_val.address,
                delegated: config.validators[index.unwrap()].delegated,
                weightage: local_val.weightage,
                percentage_filled: per_filled,
            })
        } else {
            final_validator_set.push(Validator {
                address: local_val.address,
                delegated: Uint128::zero(),
                weightage: local_val.weightage,
                percentage_filled: 0,
            });
        }
    }

    let mut messages: Vec<CosmosMsg> = vec![];
    let mut excess: Vec<(String, Uint128)> = Vec::new();
    let mut less: Vec<(String, Uint128)> = Vec::new();
    let mut rewards_obj = get_rewards(deps.as_ref(), &env.contract.address, &config).unwrap();

    //Separating validators with more than ideal and less than ideal % of amount.
    for val in &mut final_validator_set {
        let ideal_delegated_amount = pool_state
            .total_delegated
            .multiply_ratio(val.weightage, config.common_divisor);
        if val.delegated > ideal_delegated_amount {
            let difference = if let Ok(diff) = val.delegated.checked_sub(ideal_delegated_amount) {
                diff
            } else {
                return Err(StdError::generic_err("Under-flow sub error"));
            };
            val.delegated = if let Ok(del) = val.delegated.checked_sub(difference) {
                del
            } else {
                return Err(StdError::generic_err("Under-flow sub error"));
            };

            let per_filled: u64;
            if val.weightage == 0 {
                per_filled = 0;
            } else {
                per_filled = ((val.delegated.multiply_ratio(config.common_divisor, 1u128))
                    .multiply_ratio(
                        1u128,
                        pool_state
                            .total_delegated
                            .add(pool_state.total_sponsored)
                            .add(pool_state.total_reserves)
                            .multiply_ratio(val.weightage as u128, config.common_divisor as u128)
                            .u128(),
                    ))
                .u128() as u64;
            }

            val.percentage_filled = per_filled;
            excess.push((val.address.as_mut().into(), difference));
        } else if val.delegated == ideal_delegated_amount {
            //Do nothing
        } else {
            let difference = if let Ok(del) = ideal_delegated_amount.checked_sub(val.delegated) {
                del
            } else {
                return Err(StdError::generic_err("Under-flow sub error"));
            };
            less.push((val.address.as_mut().into(), difference));
        }
    }

    for mut excess_val in &mut excess {
        for less_val in &mut less {
            //restake and update the total_delegated and percentage_filled
            let restake_amount: Uint128;
            //Calculating amount to restake
            if less_val.1 <= excess_val.1 {
                restake_amount = less_val.1;
            } else {
                restake_amount = excess_val.1;
            }
            excess_val.1 = if let Ok(e_v) = excess_val.1.checked_sub(restake_amount) {
                e_v
            } else {
                return Err(StdError::generic_err("Under-flow sub error"));
            };
            less_val.1 = if let Ok(l_v) = less_val.1.checked_sub(restake_amount) {
                l_v
            } else {
                return Err(StdError::generic_err("Under-flow sub error"));
            };
            messages.push(redelegate(
                &excess_val.0,
                &less_val.0,
                restake_amount,
                &config.denom,
            ));

            for r in &mut rewards_obj {
                if r.reward.u128() > 0 {
                    if r.validator_address.as_str() == excess_val.0.as_str() {
                        pool_state.rewards_returned_to_contract.add_assign(r.reward);
                        r.reward = Uint128::zero();
                    }
                    if r.validator_address.as_str() == less_val.0.as_str() {
                        pool_state.rewards_returned_to_contract.add_assign(r.reward);
                        r.reward = Uint128::zero();
                    }
                }
            }

            //Update the total_staking and percentage_filled
            for validator in &mut final_validator_set {
                if validator.address.as_str() == less_val.0.as_str() {
                    validator.delegated.add_assign(restake_amount);

                    let per_filled;
                    if validator.weightage == 0 {
                        per_filled = 0;
                    } else {
                        if pool_state
                            .total_delegated
                            .add(pool_state.total_sponsored)
                            .add(pool_state.total_reserves)
                            .u128()
                            > 0
                        {
                            per_filled = ((validator
                                .delegated
                                .multiply_ratio(config.common_divisor, 1u128))
                            .multiply_ratio(
                                1u128,
                                pool_state
                                    .total_delegated
                                    .add(pool_state.total_sponsored)
                                    .add(pool_state.total_reserves)
                                    .multiply_ratio(
                                        validator.weightage as u128,
                                        config.common_divisor as u128,
                                    )
                                    .u128(),
                            ))
                            .u128() as u64;
                        } else {
                            per_filled = 0u64;
                        }
                    }
                    validator.percentage_filled = per_filled;
                    break;
                }
            }
            if excess_val.1.is_zero() {
                break;
            }
        }
    }

    final_validator_set.retain(|v| v.weightage != 0);

    config.validators = final_validator_set;
    config.next_validator_for_delegation = 0;
    config.next_validator_for_unbonding = 0;
    config_helper_store(deps.storage, &config)?;
    pool_state_helper_store(deps.storage, &pool_state)?;

    let res = Response::new().set_data(to_binary(&HandleAnswer::UpdateValidatorSet {
        status: Success,
    })?);

    Ok(res)
}

/// Returns StdResult<Response>
///
/// set the contract status level
///
/// # Arguments
///
/// * `deps` - mutable reference to Extern containing all the contract's external dependencies
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a mutable reference to the Config
/// * `level` - new ContractStatus
fn try_set_contract_status(
    deps: DepsMut,
    info: MessageInfo,
    config: &mut ConfigInfo,
    level: ContractStatus,
) -> StdResult<Response> {
    let _ = check_if_admin(&config, &info.sender)?;
    let new_status = level.to_u8();
    if config.status != new_status {
        config.status = new_status;
        config_helper_store(deps.storage, &config)?;
    }
    Ok(
        Response::new().set_data(to_binary(&HandleAnswer::SetContractStatus {
            status: Success,
        })?),
    )
}

///////////////////////////////////////// Helper for Handle Function //////////////////////////////////////
/// Returns StdResult<()> that will error if the priority level of the action is not
/// equal to or greater than the current contract status level
///
/// # Arguments
///
/// * `contract_status` - u8 representation of the current contract status
/// * `priority` - u8 representing the highest status level this action may execute at
fn check_status(contract_status: u8, priority: u8) -> StdResult<()> {
    if priority < contract_status {
        return Err(StdError::generic_err(
            "The contract admin has temporarily disabled this action",
        ));
    }
    Ok(())
}

/// Returns StdResult<()>
///
/// Checks if the 'account' send is admin address or not
///
/// # Arguments
///
///
/// * `config` - a reference to the Config
/// * `account` - the account address to check
fn check_if_admin(config: &ConfigInfo, account: &Addr) -> StdResult<()> {
    if config.admins.contains(account) == false {
        return Err(StdError::generic_err(
            "This is an admin command. Admin commands can only be run from admin address",
        ));
    }

    Ok(())
}

/// Returns StdResult<()>
///
/// Checks if the 'account' send is triggerer address or not
///
/// # Arguments
///
///
/// * `config` - a reference to the Config
/// * `account` - the account address to check
fn check_if_triggerer(config: &ConfigInfo, account: &Addr) -> StdResult<()> {
    if !&config.triggerers.contains(account) {
        return Err(StdError::generic_err(
            "This is an triggerer command. Triggerer commands can only be run from triggerer address",
        ));
    }

    Ok(())
}

/// Returns StdResult<()>
///
/// Checks if the 'account' send is triggerer address or not
///
/// # Arguments
///
///
/// * `config` - a reference to the Config
/// * `account` - the account address to check
fn check_if_reviewer(config: &ConfigInfo, account: &Addr) -> StdResult<()> {
    if !&config.reviewers.contains(account) {
        return Err(StdError::generic_err(
            "This is an reviewer command. Reviewer commands can only be run from reviewer address",
        ));
    }

    Ok(())
}

/// Returns StdResult<()>
///
/// Checks if current round can be ended. validate_end_time returns an error if the round ends in the future
///
/// # Arguments
///
///
/// * `end_time` - a reference to the Config
/// * `current_time` - the account address to check
fn validate_end_time(end_time: u64, current_time: u64) -> StdResult<()> {
    if current_time < end_time {
        Err(StdError::generic_err("Round end time is in the future"))
    } else {
        Ok(())
    }
}

/// Returns StdResult<Uint128>
///
/// Checks if the deposit amount deposited is greater than the minimum deposit amount, if their is any minimum deposit amount.
///
/// # Arguments
///
/// * `info` - It contains the essential info for authorization - identity of the call, and payment.
/// * `config` - a  reference to the Config
fn check_if_valid_amount(info: &MessageInfo, config: &ConfigInfo) -> StdResult<Uint128> {
    let mut deposit_amount = Uint128::zero();

    for coin in &info.funds {
        if coin.denom == config.denom {
            deposit_amount = coin.amount
        } else {
            return Err(StdError::generic_err(format!(
                "Wrong token given, expected {} found {}",
                config.denom, coin.denom
            )));
        }
    }

    if config.minimum_deposit_amount.is_some() {
        if deposit_amount < config.minimum_deposit_amount.unwrap() {
            return Err(StdError::generic_err(format!(
                "Must deposit a minimum of {} {}",
                config.minimum_deposit_amount.unwrap(),
                config.denom
            )));
        }
    }

    if deposit_amount == Uint128::zero() {
        return Err(StdError::generic_err(format!(
            "Must deposit atleast one {}",
            config.denom
        )));
    }

    return Ok(deposit_amount);
}

/// Returns StdResult<()> that will error if the starting round and current_round_index are same.
///
/// # Arguments
///
/// * `starting_round` - u64
/// * `current_round_index` - u64
fn check_if_claimable(starting_round: Option<u64>, current_round_index: u64) -> StdResult<()> {
    if starting_round.is_none() {
        return Err(StdError::generic_err(format!(
            "You have not deposited to the pook contract. No tickets found"
        )));
    } else {
        if starting_round.unwrap() == current_round_index {
            return Err(StdError::generic_err(format!(
                "You are not yet able to claim rewards. Wait for this round to end"
            )));
        }
    }
    Ok(())
}

/// Returns RewardStatePerTier for a each of the 6 tiers. Tier 0 is the highest tier. and Tier 5 is the lowest tier.
///
/// Helper function used by EndRound Handle - helps calculate the rewards distribution for a given winning amount per tier this round.
///
/// # Arguments
///
/// * `winning_amount` - The total winning amount for this round.
/// * `rewards_distribution_obj` - reference to the rewards distribution object specified in the round config
/// * `config` - a  reference to the Config
fn reward_distribution_per_tier_helper(
    winning_amount: Uint128,
    rewards_distribution_obj: &RewardsDistInfo,
    config: &ConfigInfo,
) -> StdResult<TierState> {
    let tier_1_total_rewards = winning_amount.multiply_ratio(
        rewards_distribution_obj.tier_1.percentage_of_rewards,
        config.common_divisor,
    );
    let tier_1_reward_per_match = tier_1_total_rewards.multiply_ratio(
        1u64,
        rewards_distribution_obj.tier_1.total_number_of_winners,
    );

    let tier_2_total_rewards = winning_amount.multiply_ratio(
        rewards_distribution_obj.tier_2.percentage_of_rewards,
        config.common_divisor,
    );
    let tier_2_reward_per_match = tier_2_total_rewards.multiply_ratio(
        1u64,
        rewards_distribution_obj.tier_2.total_number_of_winners,
    );

    let tier_3_total_rewards = winning_amount.multiply_ratio(
        rewards_distribution_obj.tier_3.percentage_of_rewards,
        config.common_divisor,
    );
    let tier_3_reward_per_match = tier_3_total_rewards.multiply_ratio(
        1u64,
        rewards_distribution_obj.tier_3.total_number_of_winners,
    );

    let tier_4_total_rewards = winning_amount.multiply_ratio(
        rewards_distribution_obj.tier_4.percentage_of_rewards,
        config.common_divisor,
    );
    let tier_4_reward_per_match = tier_4_total_rewards.multiply_ratio(
        1u64,
        rewards_distribution_obj.tier_4.total_number_of_winners,
    );

    let tier_5_total_rewards = winning_amount.multiply_ratio(
        rewards_distribution_obj.tier_5.percentage_of_rewards,
        config.common_divisor,
    );
    let tier_5_reward_per_match = tier_5_total_rewards.multiply_ratio(
        1u64,
        rewards_distribution_obj.tier_5.total_number_of_winners,
    );

    let tier_1_2_3_4_5_total_rewards = tier_1_reward_per_match
        .multiply_ratio(
            rewards_distribution_obj.tier_1.total_number_of_winners,
            1u128,
        )
        .add(tier_2_reward_per_match.multiply_ratio(
            rewards_distribution_obj.tier_2.total_number_of_winners,
            1u128,
        ))
        .add(tier_3_reward_per_match.multiply_ratio(
            rewards_distribution_obj.tier_3.total_number_of_winners,
            1u128,
        ))
        .add(tier_4_reward_per_match.multiply_ratio(
            rewards_distribution_obj.tier_4.total_number_of_winners,
            1u128,
        ))
        .add(tier_5_reward_per_match.multiply_ratio(
            rewards_distribution_obj.tier_5.total_number_of_winners,
            1u128,
        ));

    let tier_0_total_rewards =
        if let Ok(w_am) = winning_amount.checked_sub(tier_1_2_3_4_5_total_rewards) {
            w_am
        } else {
            return Err(StdError::generic_err("Under-flow sub error"));
        };
    let tier_0_reward_per_match = tier_0_total_rewards.multiply_ratio(
        1u64,
        rewards_distribution_obj.tier_0.total_number_of_winners,
    );

    let tier_state = TierState {
        tier_0: RewardsClaimed {
            claimed: RewardsPerTierInfo {
                num_of_rewards_claimed: Uint128::zero(),
                reward_per_match: tier_0_reward_per_match,
            },
            num_of_rewards: rewards_distribution_obj.tier_0.total_number_of_winners,
        },
        tier_1: RewardsClaimed {
            claimed: RewardsPerTierInfo {
                num_of_rewards_claimed: Uint128::zero(),
                reward_per_match: tier_1_reward_per_match,
            },

            num_of_rewards: rewards_distribution_obj.tier_1.total_number_of_winners,
        },
        tier_2: RewardsClaimed {
            claimed: RewardsPerTierInfo {
                num_of_rewards_claimed: Uint128::zero(),
                reward_per_match: tier_2_reward_per_match,
            },

            num_of_rewards: rewards_distribution_obj.tier_2.total_number_of_winners,
        },
        tier_3: RewardsClaimed {
            claimed: RewardsPerTierInfo {
                num_of_rewards_claimed: Uint128::zero(),
                reward_per_match: tier_3_reward_per_match,
            },

            num_of_rewards: rewards_distribution_obj.tier_3.total_number_of_winners,
        },
        tier_4: RewardsClaimed {
            claimed: RewardsPerTierInfo {
                num_of_rewards_claimed: Uint128::zero(),
                reward_per_match: tier_4_reward_per_match,
            },

            num_of_rewards: rewards_distribution_obj.tier_4.total_number_of_winners,
        },
        tier_5: RewardsClaimed {
            claimed: RewardsPerTierInfo {
                num_of_rewards_claimed: Uint128::zero(),
                reward_per_match: tier_5_reward_per_match,
            },

            num_of_rewards: rewards_distribution_obj.tier_5.total_number_of_winners,
        },
    };
    Ok(tier_state)
}

////////////////////////////////////// Queries ///////////////////////////////////////

/// Returns ContractStatusResponse displaying the contract's status
///
/// # Arguments
///
/// * `Deps` - a reference to the contract's storage
fn query_contract_status(deps: Deps) -> StdResult<ContractStatusResponse> {
    let config = config_helper_read_only(deps.storage)?;
    let i = config.status;
    let item = match i {
        0 => ContractStatus::Normal,
        1 => ContractStatus::StopTransactions,
        2 => ContractStatus::StopAll,
        _ => return Err(StdError::generic_err("Wrong status")),
    };
    Ok(ContractStatusResponse { status: item })
}

/// Returns ContractConfigResponse displaying the contract's configuration
///
/// # Arguments
///
/// * `Deps` - a reference to the contract's storage
fn query_config(deps: Deps) -> StdResult<ContractConfigResponse> {
    let config = config_helper_read_only(deps.storage)?;

    let mut admins = Vec::new();
    for admin in config.admins {
        admins.push(admin);
    }

    let mut triggerers = Vec::new();
    for triggerer in config.triggerers {
        triggerers.push(triggerer);
    }

    let mut reviewers = Vec::new();
    for reviewer in config.reviewers {
        reviewers.push(reviewer);
    }

    let mut exp_contract = None;

    if let Some(exp_con) = config.exp_contract {
        exp_contract = Some(exp_con.contract);
    }

    Ok(ContractConfigResponse {
        admins,
        triggerers,
        reviewers,
        denom: config.denom,
        contract_address: config.contract_address,
        validators: config.validators,
        next_unbonding_batch_time: config.next_unbonding_batch_time,
        next_unbonding_batch_amount: config.next_unbonding_batch_amount,
        unbonding_batch_duration: config.unbonding_batch_duration,
        unbonding_duration: config.unbonding_duration,
        minimum_deposit_amount: config.minimum_deposit_amount,
        exp_contract,
    })
}

/// Returns RoundResponse displaying round's configuration
///
/// # Arguments
///
/// * `deps` - a reference to Extern containing all the contract's external dependencies
fn query_round(deps: Deps) -> StdResult<RoundResponse> {
    let round_obj = round_helper_read_only(deps.storage)?;

    Ok(RoundResponse {
        duration: round_obj.duration,
        start_time: round_obj.start_time,
        end_time: round_obj.end_time,
        rewards_distribution: round_obj.rewards_distribution,
        current_round_index: round_obj.current_round_index,
        ticket_price: round_obj.ticket_price,
        rewards_expiry_duration: round_obj.rewards_expiry_duration,
        admin_share: round_obj.admin_share,
        triggerer_share_percentage: round_obj.triggerer_share_percentage,
        unclaimed_distribution: round_obj.unclaimed_distribution,
    })
}

/// Returns PoolStateInfoResponse total amount delegated to this contract
///
/// # Arguments
///
/// * `deps` - a reference to Extern containing all the contract's external dependencies
fn query_pool_state_info(deps: Deps) -> StdResult<PoolStateInfoResponse> {
    let pool_state_obj = pool_state_helper_read_only(deps.storage)?;

    Ok(PoolStateInfoResponse {
        total_delegated: pool_state_obj.total_delegated,
        rewards_returned_to_contract: pool_state_obj.rewards_returned_to_contract,
        total_reserves: pool_state_obj.total_reserves,
        total_sponsored: pool_state_obj.total_sponsored,
    })
}

/// Returns PoolStateInfoResponse total amount delegated to this contract
///
/// # Arguments
///
/// * `deps` - a reference to Extern containing all the contract's external dependencies
fn query_reward_stats(deps: Deps) -> StdResult<RewardStatsResponse> {
    let lotter_config = round_helper_read_only(deps.storage)?;
    let round_index = if let Some(r_i) = lotter_config.current_round_index.checked_sub(1) {
        r_i
    } else {
        return Err(StdError::generic_err("Under-flow sub error"));
    };
    let reward_stats = reward_stats_for_nth_round_helper_read_only(deps.storage, round_index)?;

    Ok(RewardStatsResponse {
        distribution_per_tiers: reward_stats.distribution_per_tiers,
        ticket_price: reward_stats.ticket_price,
        winning_sequence: reward_stats.winning_sequence,
        rewards_expiration_date: reward_stats.rewards_expiration_date,
        total_rewards: reward_stats.total_rewards,
        total_claimed: reward_stats.total_claimed,
        total_exp: reward_stats.total_exp,
        total_exp_claimed: reward_stats.total_exp_claimed,
    })
}

/// Returns PoolStateLiquidityStatsResponse total liquidity provided this current round
///
/// # Arguments
///
/// * `deps` - a reference to Extern containing all the contract's external dependencies
fn query_pool_state_liquidity_stats(deps: Deps) -> StdResult<PoolStateLiquidityStatsResponse> {
    let round_obj = round_helper_read_only(deps.storage)?;
    let pool_state_liquidity_snapshot_obj: PoolLiqState =
        pool_state_liquidity_helper_read_only(deps.storage, round_obj.current_round_index)?;
    let liquidity: Uint128;
    if pool_state_liquidity_snapshot_obj.total_liquidity.is_none() {
        let pool_state_obj = pool_state_helper_read_only(deps.storage)?;
        liquidity = pool_state_obj.total_delegated;
    } else {
        liquidity = pool_state_liquidity_snapshot_obj.total_liquidity.unwrap();
    }

    Ok(PoolStateLiquidityStatsResponse {
        total_liquidity: liquidity,
    })
}

/// Returns PoolStateLiquidityStatsResponse total liquidity provided for a specific past round
///
/// # Arguments
///
/// * `deps` - a reference to Extern containing all the contract's external dependencies
fn query_pool_state_liquidity_stats_specific(
    deps: Deps,
    round_index: u64,
) -> StdResult<PoolStateLiquidityStatsResponse> {
    let pool_state_liquidity_snapshot_obj: PoolLiqState =
        pool_state_liquidity_helper_read_only(deps.storage, round_index)?;
    let liquidity: Uint128;
    if pool_state_liquidity_snapshot_obj.total_liquidity.is_none() {
        let pool_state_obj = pool_state_helper_read_only(deps.storage)?;
        liquidity = pool_state_obj.total_delegated;
    } else {
        liquidity = pool_state_liquidity_snapshot_obj.total_liquidity.unwrap();
    }

    Ok(PoolStateLiquidityStatsResponse {
        total_liquidity: liquidity,
    })
}

/// Returns TotalRewardsResponse displaying current rewards
///
/// # Arguments
///
/// * `deps` - a reference to Extern containing all the contract's external dependencies
/// * `env` - Env of contract's environment
fn query_current_rewards(deps: Deps, env: Env) -> StdResult<CurrentRewardsResponse> {
    let pool_state_obj = pool_state_helper_read_only(deps.storage)?;
    let config_obj = config_helper_read_only(deps.storage)?;

    let rewards_obj = get_rewards(deps, &env.contract.address, &config_obj).unwrap();
    let mut total_rewards: Uint128 = pool_state_obj.rewards_returned_to_contract;
    for reward in rewards_obj {
        total_rewards.add_assign(reward.reward);
    }

    Ok(CurrentRewardsResponse {
        rewards: total_rewards,
    })
}

/// Returns SponsorMessageRequestResponse all message request from the users
///
/// # Arguments
///
/// * `deps` - a reference to Extern containing all the contract's external dependencies
/// * `address` address of the user querying this
/// * `start_page` starting page
/// * `page_size` page size
fn query_sponsor_message_req_check(
    deps: Deps,
    start_page: Option<u32>,
    page_size: Option<u32>,
) -> StdResult<SponsorMessageRequestResponse> {
    let binding = SPONSOR_DISPLAY_REQ_STORE;
    let mut vec = binding.paging(
        deps.storage,
        start_page.unwrap_or(0),
        page_size.unwrap_or(5),
    )?;

    let starting_offset = start_page.unwrap_or(0) * page_size.unwrap_or(5);
    for (i, item) in vec.iter_mut().enumerate() {
        item.deque_store_index = Some((i as u32) + (starting_offset));
    }

    let len = binding.get_len(deps.storage)?;

    Ok(SponsorMessageRequestResponse { vec, len })
}

/// Returns SponsorsResponse information of all the sponsors in the global sponsors list
///
/// # Arguments
///
/// * `deps` - a reference to Extern containing all the contract's external dependencies
/// * `address` address of the user querying this
/// * `start_page` starting page
/// * `page_size` page size
fn query_sponsors(
    deps: Deps,
    start_page: Option<u32>,
    page_size: Option<u32>,
) -> StdResult<SponsorsResponse> {
    let sponsor_stats = sponsor_stats_helper_read_only(deps.storage)?;
    let mut vec = vec![];
    let start_offset = start_page.unwrap_or(0) * page_size.unwrap_or(5);
    let end_offset =
        if sponsor_stats.offset < ((start_page.unwrap_or(0) + 1) * page_size.unwrap_or(5)) {
            sponsor_stats.offset
        } else {
            (start_page.unwrap_or(0) + 1) * page_size.unwrap_or(5)
        };

    let length = end_offset - start_offset;
    let mut index = 0;
    let mut offset = 0;
    for _ in 0..length {
        loop {
            if index > sponsor_stats.offset {
                break;
            }

            if offset < start_offset {
                let does_exists = SPONSOR_LIST_STORE.has(deps.storage, index);

                if does_exists {
                    offset += 1;
                }
                index += 1;
            } else {
                let value = SPONSOR_LIST_STORE.load(deps.storage, index);
                index += 1;

                if value.is_ok() {
                    offset += 1;

                    let sponsor_info_obj =
                        sponsor_info_helper_read_only(deps.storage, &value.unwrap())?;
                    vec.push(SponsorDisplayInfo {
                        amount_sponsored: sponsor_info_obj.amount_sponsored,
                        title: sponsor_info_obj.title,
                        message: sponsor_info_obj.message,
                        addr_list_index: sponsor_info_obj.addr_list_index,
                    });
                    break;
                }
            }
        }
    }

    let len = if let Some(len) = sponsor_stats
        .offset
        .checked_sub(sponsor_stats.empty_slots.len() as u32)
    {
        len
    } else {
        return Err(StdError::generic_err("Under-flow sub error"));
    };

    Ok(SponsorsResponse { vec, len })
}

fn authenticated_queries(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let (addresses, key) = msg.get_validation_params();

    for address in addresses {
        let addr = deps.api.addr_validate(address)?;

        let expected_key = read_viewing_key(deps.storage, &addr);

        if expected_key.is_none() {
            // Checking the key will take significant time. We don't want to exit immediately if it isn't set
            // in a way which will allow to time the command and determine if a viewing key doesn't exist
            key.check_viewing_key(&[0u8; VIEWING_KEY_SIZE]);
        } else if key.check_viewing_key(expected_key.unwrap().as_slice()) {
            return match msg {
                // Base
                QueryMsg::Delegated { address, .. } => to_binary(&query_delegated(deps, address)?),
                QueryMsg::Withdrawable { address, .. } => {
                    to_binary(&query_withdrawable(deps, address, env)?)
                }
                QueryMsg::Unbondings { address, .. } => {
                    to_binary(&query_unbondings(deps, address)?)
                }
                QueryMsg::Liquidity {
                    address,
                    round_index,
                    ..
                } => to_binary(&query_liquidity(deps, address, round_index)?),
                QueryMsg::SponsorInfo { address, .. } => {
                    to_binary(&query_sponsor_info(deps, address)?)
                }
                QueryMsg::SponsorUnbondings { address, .. } => {
                    to_binary(&query_sponsor_unbondings(deps, address)?)
                }
                QueryMsg::SponsorWithdrawable { address, .. } => {
                    to_binary(&query_sponsor_withdrawable(deps, address, env)?)
                }
                QueryMsg::Records {
                    address,
                    page_size,
                    start_page,
                    ..
                } => to_binary(&query_records(deps, address, start_page, page_size)?),

                _ => {
                    return Err(StdError::generic_err(format!(
                        "This query type does not require authentication"
                    )));
                }
            };
        }
    }

    Ok(to_binary(&ViewingKeyErrorResponse {
        msg: "Wrong viewing key for this address or viewing key not set".to_string(),
    })?)
}

/// Returns Binary from validating a permit and then using its creator's address when
/// performing the specified query
///
/// # Arguments
///
/// * `deps` - a reference to Extern containing all the contract's external dependencies
/// * `permit` - the permit used to authentic the query
/// * `query` - the query to perform
/// * `env` - Env of contract's environment
fn permit_queries(
    deps: Deps,
    permit: Permit<GalacticPoolsPermissions>,
    query: QueryWithPermit,
    env: Env,
) -> StdResult<Binary> {
    let config = config_helper_read_only(deps.storage)?;

    let token_address = config.contract_address.to_string();

    //Checking if token is included
    let account = validate(deps, PREFIX_REVOKED_PERMITS, &permit, token_address, None)?;

    // permit validated, process query
    return match query {
        QueryWithPermit::Delegated {} => {
            //Checking permissions
            if !(permit.check_permission(&GalacticPoolsPermissions::Delegated)
                || permit.check_permission(&GalacticPoolsPermissions::Owner))
            {
                return Err(StdError::generic_err(format!(
                    "Owner or Delegated permission is required for queries, got permissions {:?}",
                    permit.params.permissions
                )));
            }

            to_binary(&query_delegated(deps, account)?)
        }
        QueryWithPermit::UserInfo {} => {
            //Checking permissions
            if !(permit.check_permission(&GalacticPoolsPermissions::UserInfo)
                || permit.check_permission(&GalacticPoolsPermissions::Owner))
            {
                return Err(StdError::generic_err(format!(
                    "Owner or Delegated permission is required for queries, got permissions {:?}",
                    permit.params.permissions
                )));
            }

            to_binary(&query_user_info(deps, account)?)
        }
        QueryWithPermit::SponsorInfo {} => {
            //Checking permissions
            if !(permit.check_permission(&GalacticPoolsPermissions::SponsorInfo)
                || permit.check_permission(&GalacticPoolsPermissions::Owner))
            {
                return Err(StdError::generic_err(format!(
                    "Owner or Delegated permission is required for queries, got permissions {:?}",
                    permit.params.permissions
                )));
            }

            to_binary(&query_sponsor_info(deps, account)?)
        }
        QueryWithPermit::SponsorUnbondings {} => {
            if !(permit.check_permission(&GalacticPoolsPermissions::SponsorUnbondings)
                || permit.check_permission(&GalacticPoolsPermissions::Owner))
            {
                return Err(StdError::generic_err(format!(
                    "Owner or Unbondings permission is required for queries, got permissions {:?}",
                    permit.params.permissions
                )));
            }
            to_binary(&query_sponsor_unbondings(deps, account)?)
        }
        QueryWithPermit::SponsorWithdrawable {} => {
            if !(permit.check_permission(&GalacticPoolsPermissions::SponsorWithdrawable)
                || permit.check_permission(&GalacticPoolsPermissions::Owner))
            {
                return Err(StdError::generic_err(format!(
                    "Owner or Unbondings permission is required for queries, got permissions {:?}",
                    permit.params.permissions
                )));
            }
            to_binary(&query_sponsor_withdrawable(deps, account, env)?)
        }
        QueryWithPermit::Liquidity { round_index } => {
            //Checking permissions
            if !(permit.check_permission(&GalacticPoolsPermissions::Liquidity)
                || permit.check_permission(&GalacticPoolsPermissions::Owner))
            {
                return Err(StdError::generic_err(format!(
                    "Owner or Liquidity permission is required for queries, got permissions {:?}",
                    permit.params.permissions
                )));
            }

            to_binary(&query_liquidity(deps, account, round_index)?)
        }
        QueryWithPermit::Withdrawable {} => {
            if !(permit.check_permission(&GalacticPoolsPermissions::Withdrawable)
                || permit.check_permission(&GalacticPoolsPermissions::Owner))
            {
                return Err(StdError::generic_err(format!(
                    "Owner or Withdrawable permission is required for queries, got permissions {:?}",
                    permit.params.permissions
                )));
            }
            to_binary(&query_withdrawable(deps, account, env)?)
        }
        QueryWithPermit::Unbondings {} => {
            if !(permit.check_permission(&GalacticPoolsPermissions::Unbondings)
                || permit.check_permission(&GalacticPoolsPermissions::Owner))
            {
                return Err(StdError::generic_err(format!(
                    "Owner or Unbondings permission is required for queries, got permissions {:?}",
                    permit.params.permissions
                )));
            }
            to_binary(&query_unbondings(deps, account)?)
        }
        QueryWithPermit::Records {
            page_size,
            start_page,
        } => {
            if !(permit.check_permission(&GalacticPoolsPermissions::Records)
                || permit.check_permission(&GalacticPoolsPermissions::Owner))
            {
                return Err(StdError::generic_err(format!(
                    "Owner or Records permission is required for queries, got permissions {:?}",
                    permit.params.permissions
                )));
            }

            to_binary(&query_records(deps, account, start_page, page_size)?)
        }
        _ => return Err(StdError::generic_err(format!("There is no such query"))),
    };
}

/// Returns QueryResult displaying total amount delegated by th user
///
/// # Arguments
///
/// * `deps` - a reference to Extern containing all the contract's external dependencies
/// * `address` address of the user querying this
fn query_delegated(deps: Deps, address: String) -> StdResult<DelegatedResponse> {
    let addr = deps.api.addr_validate(address.as_str())?;
    let user_info = user_info_helper_read_only(deps.storage, &addr)?;

    Ok(DelegatedResponse {
        amount: user_info.amount_delegated,
    })
}

/// Returns QueryResult displaying total amount liquidity provided by the user
///
/// # Arguments
///
/// * `deps` - a reference to Extern containing all the contract's external dependencies
/// * `address` address of the user querying this
fn query_liquidity(deps: Deps, address: String, round_index: u64) -> StdResult<LiquidityResponse> {
    let addr = deps.api.addr_validate(address.as_str())?;

    let user_liq_obj =
        user_liquidity_snapshot_stats_helper_read_only(deps.storage, round_index, &addr)?;
    let user_info_obj: UserInfo = user_info_helper_read_only(deps.storage, &addr)?;
    let reward_stats = reward_stats_for_nth_round_helper_read_only(deps.storage, round_index)?;
    let pool_state_liquidity_snapshot_obj: PoolLiqState =
        pool_state_liquidity_helper_read_only(deps.storage, round_index)?;

    //Calculating User Liquidity to generate n tickets for the round
    let liquidity_current_round;
    let round_obj;
    let mut legacy_bal = Uint128::zero();
    let total_liq;
    let ticket_price;
    let mut total_tickets = Uint128::zero();
    let mut user_tickets = Uint128::zero();
    let mut tickets_used = Uint128::zero();

    //Ticket Price
    if reward_stats.ticket_price.is_zero() {
        round_obj = round_helper_read_only(deps.storage)?;
        ticket_price = round_obj.ticket_price;
    } else {
        ticket_price = reward_stats.ticket_price;
    }

    // Total Liquidity
    if pool_state_liquidity_snapshot_obj.total_liquidity.is_some() {
        total_liq = pool_state_liquidity_snapshot_obj.total_liquidity.unwrap();
    } else {
        let pool_stats = pool_state_helper_read_only(deps.storage)?;
        total_liq = pool_stats.total_delegated;
    }

    //Total Tickets
    if total_liq > Uint128::zero() {
        total_tickets = total_liq.multiply_ratio(1u128, ticket_price)
    }

    //User Liquidity
    if user_liq_obj.liquidity.is_some() {
        liquidity_current_round = user_liq_obj.liquidity.unwrap();
    } else {
        let mut finding_liq_round: u64 = if let Some(r_i) = round_index.checked_sub(1) {
            r_i
        } else {
            return Err(StdError::generic_err("Under-flow sub error"));
        };

        let start = if user_info_obj.last_claim_rewards_round.is_some() {
            user_info_obj.last_claim_rewards_round.unwrap()
        } else {
            if user_info_obj.starting_round.is_some() {
                user_info_obj.starting_round.unwrap()
            } else {
                return Ok(LiquidityResponse {
                    total_liq,
                    total_tickets,
                    ticket_price,
                    user_liq: Uint128::default(),
                    user_tickets: Uint128::default(),
                    tickets_used: Uint128::default(),
                    expiry_date: reward_stats.rewards_expiration_date,
                    total_rewards: reward_stats.total_rewards,
                    unclaimed_rewards: if let Ok(u_r) = reward_stats
                        .total_rewards
                        .checked_sub(reward_stats.total_claimed)
                    {
                        u_r
                    } else {
                        return Err(StdError::generic_err("Under-flow sub error"));
                    },
                });
            }
        };
        while finding_liq_round >= start {
            let user_liq_obj_prev_round = user_liquidity_snapshot_stats_helper_read_only(
                deps.storage,
                finding_liq_round,
                &addr,
            )?;
            if user_liq_obj_prev_round.amount_delegated.is_some() {
                legacy_bal = user_liq_obj_prev_round.amount_delegated.unwrap();
                break;
            } else {
                finding_liq_round = if let Some(f_liq) = finding_liq_round.checked_sub(1) {
                    f_liq
                } else {
                    return Err(StdError::generic_err("Under-flow sub error"));
                };
            }
        }
        liquidity_current_round = legacy_bal;
    }

    if liquidity_current_round > Uint128::zero() {
        user_tickets = liquidity_current_round.multiply_ratio(1u128, ticket_price)
    }

    if user_liq_obj.tickets_used.is_some() {
        tickets_used = user_liq_obj.tickets_used.unwrap()
    }

    Ok(LiquidityResponse {
        total_liq,
        total_tickets,
        ticket_price,
        user_liq: liquidity_current_round,
        user_tickets,
        tickets_used,
        expiry_date: reward_stats.rewards_expiration_date,
        total_rewards: reward_stats.total_rewards,
        unclaimed_rewards: if let Ok(u_r) = reward_stats
            .total_rewards
            .checked_sub(reward_stats.total_claimed)
        {
            u_r
        } else {
            return Err(StdError::generic_err("Under-flow sub error"));
        },
    })
}

/// Returns QueryResult displaying total amount liquidity provided by the user
///
/// # Arguments
///
/// * `deps` - a reference to Extern containing all the contract's external dependencies
/// * `address` address of the user querying this
fn query_user_info(deps: Deps, address: String) -> StdResult<UserInfoResponse> {
    let addr = deps.api.addr_validate(address.as_str())?;
    let user_info_obj: UserInfo = user_info_helper_read_only(deps.storage, &addr)?;

    Ok(UserInfoResponse {
        amount_delegated: user_info_obj.amount_delegated,
        amount_unbonding: user_info_obj.amount_unbonding,
        starting_round: user_info_obj.starting_round,
        total_won: user_info_obj.total_won,
        last_claim_rewards_round: user_info_obj.last_claim_rewards_round,
    })
}

/// Returns AmountWithdrawablelResponse displaying the amount that can be withdrawn this instant
///
/// # Arguments
///
/// * `deps` - a reference to Extern containing all the contract's external dependencies
/// * `address` address of the user querying this
/// * `env` - Env of contract's environment
fn query_withdrawable(deps: Deps, address: String, env: Env) -> StdResult<WithdrawablelResponse> {
    let user_info_obj =
        user_info_helper_read_only(deps.storage, &deps.api.addr_validate(address.as_str())?)?;

    let mut amount_withdrawable = user_info_obj.amount_withdrawable;

    for i in 0..user_info_obj.unbonding_batches.len() {
        let unbonding_batch_index = user_info_obj.unbonding_batches[i];
        let unbonding_batch_obj =
            unbonding_batch_helper_read_only(deps.storage, unbonding_batch_index)?;
        let unbonding_amount = user_unbond_helper_read_only(
            deps.storage,
            unbonding_batch_index,
            &deps.api.addr_validate(address.as_str())?,
        )?;

        // already unbonding
        if unbonding_batch_obj.unbonding_time.is_some() {
            if env.block.time.seconds() >= unbonding_batch_obj.unbonding_time.unwrap() {
                amount_withdrawable.add_assign(unbonding_amount)
            }
        }
    }

    Ok(WithdrawablelResponse {
        amount: amount_withdrawable,
    })
}

/// Returns UnbondingsResponse displaying total unbondings happening at the moment
///
/// # Arguments
/// * `deps` - a reference to Extern containing all the contract's external dependencies
/// * `address` address of the user querying this
/// * `start_page` page at the start of the list
/// * `page_size` number of transactions on each page
fn query_unbondings(deps: Deps, address: String) -> StdResult<UnbondingsResponse> {
    let user_info_obj =
        user_info_helper_read_only(deps.storage, &deps.api.addr_validate(address.as_str())?)?;

    let config = config_helper_read_only(deps.storage)?;

    let mut vec = vec![];

    for i in 0..user_info_obj.unbonding_batches.len() {
        let unbonding_batch_index = user_info_obj.unbonding_batches[i];
        let unbonding_batch_obj =
            unbonding_batch_helper_read_only(deps.storage, unbonding_batch_index)?;
        let unbonding_amount = user_unbond_helper_read_only(
            deps.storage,
            unbonding_batch_index,
            &deps.api.addr_validate(address.as_str())?,
        )?;

        //1 already unbonding
        if unbonding_batch_obj.unbonding_time.is_some() {
            vec.push(RequestWithdrawQueryResponse {
                amount: unbonding_amount,
                batch_index: unbonding_batch_index,
                next_batch_unbonding_time: None,
                unbonding_time: unbonding_batch_obj.unbonding_time,
            })
        }
        // 2 not unbonded
        else {
            if unbonding_batch_index == config.next_unbonding_batch_index {
                vec.push(RequestWithdrawQueryResponse {
                    amount: unbonding_amount,
                    batch_index: unbonding_batch_index,
                    next_batch_unbonding_time: Some(config.next_unbonding_batch_time),
                    unbonding_time: None,
                })
            }
        }
    }

    let len = user_info_obj.unbonding_batches.len() as u32;

    Ok(UnbondingsResponse { vec, len })
}

/// Returns RewardsLogResponse containing information of all the rewards claimed by the user
///
/// # Arguments
///
/// * `deps` - a reference to Extern containing all the contract's external dependencies
/// * `address` address of the user querying this
/// * `start_page` page at the start of the list
/// * `page_size` number of transactions on each page
fn query_records(
    deps: Deps,
    address: String,
    start_page: Option<u32>,
    page_size: Option<u32>,
) -> StdResult<RecordsResponse> {
    let vec = user_records_helper_read_only(
        deps.storage,
        &deps.api.addr_validate(address.as_str())?,
        start_page,
        page_size,
    )?;
    let user_store = USER_REWARDS_LOG_STORE.add_suffix(address.as_str());
    let len = user_store.get_len(deps.storage)?;

    Ok(RecordsResponse { vec, len })
}

/// Returns QueryResult displaying sponsor info
///
/// # Arguments
///
/// * `deps` - a reference to Extern containing all the contract's external dependencies
/// * `address` address of the user querying this
fn query_sponsor_info(deps: Deps, address: String) -> StdResult<SponsorInfoResponse> {
    let sponsor_info_obj: SponsorInfo =
        sponsor_info_helper_read_only(deps.storage, &deps.api.addr_validate(address.as_str())?)?;

    Ok(SponsorInfoResponse {
        amount_sponsored: sponsor_info_obj.amount_sponsored,
        amount_withdrawable: sponsor_info_obj.amount_withdrawable,
        amount_unbonding: sponsor_info_obj.amount_unbonding,
        title: sponsor_info_obj.title,
        message: sponsor_info_obj.message,
        addr_list_index: sponsor_info_obj.addr_list_index,
        unbonding_batches: sponsor_info_obj.unbonding_batches,
        has_requested: sponsor_info_obj.has_requested, // req_list_index: sponsor_info_obj.req_list_index,
    })
}

/// Returns AmountWithdrawablelResponse displaying the amount that can be withdrawn this instant
///
/// # Arguments
///
/// * `deps` - a reference to Extern containing all the contract's external dependencies
/// * `address` address of the user querying this
/// * `env` - Env of contract's environment
fn query_sponsor_withdrawable(
    deps: Deps,
    address: String,
    env: Env,
) -> StdResult<WithdrawablelResponse> {
    let sponsor_info_obj =
        sponsor_info_helper_read_only(deps.storage, &deps.api.addr_validate(address.as_str())?)?;

    let mut amount_withdrawable = sponsor_info_obj.amount_withdrawable;

    for i in 0..sponsor_info_obj.unbonding_batches.len() {
        let unbonding_batch_index = sponsor_info_obj.unbonding_batches[i];
        let unbonding_batch_obj =
            unbonding_batch_helper_read_only(deps.storage, unbonding_batch_index)?;
        let unbonding_amount = sponsor_unbond_helper_read_only(
            deps.storage,
            unbonding_batch_index,
            &deps.api.addr_validate(address.as_str())?,
        )?;

        // already unbonding
        if unbonding_batch_obj.unbonding_time.is_some() {
            if env.block.time.seconds() >= unbonding_batch_obj.unbonding_time.unwrap() {
                amount_withdrawable.add_assign(unbonding_amount)
            }
        }
    }

    Ok(WithdrawablelResponse {
        amount: amount_withdrawable,
    })
}

/// Returns UnbondingsResponse displaying total unbondings happening at the moment
///
/// # Arguments
/// * `deps` - a reference to Extern containing all the contract's external dependencies
/// * `address` address of the user querying this
/// * `start_page` page at the start of the list
/// * `page_size` number of transactions on each page
fn query_sponsor_unbondings(deps: Deps, address: String) -> StdResult<UnbondingsResponse> {
    let sponsor_info_obj =
        sponsor_info_helper_read_only(deps.storage, &deps.api.addr_validate(address.as_str())?)?;

    let config = config_helper_read_only(deps.storage)?;

    let mut vec = vec![];

    for i in 0..sponsor_info_obj.unbonding_batches.len() {
        let unbonding_batch_index = sponsor_info_obj.unbonding_batches[i];
        let unbonding_batch_obj =
            unbonding_batch_helper_read_only(deps.storage, unbonding_batch_index)?;
        let unbonding_amount = sponsor_unbond_helper_read_only(
            deps.storage,
            unbonding_batch_index,
            &deps.api.addr_validate(address.as_str())?,
        )?;

        //1) Already unbonding
        if unbonding_batch_obj.unbonding_time.is_some() {
            vec.push(RequestWithdrawQueryResponse {
                amount: unbonding_amount,
                batch_index: unbonding_batch_index,
                next_batch_unbonding_time: None,
                unbonding_time: unbonding_batch_obj.unbonding_time,
            })
        }
        //2) Not unbonded
        else {
            if unbonding_batch_index == config.next_unbonding_batch_index {
                vec.push(RequestWithdrawQueryResponse {
                    amount: unbonding_amount,
                    batch_index: unbonding_batch_index,
                    next_batch_unbonding_time: Some(config.next_unbonding_batch_time),
                    unbonding_time: None,
                })
            }
        }
    }

    let len = sponsor_info_obj.unbonding_batches.len() as u32;

    Ok(UnbondingsResponse { vec, len })
}
