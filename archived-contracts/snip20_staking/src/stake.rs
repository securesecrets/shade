use crate::{
    contract::check_if_admin,
    msg::{HandleAnswer, ResponseStatus::Success},
    state::{Balances, Config, ReadonlyConfig},
    state_staking::{
        DailyUnbondingQueue,
        TotalShares,
        TotalTokens,
        TotalUnbonding,
        UnbondingQueue,
        UnsentStakedTokens,
        UserCooldown,
        UserShares,
    },
    transaction_history::{
        store_add_reward,
        store_claim_reward,
        store_claim_unbond,
        store_fund_unbond,
        store_stake,
        store_unbond,
    },
};
use shade_protocol::c_std::{Uint128, Uint256};
use shade_protocol::c_std::{
    from_binary,
    to_binary,
    Api,
    Binary,
    CanonicalAddr,
    Env,
    DepsMut,
    Response,
    Addr,
    Querier,
    StdError,
    StdResult,
    Storage,
};
use shade_protocol::snip20::helpers::send_msg;
use shade_protocol::{
    contract_interfaces::staking::snip20_staking::{
        stake::{DailyUnbonding, StakeConfig, Unbonding, VecQueue},
        ReceiveType,
    },
    utils::storage::default::{BucketStorage, SingletonStorage},
};
use std::convert::TryInto;

//TODO: set errors

pub fn try_update_stake_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    unbond_time: Option<u64>,
    disable_treasury: bool,
    treasury: Option<Addr>,
) -> StdResult<Response> {
    let config = Config::from_storage(deps.storage);

    check_if_admin(&config, &info.sender)?;

    let mut stake_config = StakeConfig::load(deps.storage)?;

    if let Some(unbond_time) = unbond_time {
        stake_config.unbond_time = unbond_time;
    }

    let mut messages = vec![];

    if disable_treasury {
        stake_config.treasury = None;
    } else if let Some(treasury) = treasury {
        stake_config.treasury = Some(treasury.clone());

        let unsent_tokens = UnsentStakedTokens::load(deps.storage)?.0;
        if unsent_tokens != Uint128::zero() {
            messages.push(send_msg(
                treasury,
                unsent_tokens.into(),
                None,
                None,
                None,
                258,
                stake_config.staked_token.code_hash.clone(),
                stake_config.staked_token.address.clone(),
            )?);
            UnsentStakedTokens(Uint128::zero()).save(deps.storage)?;
        }
    }

    stake_config.save(deps.storage)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::UpdateStakeConfig {
            status: Success,
        })?))
}

const DAY: u64 = 86400; //60 * 60 * 24

///
/// Rounds down a date to the nearest day
///
fn round_date(date: u64) -> u64 {
    date - (date % DAY)
}

///
/// Updates total states to reflect balance changes
///
fn add_balance(
    storage: &mut dyn Storage,
    stake_config: &StakeConfig,
    sender: &Addr,
    sender_canon: &CanonicalAddr,
    amount: Uint128,
) -> StdResult<()> {
    // Check if user account exists
    let mut user_shares = UserShares::may_load(storage, sender.as_str().as_bytes())?
        .unwrap_or(UserShares(Uint256::zero()));

    // Update user staked tokens
    let mut balances = Balances::from_storage(storage);
    let mut account_balance = balances.balance(sender_canon);
    if let Some(new_balance) = account_balance.checked_add(amount.u128()) {
        account_balance = new_balance;
    } else {
        return Err(StdError::generic_err(
            "This mint attempt would increase the account's balance above the supported maximum",
        ));
    }
    balances.set_account_balance(sender_canon, account_balance);

    // Get total supplied tokens
    let mut total_shares = TotalShares::load(storage)?;
    let total_tokens = TotalTokens::load(storage)?;

    // Update total staked
    // We do this before reaching shares to get overflows out of the way
    match total_tokens.0.checked_add(amount) {
        Ok(total_staked) => TotalTokens(total_staked).save(storage)?,
        Err(_) => return Err(StdError::generic_err("Total staked tokens overflow")),
    };

    let supply = ReadonlyConfig::from_storage(storage).total_supply();
    Config::from_storage(storage).set_total_supply(supply + amount.u128());

    // Calculate shares per token supplied
    let shares = shares_per_token(stake_config, &amount, &total_tokens.0, &total_shares.0)?;

    // Update total shares
    match total_shares.0.checked_add(shares) {
        Ok(total_added_shares) => total_shares = TotalShares(total_added_shares),
        Err(_) => return Err(StdError::generic_err("Shares overflow")),
    };

    total_shares.save(storage)?;

    // Update user's shares - this will not break as total_shares >= user_shares
    user_shares.0 += shares;
    user_shares.save(storage, sender.as_str().as_bytes())?;

    Ok(())
}

