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
use cosmwasm_std::{
    from_binary,
    to_binary,
    Api,
    Binary,
    CanonicalAddr,
    Decimal,
    Env,
    Extern,
    HandleResponse,
    HumanAddr,
    Querier,
    StdError,
    StdResult,
    Storage,
    Uint128,
};
use ethnum::u256;
use secret_toolkit::snip20::send_msg;
use shade_protocol::{
    contract_interfaces::staking::snip20_staking::{
        stake::{DailyUnbonding, StakeConfig, Unbonding, VecQueue},
        ReceiveType,
    },
    utils::storage::default::{BucketStorage, SingletonStorage},
};

//TODO: set errors

pub fn try_update_stake_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    unbond_time: Option<u64>,
    disable_treasury: bool,
    treasury: Option<HumanAddr>,
) -> StdResult<HandleResponse> {
    let config = Config::from_storage(&mut deps.storage);

    check_if_admin(&config, &env.message.sender)?;

    let mut stake_config = StakeConfig::load(&deps.storage)?;

    if let Some(unbond_time) = unbond_time {
        stake_config.unbond_time = unbond_time;
    }

    let mut messages = vec![];

    if disable_treasury {
        stake_config.treasury = None;
    } else if let Some(treasury) = treasury {
        stake_config.treasury = Some(treasury.clone());

        let unsent_tokens = UnsentStakedTokens::load(&deps.storage)?;
        if unsent_tokens.0 != Uint128::zero() {
            messages.push(send_msg(
                treasury,
                unsent_tokens.0,
                None,
                None,
                None,
                258,
                stake_config.staked_token.code_hash.clone(),
                stake_config.staked_token.address.clone(),
            )?);
            UnsentStakedTokens(Uint128::zero()).save(&mut deps.storage)?;
        }
    }

    stake_config.save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateStakeConfig {
            status: Success,
        })?),
    })
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
fn add_balance<S: Storage>(
    storage: &mut S,
    stake_config: &StakeConfig,
    sender: &HumanAddr,
    sender_canon: &CanonicalAddr,
    amount: u128,
) -> StdResult<()> {
    // Check if user account exists
    let mut user_shares = UserShares::may_load(storage, sender.as_str().as_bytes())?
        .unwrap_or(UserShares(Uint128::zero()));

    // Update user staked tokens
    let mut balances = Balances::from_storage(storage);
    let mut account_balance = balances.balance(sender_canon);
    if let Some(new_balance) = account_balance.checked_add(amount) {
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
    if let Some(total_staked) = total_tokens.0.u128().checked_add(amount) {
        TotalTokens(Uint128(total_staked)).save(storage)?;
    } else {
        return Err(StdError::generic_err("Total staked tokens overflow"));
    }
    let supply = ReadonlyConfig::from_storage(storage).total_supply();
    Config::from_storage(storage).set_total_supply(supply + amount);

    // Calculate shares per token supplied
    let shares = Uint128(shares_per_token(
        stake_config,
        &amount,
        &total_tokens.0.u128(),
        &total_shares.0.u128(),
    )?);

    // Update total shares
    if let Some(total_added_shares) = total_shares.0.u128().checked_add(shares.u128()) {
        total_shares = TotalShares(Uint128(total_added_shares));
    } else {
        return Err(StdError::generic_err("Shares overflow"));
    }
    total_shares.save(storage)?;

    // Update user's shares - this will not break as total_shares >= user_shares
    user_shares.0 += shares;
    user_shares.save(storage, sender.as_str().as_bytes())?;

    Ok(())
}

///
/// Removed items from internal supply
///
fn subtract_internal_supply<S: Storage>(
    storage: &mut S,
    total_shares: &mut TotalShares,
    shares: u128,
    total_tokens: &mut TotalTokens,
    tokens: u128,
    remove_supply: bool,
) -> StdResult<()> {
    // Update total shares
    if let Some(total) = total_shares.0.u128().checked_sub(shares) {
        TotalShares(Uint128(total)).save(storage)?;
    } else {
        return Err(StdError::generic_err("Insufficient shares"));
    }

    // Update total staked
    if let Some(total) = total_tokens.0.u128().checked_sub(tokens) {
        TotalTokens(Uint128(total)).save(storage)?;
    } else {
        return Err(StdError::generic_err("Insufficient tokens"));
    }
    if remove_supply {
        let supply = ReadonlyConfig::from_storage(storage).total_supply();
        if let Some(total) = supply.checked_sub(tokens) {
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
fn remove_balance<S: Storage>(
    storage: &mut S,
    stake_config: &StakeConfig,
    account: &HumanAddr,
    account_cannon: &CanonicalAddr,
    amount: u128,
    time: u64,
) -> StdResult<()> {
    // Return insufficient funds
    let user_shares =
        UserShares::may_load(storage, account.as_str().as_bytes())?.expect("No funds");

    // Get total supplied tokens
    let mut total_shares = TotalShares::load(storage)?;
    let mut total_tokens = TotalTokens::load(storage)?;

    // Calculate shares per token supplied
    let shares = shares_per_token(
        stake_config,
        &amount,
        &total_tokens.0.u128(),
        &total_shares.0.u128(),
    )?;

    // Update user's shares
    if let Some(user_shares) = user_shares.0.u128().checked_sub(shares) {
        UserShares(Uint128(user_shares)).save(storage, account.as_str().as_bytes())?;
    } else {
        return Err(StdError::generic_err("Insufficient shares"));
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

    if let Some(new_balance) = account_balance.checked_sub(amount) {
        account_balance = new_balance;
    } else {
        return Err(StdError::generic_err(
            "This burn attempt would decrease the account's balance to a negative",
        ));
    }
    balances.set_account_balance(account_cannon, account_balance);
    remove_from_cooldown(
        storage,
        account,
        Uint128(account_tokens),
        Uint128(amount),
        time,
    )?;
    Ok(())
}

pub fn claim_rewards<S: Storage>(
    storage: &mut S,
    stake_config: &StakeConfig,
    sender: &HumanAddr,
    sender_canon: &CanonicalAddr,
) -> StdResult<u128> {
    let user_shares = UserShares::may_load(storage, sender.as_str().as_bytes())?.expect("No funds");

    let user_balance = Balances::from_storage(storage).balance(sender_canon);

    // Get total supplied tokens
    let mut total_shares = TotalShares::load(storage)?;
    let mut total_tokens = TotalTokens::load(storage)?;

    let (reward_token, reward_shares) = calculate_rewards(
        stake_config,
        user_balance,
        user_shares.0.u128(),
        total_tokens.0.u128(),
        total_shares.0.u128(),
    )?;

    // Do nothing if no rewards are gonna be claimed
    if reward_token == 0 {
        return Ok(reward_token);
    }

    if let Some(user_shares) = user_shares.0.u128().checked_sub(reward_shares) {
        UserShares(Uint128(user_shares)).save(storage, sender.as_str().as_bytes())?;
    } else {
        return Err(StdError::generic_err("Insufficient shares"));
    }

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
    token_amount: &u128,
    total_tokens: &u128,
    total_shares: &u128,
) -> StdResult<u128> {
    let t_tokens = u256::from(*total_tokens);
    let t_shares = u256::from(*total_shares);
    let tokens = u256::from(*token_amount);

    if *total_tokens == 0 && *total_shares == 0 {
        // Used to normalize the staked token to the stake token
        let token_multiplier = u256::from(10u16).pow(config.decimal_difference.into());
        if let Some(shares) = tokens.checked_mul(token_multiplier) {
            return Ok(shares.as_u128());
        } else {
            return Err(StdError::generic_err("Share calculation overflow"));
        }
    }

    if let Some(shares) = tokens.checked_mul(t_shares) {
        return Ok((shares / t_tokens).as_u128());
    } else {
        return Err(StdError::generic_err("Share calculation overflow"));
    }
}

pub fn tokens_per_share(
    config: &StakeConfig,
    shares_amount: &u128,
    total_tokens: &u128,
    total_shares: &u128,
) -> StdResult<u128> {
    let t_tokens = u256::from(*total_tokens);
    let t_shares = u256::from(*total_shares);
    let shares = u256::from(*shares_amount);

    if *total_tokens == 0 && *total_shares == 0 {
        // Used to normalize the staked token to the stake tokes
        let token_multiplier = u256::from(10u16).pow(config.decimal_difference.into());
        if let Some(tokens) = shares.checked_div(token_multiplier) {
            return Ok(tokens.as_u128());
        } else {
            return Err(StdError::generic_err("Token calculation overflow"));
        }
    }

    if let Some(tokens) = shares.checked_mul(t_tokens) {
        return Ok((tokens / t_shares).as_u128());
    } else {
        return Err(StdError::generic_err("Token calculation overflow"));
    }
}

///
/// Returns rewards in tokens, and shares
///
pub fn calculate_rewards(
    config: &StakeConfig,
    tokens: u128,
    shares: u128,
    total_tokens: u128,
    total_shares: u128,
) -> StdResult<(u128, u128)> {
    let token_reward = tokens_per_share(config, &shares, &total_tokens, &total_shares)? - tokens;
    Ok((
        token_reward,
        shares_per_token(config, &token_reward, &total_tokens, &total_shares)?,
    ))
}

pub fn try_receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    sender: HumanAddr,
    from: HumanAddr,
    amount: Uint128,
    msg: Option<Binary>,
    memo: Option<String>,
) -> StdResult<HandleResponse> {
    let sender_canon = deps.api.canonical_address(&sender)?;

    let stake_config = StakeConfig::load(&deps.storage)?;

    if env.message.sender != stake_config.staked_token.address {
        return Err(StdError::generic_err("Not the stake token"));
    }

    let receive_type: ReceiveType;
    if let Some(msg) = msg {
        receive_type = from_binary(&msg)?;
    } else {
        return Err(StdError::generic_err("No receive type supplied in message"));
    }

    let symbol = ReadonlyConfig::from_storage(&deps.storage)
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
                &mut deps.storage,
                &stake_config,
                &target,
                &target_canon,
                amount.u128(),
            )?;

            // Store data
            store_stake(
                &mut deps.storage,
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
                    amount,
                    None,
                    None,
                    None,
                    256,
                    stake_config.staked_token.code_hash,
                    stake_config.staked_token.address,
                )?);
            } else {
                let mut stored_tokens = UnsentStakedTokens::load(&deps.storage)?;
                stored_tokens.0 += amount;
                stored_tokens.save(&mut deps.storage)?;
            }
        }

        ReceiveType::Reward => {
            let mut total_tokens = TotalTokens::load(&deps.storage)?;
            total_tokens.0 += amount;
            total_tokens.save(&mut deps.storage)?;

            // Store data
            store_add_reward(
                &mut deps.storage,
                &sender_canon,
                amount,
                symbol,
                memo,
                &env.block,
            )?;
        }

        ReceiveType::Unbond => {
            let mut remaining_amount = amount;

            let mut daily_unbond_queue = DailyUnbondingQueue::load(&deps.storage)?;

            while !daily_unbond_queue.0.0.is_empty() {
                remaining_amount = daily_unbond_queue.0.0[0].fund(remaining_amount);
                if daily_unbond_queue.0.0[0].is_funded() {
                    daily_unbond_queue.0.0.pop();
                }
                if remaining_amount == Uint128::zero() {
                    break;
                }
            }

            daily_unbond_queue.save(&mut deps.storage)?;

            // Send back if overfunded
            if remaining_amount > Uint128::zero() {
                messages.push(send_msg(
                    sender,
                    remaining_amount,
                    None,
                    None,
                    None,
                    256,
                    stake_config.staked_token.code_hash,
                    stake_config.staked_token.address,
                )?);
            }

            store_fund_unbond(
                &mut deps.storage,
                &sender_canon,
                (amount - remaining_amount)?,
                symbol,
                None,
                &env.block,
            )?;
        }
    };

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Receive { status: Success })?),
    })
}

