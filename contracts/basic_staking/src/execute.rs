use shade_protocol::c_std::{
    to_binary,
    Addr,
    Api,
    BalanceResponse,
    BankQuery,
    Binary,
    Coin,
    CosmosMsg,
    Deps,
    DepsMut,
    DistributionMsg,
    Env,
    MessageInfo,
    Querier,
    Response,
    StakingMsg,
    StdError,
    StdResult,
    Storage,
    Uint128,
    Validator,
};

use shade_protocol::snip20::helpers::{deposit_msg, redeem_msg, send_msg};

use shade_protocol::{
    basic_staking::{Config, ExecuteAnswer},
    utils::{
        asset::{scrt_balance, Contract},
        generic_response::ResponseStatus,
        wrap::{unwrap, wrap_and_send},
    },
};

use crate::{query, storage::*};

pub fn update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: Config,
) -> StdResult<Response> {
    let cur_config = CONFIG.load(deps.storage)?;

    if cur_config.admins.contains(&info.sender) {
        return Err(StdError::generic_err("unauthorized"));
    }

    // Save new info
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

    let reward_tokens = REWARD_TOKENS.load(deps.storage)?;

    if reward_tokens.contains(token)? {
        return Err(StdError::generic_err("Reward token already registered"));
    }

    Ok(
        Response::new()
            .add_messages(vec![
                set_viewing_key_msg(msg.viewing_key, None, &config.token)?,
                register_receive(env.contract.code_hash, None, &config.token)?,
            ])
            .set_data(
                to_binary(&ExecuteAnswer::RegisterRewards {
                    status: ResponseStatus::Success,
                })?
            )
    )
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
    let config = CONFIG.load(deps.storage)?;

    match msg {
        Some(m) => match from_binary(&m)? {
            Action::Stake {} => {
                let stake_token = STAKE_TOKEN.load(deps.storage)?;
                if info.sender != stake_token.address {
                    return Err(StdError::generic_err(format!("Invalid Stake: {}", info.sender));
                }

                // TODO claim rewards based on old stake amount
                USER_LAST_CLAIM.save(deps.storage, from, &env.block.time.seconds())?;

                if let Some(user_stake) = USER_STAKED.may_load(deps.storage)? {
                    USER_STAKED.save(deps.storage, from, &(user_stake + amount))?;
                }
                else {
                    USER_STAKED.save(deps.storage, from, &amount)?;
                }

                let total_staked = TOTAL_STAKED.load(deps.storage)?;
                TOTAL_STAKED.save(deps.storage, &(total_staked + amount))?;

                Ok(Response::new()
                    .set_data(
                        to_binary(&ExecuteAnswer::Stake {
                            status: ResponseStatus::Success,
                        })?
                    ))
            }
            Action::Rewards {
                start,
                end,
            } => {
                let reward_tokens = REWARD_TOKENS.load(deps.storage)?;

                if let Some(token) = reward_tokens
                    .iter()
                    .find(|contract| => contract.address == info.sender) {

                    if start >= end {
                        return Err(StdError::generic_err("'start' must be after 'end'"));
                    }

                    if start > env.block.time.seconds() {
                        return Err(StdError::generic_err("Cannot start emitting in the past"));
                    }

                    let reward_pools = REWARD_POOLS.load(deps.storage)?;
                    let uuid = match reward_pools.is_empty() {
                        true => Uint128::zero(),
                        false => reward_pools.last().uuid + 1,
                    };

                    // Tokens per second emitted from this pool
                    let rate = amount * 10u128.pow(18) / (end - start);

                    reward_pools.push(RewardPool {
                        id: uuid,
                        amount,
                        start,
                        end,
                        token,
                        rate,
                        reward_per_token: Uint128::zero(),
                    });
                    REWARD_POOLS.save(deps.storage, &reward_pools)?;

                    Ok(Response::new()
                        .set_data(
                            to_binary(&ExecuteAnswer::Rewards {
                                status: ResponseStatus::Success,
                            })?
                        ))
                }
                else {
                    return Err(StdError::generic_err(format!("Invalid Reward: {}", info.sender));
                }

            }
        },
        None => {
            return Err(StdError::generic_err("No action provided"));
        }
    }
}

pub fn claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {

    let user_last_claim = USER_LAST_CLAIM.load(deps.storage)?;
    let user_staked = USER_STAKED.load(deps.storage, &info.sender)?;
    let reward_pools = REWARD_POOLS.load(deps.storage)?;
    let total_staked = TOTAL_STAKED.load(deps.storage)?;
    let now = env.block.time.seconds();

    let mut response = Response::new();

    for mut reward_pool in reward_pools {

        if now < reward_pool.start {
            // reward pool hasn't started emitting yet
            continue;
        }

        let reward_per_token = reward_pool.reward_per_token + (reward_pool.rate * (reward_pool.end - reward_pool.start)) / total_staked;

        let user_reward = (reward_per_token - reward_pool.reward_per_token) * user_staked;

        // Send reward
        response.add_message(
            send_msg(
                info.sender,
                user_reward,
                None,
                None,
                None,
                &reward_pool.token,
            )
        );

        reward_pool.reward_per_token += reward_per_token;
        USER_REWARD_PER_TOKEN.save(deps.storage, &reward_per_token)?;

        // TODO adjust reward_pool.reward_per_token in place
    }

    //TODO save updated reward pools
    USER_LAST_CLAIM.save(deps.storage, &now)?;

    Ok(response.set_data(
        to_binary(&ExecuteAnswer::Claim {
            status: ResponseStatus::Success,
        })?
    ))
}

pub fn unbond(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> StdResult<Response> {

    let config = CONFIG.load(deps.storage)?;

    if let Some(user_staked) = USER_STAKED.may_load(deps.storage, &info.sender)? {
        if user_staked.is_zero() {
            return Err(StdError::generic_err("User has no staked tokens"));
        }
        if user_staked < amount {
            return Err(StdError::generic_err(format!("Cannot unbond {}, staked: {}", amount, user_staked)));
        }

        // TODO claim user rewards

        let total_staked = TOTAL_STAKED.load(deps.storage)?;
        TOTAL_STAKED.save(deps.storage, &(total_staked - amount))?;

        USER_STAKED.save(deps.storage, &(user_staked - amount))?;

        let user_unbonding = USER_UNBONDING.load(deps.storage, &info.sender)?;
        user_unbonding.push(Unbonding {
            amount,
            complete: env.block.time.seconds() + config.unbond_period,
        });

        USER_UNBONDING.save(deps.storage, &user_unbonding)?;

        Ok(Response::new()
            .set_data(
                to_binary(&ExecuteAnswer::Unbond {
                    status: ResponseStatus::Success,
                })?
            )
        )
    }
    else {
        return Err(StdError::generic_err("User has no staked tokens"));
    }
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {

    let config = CONFIG.load(deps.storage)?;
    let user_unbonding = USER_UNBONDING.load(deps.storage, info.sender)?;

    let mut withdraw_amount = Uint128::zero();

    let mut remaining_unbondings = vec![];

    for unbonding in user_unbonding {
        if env.block.time.seconds() >= unbonding.complete {
            withdraw_amount += unbonding.amount;
        }
        else {
            remaining_unbondings.push(&mut unbonding);
        }
    }

    USER_UNBONDING.save(deps.storage, info.sender, &remaining_unbondings)?;

    if withdraw_amount.is_zero() {
        return Err(StdError::generic_err("No completed unbondings"));
    }

    Ok(Response::new()
        .add_message(
            send_msg(
                info.sender,
                withdraw_amount,
                None,
                None,
                None,
                &config.stake_token,
            )
        )
        .set_data(to_binary(&ExecuteAnswer::Withdraw {
            success: ResponseStatus::Success,
        })?)
    )
}

pub fn compound(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    Ok()
}