///
/// Removed items from internal supply
///
fn subtract_internal_supply(
    storage: &mut dyn Storage,
    total_shares: &mut TotalShares,
    shares: Uint256,
    total_tokens: &mut TotalTokens,
    tokens: Uint128,
    remove_supply: bool,
) -> StdResult<()> {
    // Update total shares
    match total_shares.0.checked_sub(shares) {
        Ok(total) => TotalShares(total).save(storage)?,
        Err(_) => return Err(StdError::generic_err("Insufficient shares")),
    };

    // Update total staked
    match total_tokens.0.checked_sub(tokens) {
        Ok(total) => TotalTokens(total).save(storage)?,
        Err(_) => return Err(StdError::generic_err("Insufficient tokens")),
    };

    if remove_supply {
        let supply = ReadonlyConfig::from_storage(storage).total_supply();
        if let Some(total) = supply.checked_sub(tokens.u128()) {
            Config::from_storage(storage).set_total_supply(total);
        } else {
            return Err(StdError::generic_err("Insufficient shares"));
        }
    }

    Ok(())
}

///
/// Updates total states to reflect balance changes
///
fn remove_balance(
    storage: &mut dyn Storage,
    stake_config: &StakeConfig,
    account: &Addr,
    account_cannon: &CanonicalAddr,
    amount: Uint128,
    time: u64,
) -> StdResult<()> {
    // Return insufficient funds
    let user_shares =
        UserShares::may_load(storage, account.as_str().as_bytes())?.expect("No funds");

    // Get total supplied tokens
    let mut total_shares = TotalShares::load(storage)?;
    let mut total_tokens = TotalTokens::load(storage)?;

    // Calculate shares per token supplied
    let shares = shares_per_token(stake_config, &amount, &total_tokens.0, &total_shares.0)?;

    // Update user's shares
    match user_shares.0.checked_sub(shares) {
        Ok(user_shares) => UserShares(user_shares).save(storage, account.as_str().as_bytes())?,
        Err(_) => return Err(StdError::generic_err("Insufficient shares")),
    }

    subtract_internal_supply(
        storage,
        &mut total_shares,
        shares,
        &mut total_tokens,
        amount,
        true,
    )?;

    // Load balance
    let mut balances = Balances::from_storage(storage);
    let mut account_balance = balances.balance(account_cannon);
    let account_tokens = account_balance;

    if let Some(new_balance) = account_balance.checked_sub(amount.u128()) {
        account_balance = new_balance;
    } else {
        return Err(StdError::generic_err(
            "This burn attempt would decrease the account's balance to a negative",
        ));
    }
    balances.set_account_balance(account_cannon, account_balance);
    remove_from_cooldown(storage, account, Uint128::new(account_tokens), amount, time)?;
    Ok(())
}

pub fn claim_rewards(
    storage: &mut dyn Storage,
    stake_config: &StakeConfig,
    sender: &Addr,
    sender_canon: &CanonicalAddr,
) -> StdResult<Uint128> {
    let user_shares = UserShares::may_load(storage, sender.as_str().as_bytes())?.expect("No funds");

    let user_balance = Balances::from_storage(storage).balance(sender_canon);

    // Get total supplied tokens
    let mut total_shares = TotalShares::load(storage)?;
    let mut total_tokens = TotalTokens::load(storage)?;

    let (reward_token, reward_shares) = calculate_rewards(
        stake_config,
        Uint128::new(user_balance),
        user_shares.0,
        total_tokens.0,
        total_shares.0,
    )?;

    // Do nothing if no rewards are gonna be claimed
    if reward_token.is_zero() {
        return Ok(reward_token);
    }

    match user_shares.0.checked_sub(reward_shares) {
        Ok(user_shares) => UserShares(user_shares).save(storage, sender.as_str().as_bytes())?,
        Err(_) => return Err(StdError::generic_err("Insufficient shares")),
    };

    subtract_internal_supply(
        storage,
        &mut total_shares,
        reward_shares,
        &mut total_tokens,
        reward_token,
        false,
    )?;

    Ok(reward_token)
}