pub fn remove_from_cooldown<S: Storage>(
    store: &mut S,
    user: &HumanAddr,
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

    let unlocked_tokens = (user_tokens - cooldown.total)?;
    if remove_amount > unlocked_tokens {
        cooldown.remove_cooldown((remove_amount - unlocked_tokens)?);
    }
    cooldown.save(store, user.as_str().as_bytes())?;

    Ok(())
}

pub fn try_unbond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let sender = env.message.sender;
    let sender_canon = deps.api.canonical_address(&sender)?;

    let stake_config = StakeConfig::load(&deps.storage)?;

    // Try to claim before unbonding
    let claim = claim_rewards(&mut deps.storage, &stake_config, &sender, &sender_canon)?;

    // Subtract tokens from user balance
    remove_balance(
        &mut deps.storage,
        &stake_config,
        &sender,
        &sender_canon,
        amount.u128(),
        env.block.time,
    )?;

    let mut total_unbonding = TotalUnbonding::load(&deps.storage)?;
    total_unbonding.0 += amount;
    total_unbonding.save(&mut deps.storage)?;

    // Round to that day's public unbonding queue, initialize one if empty
    let mut daily_unbond_queue = DailyUnbondingQueue::load(&deps.storage)?;
    // Will add or merge a new unbonding date
    daily_unbond_queue.0.push(&DailyUnbonding {
        unbonding: amount,
        funded: Default::default(),
        release: round_date(env.block.time + stake_config.unbond_time),
    });

    daily_unbond_queue.save(&mut deps.storage)?;

    // Check if user has an existing queue, if not, init one
    let mut unbond_queue = UnbondingQueue::may_load(&deps.storage, sender.as_str().as_bytes())?
        .unwrap_or(UnbondingQueue(VecQueue::new(vec![])));

    // Add unbonding to user queue
    unbond_queue.0.push(&Unbonding {
        amount,
        release: env.block.time + stake_config.unbond_time,
    });

    unbond_queue.save(&mut deps.storage, sender.as_str().as_bytes())?;

    // Store the tx
    let symbol = ReadonlyConfig::from_storage(&deps.storage)
        .constants()?
        .symbol;
    let mut messages = vec![];
    if claim != 0 {
        messages.push(send_msg(
            sender.clone(),
            Uint128(claim),
            None,
            None,
            None,
            256,
            stake_config.staked_token.code_hash,
            stake_config.staked_token.address,
        )?);

        store_claim_reward(
            &mut deps.storage,
            &sender_canon,
            Uint128(claim),
            symbol.clone(),
            None,
            &env.block,
        )?;
    }
    store_unbond(
        &mut deps.storage,
        &deps.api.canonical_address(&sender)?,
        amount,
        symbol,
        None,
        &env.block,
    )?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Unbond { status: Success })?),
    })
}

