use shade_protocol::{
    c_std::{
        entry_point,
        from_binary,
        to_binary,
        Addr,
        Attribute,
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
        Uint256,
    },
    liquidity_book::staking::{EpochInfo, StakerInfo, StakerLiquiditySnapshot},
    query_auth::QueryPermit,
    secret_storage_plus::ItemStorage,
    snip20::helpers::{register_receive, set_viewing_key_msg, token_info},
    swap::staking::{InstantiateMsg, RewardTokenInfo, RewardTokenSet, State},
    Contract,
    BLOCK_SIZE,
};

use crate::state::{
    EPOCH_STORE,
    REWARD_TOKENS,
    REWARD_TOKEN_INFO,
    STAKERS,
    STAKERS_LIQUIDITY_SNAPSHOT,
    TOTAL_LIQUIDITY,
    TOTAL_LIQUIDITY_SNAPSHOT,
};

pub fn create_reward_token(
    storage: &mut dyn Storage,
    now_epoch_index: u64,
    token: &Contract,
    epoch_emission_amount: Uint128,
    valid_to_epoch: u64,
    decimals: u8,
) -> StdResult<Vec<RewardTokenInfo>> {
    let mut reward_configs = match REWARD_TOKEN_INFO.may_load(storage, &token.address)? {
        Some(rewards) => rewards,
        None => vec![],
    };
    let info = init_from_daily_rewards(
        now_epoch_index,
        token,
        decimals,
        epoch_emission_amount,
        valid_to_epoch,
    )?;
    match REWARD_TOKENS.may_load(storage)? {
        Some(mut tokens) => {
            tokens.insert(&info.token.address);
            REWARD_TOKENS.save(storage, &tokens)?;
        }
        None => REWARD_TOKENS.save(storage, &RewardTokenSet(vec![info.token.address.clone()]))?,
    };
    reward_configs.push(info);
    REWARD_TOKEN_INFO.save(storage, &token.address, &reward_configs)?;
    Ok(reward_configs)
}

pub fn init_from_daily_rewards(
    now: u64,
    token: &Contract,
    decimals: u8,
    epoch_emission_amount: Uint128,
    valid_to: u64,
) -> StdResult<RewardTokenInfo> {
    Ok(RewardTokenInfo {
        token: token.clone(),
        decimals,
        reward_per_epoch: epoch_emission_amount.into(),
        valid_to,
    })
}

pub fn store_empty_reward_set(storage: &mut dyn Storage) -> StdResult<()> {
    match REWARD_TOKENS.may_load(storage)? {
        Some(_) => Err(StdError::generic_err("Reward token storage already exists")),
        None => REWARD_TOKENS.save(storage, &RewardTokenSet(vec![])),
    }
}

pub fn require_lp_token(state: &State, addr: &Addr) -> StdResult<()> {
    if state.lp_token.address.eq(addr) {
        return Ok(());
    }
    Err(StdError::generic_err(format!(
        "Must stake the LP token {}. Attempted to stake {addr}.",
        state.lp_token.address
    )))
}

pub fn staker_init_checker(
    storage: &mut dyn Storage,
    state: &State,
    staker_addr: &Addr,
) -> StdResult<()> {
    let staker = STAKERS.load(storage, &staker_addr);

    if staker.is_err() {
        STAKERS.save(storage, &staker_addr, &StakerInfo {
            starting_round: Some(state.epoch_index),
            total_rewards_earned: Uint128::default(),
            last_claim_rewards_round: None,
        })?;
    }
    Ok(())
}

pub fn assert_lb_pair(state: &State, info: MessageInfo) -> StdResult<()> {
    if info.sender != state.lb_pair {
        Err(StdError::generic_err(format!(
            "Onlt accessible by  lb-pair {}",
            state.lb_pair
        )))
    } else {
        Ok(())
    }
}