pub fn shares_per_token(
    config: &StakeConfig,
    token_amount: &Uint128,
    total_tokens: &Uint128,
    total_shares: &Uint256,
) -> StdResult<Uint256> {
    let t_tokens = Uint256::from(*total_tokens);
    let t_shares = *total_shares;
    let tokens = Uint256::from(*token_amount);

    if total_tokens.is_zero() && total_shares.is_zero() {
        // Used to normalize the staked token to the stake token
        let token_multiplier =
            Uint256::from(10u128).checked_pow(config.decimal_difference.into())?;

        return match tokens.checked_mul(token_multiplier) {
            Ok(shares) => Ok(shares),
            Err(_) => Err(StdError::generic_err("Share calculation overflow")),
        };
    }

    return match tokens.checked_mul(t_shares) {
        Ok(shares) => Ok(shares.checked_div(t_tokens)?),
        Err(_) => Err(StdError::generic_err("Share calculation overflow")),
    };
}

pub fn tokens_per_share(
    config: &StakeConfig,
    shares_amount: &Uint256,
    total_tokens: &Uint128,
    total_shares: &Uint256,
) -> StdResult<Uint128> {
    let t_tokens = Uint256::from(*total_tokens);
    let t_shares = *total_shares;
    let shares = *shares_amount;

    if total_tokens.is_zero() && total_shares.is_zero() {
        // Used to normalize the staked token to the stake tokes
        let token_multiplier =
            Uint256::from(10u128).checked_pow(config.decimal_difference.try_into().unwrap())?;

        return match shares.checked_div(token_multiplier) {
            Ok(tokens) => Ok(tokens.try_into()?),
            Err(_) => Err(StdError::generic_err("Token calculation overflow")),
        };
    }

    return match shares.checked_mul(t_tokens) {
        Ok(tokens) => Ok(tokens.checked_div(t_shares)?.try_into()?),
        Err(_) => Err(StdError::generic_err("Token calculation overflow")),
    };
}

///
/// Returns rewards in tokens, and shares
///
pub fn calculate_rewards(
    config: &StakeConfig,
    tokens: Uint128,
    shares: Uint256,
    total_tokens: Uint128,
    total_shares: Uint256,
) -> StdResult<(Uint128, Uint256)> {
    let token_reward = tokens_per_share(config, &shares, &total_tokens, &total_shares)?
        .checked_sub(tokens.into())?;
    Ok((
        token_reward,
        shares_per_token(config, &token_reward, &total_tokens, &total_shares)?,
    ))
}

pub fn try_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Addr,
    from: Addr,
    amount: Uint128,
    msg: Option<Binary>,
    memo: Option<String>,
) -> StdResult<Response> {
    let sender_canon = deps.api.canonical_address(&sender)?;

    let stake_config = StakeConfig::load(deps.storage)?;

    if info.sender != stake_config.staked_token.address {
        return Err(StdError::generic_err("Not the stake token"));
    }

    let receive_type: ReceiveType;
    if let Some(msg) = msg {
        receive_type = from_binary(&msg)?;
    } else {
        return Err(StdError::generic_err("No receive type supplied in message"));
    }

    let symbol = ReadonlyConfig::from_storage(deps.storage)
        .constants()?
        .symbol;
    let mut messages = vec![];
    match receive_type {
        ReceiveType::Bond { use_from } => {
            let mut target = sender;
            let mut target_canon = sender_canon;
            if let Some(use_from) = use_from {
                if use_from {
                    target_canon = deps.api.canonical_address(&from)?;
                    target = from;
                }
            }

            // Update user stake
            add_balance(
                deps.storage,
                &stake_config,
                &target,
                &target_canon,
                amount,
            )?;

            // Store data
            store_stake(
                deps.storage,
                &target_canon,
                amount,
                symbol,
                memo,
                &env.block,
            )?;

            // Send tokens
            if let Some(treasury) = stake_config.treasury {
                messages.push(send_msg(
                    treasury,
                    amount.into(),
                    None,
                    None,
                    None,
                    256,
                    stake_config.staked_token.code_hash,
                    stake_config.staked_token.address,
                )?);
            } else {
                let mut stored_tokens = UnsentStakedTokens::load(deps.storage)?;
                stored_tokens.0 += amount;
                stored_tokens.save(deps.storage)?;
            }
        }

        ReceiveType::Reward => {
            let mut total_tokens = TotalTokens::load(deps.storage)?;
            total_tokens.0 += amount;
            total_tokens.save(deps.storage)?;

            // Store data
            store_add_reward(
                deps.storage,
                &sender_canon,
                amount,
                symbol,
                memo,
                &env.block,
            )?;
        }

        ReceiveType::Unbond => {
            let mut remaining_amount = amount;

            let mut daily_unbond_queue = DailyUnbondingQueue::load(deps.storage)?;

            while !daily_unbond_queue.0.0.is_empty() {
                remaining_amount = daily_unbond_queue.0.0[0].fund(remaining_amount);
                if daily_unbond_queue.0.0[0].is_funded() {
                    daily_unbond_queue.0.0.pop();
                }
                if remaining_amount == Uint128::zero() {
                    break;
                }
            }

            daily_unbond_queue.save(deps.storage)?;

            // Send back if overfunded
            if remaining_amount > Uint128::zero() {
                messages.push(send_msg(
                    sender,
                    remaining_amount.into(),
                    None,
                    None,
                    None,
                    256,
                    stake_config.staked_token.code_hash,
                    stake_config.staked_token.address,
                )?);
            }

            store_fund_unbond(
                deps.storage,
                &sender_canon,
                amount.checked_sub(remaining_amount)?,
                symbol,
                None,
                &env.block,
            )?;
        }
    };

    Ok(Response::new().set_data(to_binary(&HandleAnswer::Receive { status: Success })?))
}

