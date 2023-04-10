use shade_protocol::{
    basic_staking::{QueryAnswer, Reward, RewardPool, RewardPoolInternal, StakingInfo},
    c_std::{Addr, Deps, Env, StdError, StdResult, Uint128},
};

use crate::{
    execute::{reward_per_token, rewards_earned},
    storage::*,
};

pub fn config(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: CONFIG.load(deps.storage)?,
    })
}

pub fn stake_token(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::StakeToken {
        token: STAKE_TOKEN.load(deps.storage)?.address,
    })
}

pub fn staking_info(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::StakingInfo {
        info: StakingInfo {
            stake_token: STAKE_TOKEN.load(deps.storage)?.address,
            total_staked: TOTAL_STAKED.load(deps.storage)?,
            unbond_period: CONFIG.load(deps.storage)?.unbond_period,
            reward_pools: REWARD_POOLS
                .load(deps.storage)?
                .into_iter()
                .map(
                    |RewardPoolInternal {
                         id,
                         amount,
                         start,
                         end,
                         token,
                         rate,
                         official,
                         ..
                     }| RewardPool {
                        id,
                        amount,
                        start,
                        end,
                        token,
                        rate,
                        official,
                    },
                )
                .collect(),
        },
    })
}

pub fn total_staked(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::TotalStaked {
        amount: TOTAL_STAKED.load(deps.storage)?,
    })
}

pub fn reward_tokens(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::RewardTokens {
        tokens: REWARD_TOKENS
            .load(deps.storage)?
            .iter()
            .map(|contract| contract.address.clone())
            .collect(),
    })
}

pub fn reward_pools(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::RewardPools {
        rewards: REWARD_POOLS
            .load(deps.storage)?
            .into_iter()
            .map(
                |RewardPoolInternal {
                     id,
                     amount,
                     start,
                     end,
                     token,
                     rate,
                     official,
                     ..
                 }| RewardPool {
                    id,
                    amount,
                    start,
                    end,
                    token,
                    rate,
                    official,
                },
            )
            .collect(),
    })
}

pub fn user_balance(
    deps: Deps,
    env: Env,
    user: Addr,
    unbonding_ids: Vec<Uint128>,
) -> StdResult<QueryAnswer> {
    let mut unbondings = vec![];

    for unbonding_id in unbonding_ids.iter() {
        if let Some(unbonding) = USER_UNBONDING.may_load(
            deps.storage,
            user_unbonding_key(user.clone(), *unbonding_id),
        )? {
            unbondings.push(unbonding);
        } else {
            return Err(StdError::generic_err(format!(
                "Bad Unbonding ID {}",
                unbonding_id
            )));
        }
    }

    let mut rewards = vec![];

    if let Some(user_staked) = USER_STAKED.may_load(deps.storage, user.clone())? {
        if user_staked.is_zero() {
            return Ok(QueryAnswer::Balance {
                staked: user_staked,
                rewards,
                unbondings,
            });
        }
        let reward_pools = REWARD_POOLS.load(deps.storage)?;
        let total_staked = TOTAL_STAKED.load(deps.storage)?;
        let now = env.block.time.seconds();

        for reward_pool in reward_pools {
            let user_reward_per_token_paid = USER_REWARD_PER_TOKEN_PAID
                .may_load(deps.storage, user_pool_key(user.clone(), reward_pool.id))?
                .unwrap_or(Uint128::zero());
            let reward_per_token = reward_per_token(total_staked, now, &reward_pool);
            let rewards_earned =
                rewards_earned(user_staked, reward_per_token, user_reward_per_token_paid);
            if !rewards_earned.is_zero() {
                rewards.push(Reward {
                    token: reward_pool.token,
                    amount: rewards_earned,
                });
            }
        }

        Ok(QueryAnswer::Balance {
            staked: user_staked,
            rewards,
            unbondings,
        })
    } else {
        Ok(QueryAnswer::Balance {
            staked: Uint128::zero(),
            rewards,
            unbondings,
        })
    }
}

pub fn user_staked(deps: Deps, user: Addr) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Staked {
        amount: USER_STAKED
            .may_load(deps.storage, user)?
            .unwrap_or(Uint128::zero()),
    })
}

pub fn user_rewards(deps: Deps, env: Env, user: Addr) -> StdResult<QueryAnswer> {
    let mut rewards = vec![];

    if let Some(user_staked) = USER_STAKED.may_load(deps.storage, user.clone())? {
        if user_staked.is_zero() {
            return Ok(QueryAnswer::Rewards { rewards });
        }
        let reward_pools = REWARD_POOLS.load(deps.storage)?;
        let total_staked = TOTAL_STAKED.load(deps.storage)?;
        let now = env.block.time.seconds();

        for reward_pool in reward_pools {
            let user_reward_per_token_paid = USER_REWARD_PER_TOKEN_PAID
                .may_load(deps.storage, user_pool_key(user.clone(), reward_pool.id))?
                .unwrap_or(Uint128::zero());
            let reward_per_token = reward_per_token(total_staked, now, &reward_pool);
            rewards.push(Reward {
                token: reward_pool.token,
                amount: rewards_earned(user_staked, reward_per_token, user_reward_per_token_paid),
            });
        }
    }

    Ok(QueryAnswer::Rewards { rewards })
}

pub fn user_unbondings(deps: Deps, user: Addr, ids: Vec<Uint128>) -> StdResult<QueryAnswer> {
    let mut unbondings = vec![];

    for id in ids.iter() {
        if let Some(unbonding) =
            USER_UNBONDING.may_load(deps.storage, user_unbonding_key(user.clone(), *id))?
        {
            unbondings.push(unbonding);
        } else {
            return Err(StdError::generic_err(format!("Bad ID {}", id)));
        }
    }

    Ok(QueryAnswer::Unbonding { unbondings })
}