pub fn try_claim_unbond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let sender = &env.message.sender;
    let sender_canon = &deps.api.canonical_address(sender)?;

    let stake_config = StakeConfig::load(&deps.storage)?;

    let mut total_unbonding = TotalUnbonding::load(&deps.storage)?;

    // Instead of iterating over it we just look at its smallest value (first in queue)
    let daily_unbond_queue = DailyUnbondingQueue::load(&deps.storage)?.0;

    // Check if user has an existing queue, if not, init one
    let mut unbond_queue = UnbondingQueue::may_load(&deps.storage, sender.as_str().as_bytes())?
        .expect("No unbonding queue found");

    let mut total = Uint128::zero();
    // Iterate over the sorted queue
    while !unbond_queue.0.0.is_empty() {
        // Since the queue is sorted, the moment we find a date above the current then we assume
        // that no other item in the queue is eligible
        if unbond_queue.0.0[0].release <= env.block.time {
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

    unbond_queue.save(&mut deps.storage, sender.as_str().as_bytes())?;
    total_unbonding.0 = (total_unbonding.0 - total)?;
    total_unbonding.save(&mut deps.storage)?;

    let symbol = ReadonlyConfig::from_storage(&deps.storage)
        .constants()?
        .symbol;
    store_claim_unbond(
        &mut deps.storage,
        sender_canon,
        total,
        symbol,
        None,
        &env.block,
    )?;

    let messages = vec![send_msg(
        sender.clone(),
        total,
        None,
        None,
        None,
        256,
        stake_config.staked_token.code_hash,
        stake_config.staked_token.address,
    )?];

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ClaimUnbond { status: Success })?),
    })
}