pub fn remove_from_cooldown(
    store: &mut dyn Storage,
    user: &Addr,
    user_tokens: Uint128,
    remove_amount: Uint128,
    time: u64,
) -> StdResult<()> {
    let mut cooldown =
        UserCooldown::may_load(store, user.as_str().as_bytes())?.unwrap_or(UserCooldown {
            total: Uint128::zero(),
            queue: VecQueue(vec![]),
        });

    cooldown.update(time);

    let unlocked_tokens = user_tokens.checked_sub(cooldown.total)?;
    if remove_amount > unlocked_tokens {
        cooldown.remove_cooldown(remove_amount.checked_sub(unlocked_tokens)?);
    }
    cooldown.save(store, user.as_str().as_bytes())?;

    Ok(())
}

pub fn try_unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> StdResult<Response> {
    let sender = info.sender;
    let sender_canon = deps.api.canonical_address(&sender)?;

    let stake_config = StakeConfig::load(deps.storage)?;

    // Try to claim before unbonding
    let claim = claim_rewards(deps.storage, &stake_config, &sender, &sender_canon)?;

    // Subtract tokens from user balance
    remove_balance(
        deps.storage,
        &stake_config,
        &sender,
        &sender_canon,
        amount,
        env.block.time.seconds(),
    )?;

    let mut total_unbonding = TotalUnbonding::load(deps.storage)?;
    total_unbonding.0 += amount;
    total_unbonding.save(deps.storage)?;

    // Round to that day's public unbonding queue, initialize one if empty
    let mut daily_unbond_queue = DailyUnbondingQueue::load(deps.storage)?;
    // Will add or merge a new unbonding date
    daily_unbond_queue.0.push(&DailyUnbonding {
        unbonding: amount,
        funded: Default::default(),
        release: round_date(env.block.time.seconds() + stake_config.unbond_time),
    });

    daily_unbond_queue.save(deps.storage)?;

    // Check if user has an existing queue, if not, instantiate one
    let mut unbond_queue = UnbondingQueue::may_load(deps.storage, sender.as_str().as_bytes())?
        .unwrap_or(UnbondingQueue(VecQueue::new(vec![])));

    // Add unbonding to user queue
    unbond_queue.0.push(&Unbonding {
        amount,
        release: env.block.time.seconds() + stake_config.unbond_time,
    });

    unbond_queue.save(deps.storage, sender.as_str().as_bytes())?;

    // Store the tx
    let symbol = ReadonlyConfig::from_storage(deps.storage)
        .constants()?
        .symbol;
    let mut messages = vec![];
    if !claim.is_zero() {
        messages.push(send_msg(
            sender.clone(),
            claim.into(),
            None,
            None,
            None,
            256,
            stake_config.staked_token.code_hash,
            stake_config.staked_token.address,
        )?);

        store_claim_reward(
            deps.storage,
            &sender_canon,
            claim,
            symbol.clone(),
            None,
            &env.block,
        )?;
    }
    store_unbond(
        deps.storage,
        &deps.api.canonical_address(&sender)?,
        amount,
        symbol,
        None,
        &env.block,
    )?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::Unbond { status: Success })?))
}