pub fn check_if_claimable(starting_round: Option<u64>, current_round_index: u64) -> StdResult<()> {
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

pub fn finding_user_liquidity(
    storage: &mut dyn Storage,
    info: &MessageInfo,
    staker_info: &StakerInfo,
    epoch_index: u64,
    bin_id: u32,
) -> StdResult<Uint256> {
    let mut legacy_bal: Uint256 = Uint256::zero();
    let mut staker_liq_snap = STAKERS_LIQUIDITY_SNAPSHOT
        .load(storage, (&info.sender, epoch_index, bin_id))
        .unwrap_or_default();

    if !staker_liq_snap.liquidity.is_zero() {
        Ok(staker_liq_snap.liquidity)
    } else {
        let mut finding_liq_round: u64 = if let Some(rn) = epoch_index.checked_sub(1) {
            rn
        } else {
            return Err(StdError::generic_err("Under-flow sub error 3"));
        };
        let start = if staker_info.last_claim_rewards_round.is_some() {
            staker_info.last_claim_rewards_round.unwrap()
        } else {
            staker_info.starting_round.unwrap()
        };
        while finding_liq_round >= start {
            let staker_liq_snap_prev_round = STAKERS_LIQUIDITY_SNAPSHOT
                .load(storage, (&info.sender, finding_liq_round, bin_id))
                .unwrap_or_default();
            if !staker_liq_snap_prev_round.amount_delegated.is_zero() {
                legacy_bal = staker_liq_snap_prev_round.amount_delegated;
                break;
            } else {
                finding_liq_round = if let Some(f_liq_round) = finding_liq_round.checked_sub(1) {
                    f_liq_round
                } else {
                    return Err(StdError::generic_err("Under-flow sub error 4"));
                };
            }
        }

        staker_liq_snap.liquidity = legacy_bal;
        staker_liq_snap.amount_delegated = legacy_bal;
        // user_liquidity_snapshot_stats_helper_store(storage, round_index, sender, staker_liq_snap)?;
        STAKERS_LIQUIDITY_SNAPSHOT.save(
            storage,
            (&info.sender, epoch_index, bin_id),
            &staker_liq_snap,
        )?;

        Ok(legacy_bal)
    }
}

pub fn finding_total_liquidity(
    storage: &mut dyn Storage,
    epoch_index: u64,
    bin_id: u32,
) -> StdResult<Uint256> {
    let mut legacy_bal: Uint256 = Uint256::zero();
    let mut total_liq_snap = TOTAL_LIQUIDITY_SNAPSHOT
        .load(storage, (epoch_index, bin_id))
        .unwrap_or_default();
    let total_liq = TOTAL_LIQUIDITY.load(storage, bin_id).unwrap_or_default();

    if !total_liq_snap.liquidity.is_zero() {
        Ok(total_liq_snap.liquidity)
    } else {
        let mut finding_liq_round: u64 = if let Some(rn) = epoch_index.checked_sub(1) {
            rn
        } else {
            return Err(StdError::generic_err("Under-flow sub error 3"));
        };
        let start = if total_liq.last_deposited.is_some() {
            total_liq.last_deposited.unwrap()
        } else {
            return Ok(legacy_bal);
        };
        while finding_liq_round >= start {
            let staker_liq_snap_prev_round = TOTAL_LIQUIDITY_SNAPSHOT
                .load(storage, (finding_liq_round, bin_id))
                .unwrap_or_default();
            if !staker_liq_snap_prev_round.amount_delegated.is_zero() {
                legacy_bal = staker_liq_snap_prev_round.amount_delegated;
                break;
            } else {
                finding_liq_round = if let Some(f_liq_round) = finding_liq_round.checked_sub(1) {
                    f_liq_round
                } else {
                    return Err(StdError::generic_err("Under-flow sub error 4"));
                };
            }
        }

        total_liq_snap.liquidity = legacy_bal;
        total_liq_snap.amount_delegated = legacy_bal;
        // user_liquidity_snapshot_stats_helper_store(storage, round_index, sender, staker_liq_snap)?;
        TOTAL_LIQUIDITY_SNAPSHOT.save(storage, (epoch_index, bin_id), &total_liq_snap)?;

        Ok(legacy_bal)
    }
}