pub fn try_claim_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let stake_config = StakeConfig::load(&deps.storage)?;

    let sender = &env.message.sender;
    let sender_canon = &deps.api.canonical_address(sender)?;

    let claim = claim_rewards(&mut deps.storage, &stake_config, sender, sender_canon)?;

    if claim == 0 {
        return Err(StdError::generic_err("Nothing to claim"));
    }

    let messages = vec![send_msg(
        sender.clone(),
        Uint128(claim),
        None,
        None,
        None,
        256,
        stake_config.staked_token.code_hash,
        stake_config.staked_token.address,
    )?];

    let symbol = ReadonlyConfig::from_storage(&deps.storage)
        .constants()?
        .symbol;
    store_claim_reward(
        &mut deps.storage,
        sender_canon,
        Uint128(claim),
        symbol,
        None,
        &env.block,
    )?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ClaimRewards { status: Success })?),
    })
}

pub fn try_stake_rewards<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    // Clam rewards
    let symbol = ReadonlyConfig::from_storage(&deps.storage)
        .constants()?
        .symbol;
    let stake_config = StakeConfig::load(&deps.storage)?;

    let sender = &env.message.sender;
    let sender_canon = &deps.api.canonical_address(sender)?;

    let claim = Uint128(claim_rewards(
        &mut deps.storage,
        &stake_config,
        sender,
        sender_canon,
    )?);

    store_claim_reward(
        &mut deps.storage,
        sender_canon,
        claim,
        symbol.clone(),
        None,
        &env.block,
    )?;

    // Stake rewards
    // Update user stake
    add_balance(
        &mut deps.storage,
        &stake_config,
        sender,
        sender_canon,
        claim.u128(),
    )?;

    // Store data
    // Store data
    store_stake(
        &mut deps.storage,
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
            claim,
            None,
            None,
            None,
            256,
            stake_config.staked_token.code_hash,
            stake_config.staked_token.address,
        )?);
    } else {
        let mut stored_tokens = UnsentStakedTokens::load(&deps.storage)?;
        stored_tokens.0 += claim;
        stored_tokens.save(&mut deps.storage)?;
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::StakeRewards { status: Success })?),
    })
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

        let token_1 = 10000000 * 10u128.pow(token_decimals.into());
        let share_1 = 10000000 * 10u128.pow(shares_decimals.into());

        // Check for proper init
        assert_eq!(
            tokens_per_share(&config, &share_1, &0, &0).unwrap(),
            token_1
        );

        // Check for stability
        assert_eq!(
            tokens_per_share(&config, &share_1, &token_1, &share_1).unwrap(),
            token_1
        );
        assert_eq!(
            tokens_per_share(&config, &share_1, &(token_1 * 2), &(share_1 * 2)).unwrap(),
            token_1
        );

        // check that shares increase when tokens decrease
        assert!(tokens_per_share(&config, &share_1, &(token_1 * 2), &share_1).unwrap() > token_1);

        // check that shares decrease when tokens increase
        assert!(tokens_per_share(&config, &share_1, &token_1, &(share_1 * 2)).unwrap() < token_1);
    }

    #[test]
    fn shares_per_token_test() {
        let token_decimals = 8;
        let shares_decimals = 18;
        let config = init_config(token_decimals, shares_decimals);

        let token_1 = 100 * 10u128.pow(token_decimals.into());
        let share_1 = 100 * 10u128.pow(shares_decimals.into());

        // Check for proper init
        assert_eq!(
            shares_per_token(&config, &token_1, &0, &0).unwrap(),
            share_1
        );

        // Check for stability
        assert_eq!(
            shares_per_token(&config, &token_1, &token_1, &share_1).unwrap(),
            share_1
        );
        assert_eq!(
            shares_per_token(&config, &token_1, &(token_1 * 2), &(share_1 * 2)).unwrap(),
            share_1
        );

        // check that shares increase when tokens decrease
        assert!(shares_per_token(&config, &token_1, &(token_1 * 2), &share_1).unwrap() < share_1);

        // check that shares decrease when tokens increase
        assert!(shares_per_token(&config, &token_1, &token_1, &(share_1 * 2)).unwrap() > share_1);
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

        let u_t = 100 * 10u128.pow(token_decimals.into());
        let mut u_s = 100 * 10u128.pow(shares_decimals.into());
        let mut t_t = 150 * 10u128.pow(token_decimals.into());
        let mut t_s = 150 * 10u128.pow(shares_decimals.into());

        // No rewards
        let (tokens, shares) = calculate_rewards(&config, u_t, u_s, t_t, t_s).unwrap();

        assert_eq!(tokens, 0);
        assert_eq!(shares, 0);

        // Some rewards
        // We add 300 tokens, tester should get 200 tokens
        let reward = 300 * 10u128.pow(token_decimals.into());
        t_t += reward;
        let (tokens, shares) = calculate_rewards(&config, u_t, u_s, t_t, t_s).unwrap();

        assert_eq!(tokens, reward * 2 / 3);
        t_t = t_t - tokens;
        // We should receive 2/3 of current shares
        assert_eq!(shares, u_s * 2 / 3);
        u_s = u_s - shares;
        t_s = t_s - shares;

        // After claiming
        let (tokens, shares) = calculate_rewards(&config, u_t, u_s, t_t, t_s).unwrap();

        assert_eq!(tokens, 0);
        assert_eq!(shares, 0);
    }

    #[test]
    fn simulate_claim_rewards() {
        let token_decimals = 8;
        let shares_decimals = 18;
        let config = init_config(token_decimals, shares_decimals);
        let mut user_shares = Uint128::new(50000000000000);

        let user_balance = 5000;

        // Get total supplied tokens
        let mut total_shares = Uint128::new(50000000000000);
        let mut total_tokens = Uint128::new(5000);

        let (reward_token, reward_shares) = calculate_rewards(
            &config,
            user_balance,
            user_shares.u128(),
            total_tokens.u128(),
            total_shares.u128(),
        )
        .unwrap();

        assert_eq!(reward_token, 0);
    }

    use cosmwasm_math_compat::Uint128;
    use rand::Rng;

    #[test]
    fn staking_simulation() {
        let token_decimals = 8;
        let shares_decimals = 18;
        let config = init_config(token_decimals, shares_decimals);

        let mut t_t = 0;
        let mut t_s = 0;
        let mut rand = rand::thread_rng();

        let mut stakers = vec![];

        for _ in 0..10 {
            // Generate stakers in this round
            for _ in 0..rand.gen_range(1..=4) {
                let tokens = rand.gen_range(1..100 * 10u128.pow(token_decimals.into()));

                let shares = shares_per_token(&config, &tokens, &t_t, &t_s).unwrap();

                stakers.push((tokens, shares));

                t_t += tokens;
                t_s += shares;
            }

            // Add random rewards
            t_t += rand.gen_range(1..t_t / 2);

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
                assert_eq!(r_tokens, 0);
                assert_eq!(r_shares, 0);

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
                assert_eq!(r_tokens, 0);
                assert_eq!(r_shares, 0);
            }
        }
    }
}