pub fn try_claim_unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    let sender = &info.sender;
    let sender_canon = &deps.api.canonical_address(sender)?;

    let stake_config = StakeConfig::load(deps.storage)?;

    let mut total_unbonding = TotalUnbonding::load(deps.storage)?;

    // Instead of iterating over it we just look at its smallest value (first in queue)
    let daily_unbond_queue = DailyUnbondingQueue::load(deps.storage)?.0;

    // Check if user has an existing queue, if not, instantiate one
    let mut unbond_queue = UnbondingQueue::may_load(deps.storage, sender.as_str().as_bytes())?
        .expect("No unbonding queue found");

    let mut total = Uint128::zero();
    // Iterate over the sorted queue
    while !unbond_queue.0.0.is_empty() {
        // Since the queue is sorted, the moment we find a date above the current then we assume
        // that no other item in the queue is eligible
        if unbond_queue.0.0[0].release <= env.block.time.seconds() {
            // Daily unbond queue is also sorted, therefore as long as its next item is greater
            // than the unbond then we assume its funded
            if daily_unbond_queue.0.is_empty()
                || round_date(unbond_queue.0.0[0].release) < daily_unbond_queue.0[0].release
            {
                total += unbond_queue.0.0[0].amount;
                unbond_queue.0.pop();
            } else {
                break;
            }
        } else {
            break;
        }
    }

    if total == Uint128::zero() {
        return Err(StdError::generic_err("Nothing to claim"));
    }

    unbond_queue.save(deps.storage, sender.as_str().as_bytes())?;
    total_unbonding.0 = total_unbonding.0.checked_sub(total)?;
    total_unbonding.save(deps.storage)?;

    let symbol = ReadonlyConfig::from_storage(deps.storage)
        .constants()?
        .symbol;
    store_claim_unbond(
        deps.storage,
        sender_canon,
        total,
        symbol,
        None,
        &env.block,
    )?;

    let messages = vec![send_msg(
        sender.clone(),
        total.into(),
        None,
        None,
        None,
        256,
        stake_config.staked_token.code_hash,
        stake_config.staked_token.address,
    )?];

    Ok(Response::new().set_data(to_binary(&HandleAnswer::ClaimUnbond { status: Success })?))
}

pub fn try_claim_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    let stake_config = StakeConfig::load(deps.storage)?;

    let sender = &info.sender;
    let sender_canon = &deps.api.canonical_address(sender)?;

    let claim = claim_rewards(deps.storage, &stake_config, sender, sender_canon)?;

    if claim.is_zero() {
        return Err(StdError::generic_err("Nothing to claim"));
    }

    let messages = vec![send_msg(
        sender.clone(),
        claim.into(),
        None,
        None,
        None,
        256,
        stake_config.staked_token.code_hash,
        stake_config.staked_token.address,
    )?];

    let symbol = ReadonlyConfig::from_storage(deps.storage)
        .constants()?
        .symbol;
    store_claim_reward(
        deps.storage,
        sender_canon,
        claim,
        symbol,
        None,
        &env.block,
    )?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::ClaimRewards { status: Success })?))
}

pub fn try_stake_rewards(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    // Clam rewards
    let symbol = ReadonlyConfig::from_storage(deps.storage)
        .constants()?
        .symbol;
    let stake_config = StakeConfig::load(deps.storage)?;

    let sender = &info.sender;
    let sender_canon = &deps.api.canonical_address(sender)?;

    let claim = claim_rewards(deps.storage, &stake_config, sender, sender_canon)?;

    store_claim_reward(
        deps.storage,
        sender_canon,
        claim,
        symbol.clone(),
        None,
        &env.block,
    )?;

    // Stake rewards
    // Update user stake
    add_balance(
        deps.storage,
        &stake_config,
        sender,
        sender_canon,
        claim,
    )?;

    // Store data
    // Store data
    store_stake(
        deps.storage,
        sender_canon,
        claim,
        symbol,
        None,
        &env.block,
    )?;

    let mut messages = vec![];

    // Send tokens
    if let Some(treasury) = stake_config.treasury {
        messages.push(send_msg(
            treasury,
            claim.into(),
            None,
            None,
            None,
            256,
            stake_config.staked_token.code_hash,
            stake_config.staked_token.address,
        )?);
    } else {
        let mut stored_tokens = UnsentStakedTokens::load(deps.storage)?;
        stored_tokens.0 += claim;
        stored_tokens.save(deps.storage)?;
    }

    Ok(Response::new().set_data(to_binary(&HandleAnswer::StakeRewards { status: Success })?))
}

