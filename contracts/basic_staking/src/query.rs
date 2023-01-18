use shade_protocol::{
    basic_staking::QueryAnswer,
    c_std::{Addr, Deps, Env, StdResult, Uint128},
};

use crate::{
    execute::{reward_per_token, rewards_earned},
    storage::*,
};
use std::cmp::max;

pub fn config(deps: Deps, env: Env) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: CONFIG.load(deps.storage)?,
    })
}

pub fn total_staked(deps: Deps, env: Env) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::TotalStaked {
        amount: TOTAL_STAKED.load(deps.storage)?,
    })
}

pub fn reward_tokens(deps: Deps, env: Env) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::RewardTokens {
        tokens: REWARD_TOKENS
            .load(deps.storage)?
            .iter()
            .map(|contract| contract.address.clone())
            .collect(),
    })
}

pub fn reward_pool(deps: Deps, env: Env) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::RewardPool {
        rewards: REWARD_POOLS.load(deps.storage)?,
    })
}

pub fn user_balance(deps: Deps, env: Env, user: Addr) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Balance {
        amount: USER_STAKED
            .may_load(deps.storage, user)?
            .unwrap_or(Uint128::zero()),
    })
}

pub fn user_share(deps: Deps, env: Env, user: Addr) -> StdResult<QueryAnswer> {
    let total_staked = TOTAL_STAKED.load(deps.storage)?;

    let user_staked = USER_STAKED.load(deps.storage, user)?;

    let user_norm = Uint128::new(user_staked.u128() * 10u128.pow(18));
    let total_norm = Uint128::new(total_staked.u128() * 10u128.pow(18));

    Ok(QueryAnswer::Share {
        share: user_norm / total_norm,
    })
}

pub fn user_rewards(deps: Deps, env: Env, user: Addr) -> StdResult<QueryAnswer> {
    // let user_last_claim = USER_LAST_CLAIM.load(deps.storage, user.clone())?;
    let user_staked = USER_STAKED.load(deps.storage, user.clone())?;
    let reward_pools = REWARD_POOLS.load(deps.storage)?;
    let total_staked = TOTAL_STAKED.load(deps.storage)?;
    let now = env.block.time.seconds();

    let mut reward_sum = Uint128::zero();

    // TODO CRITICAL differentiate different denoms
    for reward_pool in reward_pools {
        let user_reward_per_token_paid = USER_REWARD_PER_TOKEN_PAID
            .may_load(deps.storage, user.clone())?
            .unwrap_or(Uint128::zero());
        let reward_per_token = reward_per_token(total_staked, now, &reward_pool);
        let reward = rewards_earned(
            user_staked,
            reward_per_token,
            user_reward_per_token_paid,
            now,
            &reward_pool,
        );
        reward_sum += reward;
    }

    Ok(QueryAnswer::Rewards { amount: reward_sum })
}

pub fn user_unbonding(deps: Deps, env: Env, user: Addr) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Unbonding {
        unbondings: USER_UNBONDINGS.load(deps.storage, user)?,
    })
}