#[cfg(test)]
mod tests {
    use crate::stake::{calculate_rewards, round_date, shares_per_token, tokens_per_share};
    use shade_protocol::{
        contract_interfaces::staking::snip20_staking::stake::StakeConfig,
        utils::asset::Contract,
    };

    fn init_config(token_decimals: u8, shares_decimals: u8) -> StakeConfig {
        StakeConfig {
            unbond_time: 0,
            staked_token: Contract {
                address: Default::default(),
                code_hash: "".to_string(),
            },
            decimal_difference: shares_decimals - token_decimals,
            treasury: None,
        }
    }

    #[test]
    fn tokens_per_share_test() {
        let token_decimals = 8;
        let shares_decimals = 18;
        let config = init_config(token_decimals, shares_decimals);

        let token_1 = Uint128::new(10000000 * 10u128.pow(token_decimals.into()));
        let share_1 = Uint256::from(10000000 * 10u128.pow(shares_decimals.into()));

        // Check for proper instantiate
        assert_eq!(
            tokens_per_share(&config, &share_1, &Uint128::zero(), &Uint256::zero()).unwrap(),
            token_1
        );

        // Check for stability
        assert_eq!(
            tokens_per_share(&config, &share_1, &token_1, &share_1).unwrap(),
            token_1
        );
        assert_eq!(
            tokens_per_share(
                &config,
                &share_1,
                &(token_1 * Uint128::new(2)),
                &(share_1 * Uint256::from(2u32))
            )
            .unwrap(),
            token_1
        );

        // check that shares increase when tokens decrease
        assert!(
            tokens_per_share(&config, &share_1, &(token_1 * Uint128::new(2)), &share_1).unwrap()
                > token_1
        );

        // check that shares decrease when tokens increase
        assert!(
            tokens_per_share(
                &config,
                &share_1,
                &token_1,
                &(share_1 * Uint256::from(2u32))
            )
            .unwrap()
                < token_1
        );
    }

    #[test]
    fn shares_per_token_test() {
        let token_decimals = 8;
        let shares_decimals = 18;
        let config = init_config(token_decimals, shares_decimals);

        let token_1 = Uint128::new(100 * 10u128.pow(token_decimals.into()));
        let share_1 = Uint256::from(100 * 10u128.pow(shares_decimals.into()));

        // Check for proper instantiate
        assert_eq!(
            shares_per_token(&config, &token_1, &Uint128::zero(), &Uint256::zero()).unwrap(),
            share_1
        );

        // Check for stability
        assert_eq!(
            shares_per_token(&config, &token_1, &token_1, &share_1).unwrap(),
            share_1
        );
        assert_eq!(
            shares_per_token(
                &config,
                &token_1,
                &(token_1 * Uint128::new(2)),
                &(share_1 * Uint256::from(2u32))
            )
            .unwrap(),
            share_1
        );

        // check that shares increase when tokens decrease
        assert!(
            shares_per_token(&config, &token_1, &(token_1 * Uint128::new(2)), &share_1).unwrap()
                < share_1
        );

        // check that shares decrease when tokens increase
        assert!(
            shares_per_token(
                &config,
                &token_1,
                &token_1,
                &(share_1 * Uint256::from(2u32))
            )
            .unwrap()
                > share_1
        );
    }

    #[test]
    fn round_date_test() {
        assert_eq!(round_date(1645740448), 1645660800)
    }

    #[test]
    fn calculate_rewards_test() {
        let token_decimals = 8;
        let shares_decimals = 18;
        let config = init_config(token_decimals, shares_decimals);

        // Tester has 100 tokens
        // Other user has 50

        let u_t = Uint128::new(100 * 10u128.pow(token_decimals.into()));
        let mut u_s = Uint256::from(100 * 10u128.pow(shares_decimals.into()));
        let mut t_t = Uint128::new(150 * 10u128.pow(token_decimals.into()));
        let mut t_s = Uint256::from(150 * 10u128.pow(shares_decimals.into()));

        // No rewards
        let (tokens, shares) = calculate_rewards(&config, u_t, u_s, t_t, t_s).unwrap();

        assert_eq!(tokens, Uint128::zero());
        assert_eq!(shares, Uint256::zero());

        // Some rewards
        // We add 300 tokens, tester should get 200 tokens
        let reward = 300 * 10u128.pow(token_decimals.into());
        t_t += Uint128::new(reward);
        let (tokens, shares) = calculate_rewards(&config, u_t, u_s, t_t, t_s).unwrap();

        assert_eq!(tokens.u128(), reward * 2 / 3);
        t_t = t_t - tokens;
        // We should receive 2/3 of current shares
        assert_eq!(shares, u_s * Uint256::from(2u32) / Uint256::from(3u32));
        u_s = u_s - shares;
        t_s = t_s - shares;

        // After claiming
        let (tokens, shares) = calculate_rewards(&config, u_t, u_s, t_t, t_s).unwrap();

        assert_eq!(tokens, Uint128::zero());
        assert_eq!(shares, Uint256::zero());
    }

    #[test]
    fn simulate_claim_rewards() {
        let token_decimals = 8;
        let shares_decimals = 18;
        let config = init_config(token_decimals, shares_decimals);
        let mut user_shares = Uint256::from(50000000000000u128);

        let user_balance = Uint128::new(5000);

        // Get total supplied tokens
        let mut total_shares = Uint256::from(50000000000000u128);
        let mut total_tokens = Uint128::new(5000);

        let (reward_token, reward_shares) = calculate_rewards(
            &config,
            user_balance,
            user_shares,
            total_tokens,
            total_shares,
        )
        .unwrap();

        assert_eq!(reward_token, Uint128::zero());
    }

    use shade_protocol::c_std::{Uint128, Uint256};
    use rand::Rng;

    #[test]
    fn staking_simulation() {
        let token_decimals = 8;
        let shares_decimals = 18;
        let config = init_config(token_decimals, shares_decimals);

        let mut t_t = Uint128::zero();
        let mut t_s = Uint256::zero();
        let mut rand = rand::thread_rng();

        let mut stakers = vec![];

        for _ in 0..10 {
            // Generate stakers in this round
            for _ in 0..rand.gen_range(1..=4) {
                let tokens =
                    Uint128::new(rand.gen_range(1..100 * 10u128.pow(token_decimals.into())));

                let shares = shares_per_token(&config, &tokens, &t_t, &t_s).unwrap();

                stakers.push((tokens, shares));

                t_t += tokens;
                t_s += shares;
            }

            // Add random rewards
            t_t += Uint128::new(rand.gen_range(1u128..t_t.u128() / 2u128));

            // Claim and unstake
            for _ in 0..rand.gen_range(0..=stakers.len() / 2) {
                let (mut tokens, mut shares) = stakers.remove(rand.gen_range(0..stakers.len()));
                let (r_tokens, r_shares) =
                    calculate_rewards(&config, tokens, shares, t_t, t_s).unwrap();

                t_t -= r_tokens;
                t_s -= r_shares;
                shares -= r_shares;

                let (r_tokens, r_shares) =
                    calculate_rewards(&config, tokens, shares, t_t, t_s).unwrap();
                assert_eq!(r_tokens, Uint128::zero());
                assert_eq!(r_shares, Uint256::zero());

                // Unstake
                t_t -= tokens;
                t_s -= shares;
            }

            // Claim the rest
            while !stakers.is_empty() {
                let (mut tokens, mut shares) = stakers.pop().unwrap();
                let (r_tokens, r_shares) =
                    calculate_rewards(&config, tokens, shares, t_t, t_s).unwrap();

                t_t -= r_tokens;
                t_s -= r_shares;
                shares -= r_shares;

                let (r_tokens, r_shares) =
                    calculate_rewards(&config, tokens, shares, t_t, t_s).unwrap();
                assert_eq!(r_tokens, Uint128::zero());
                assert_eq!(r_shares, Uint256::zero());
            }
        }
    }
}
