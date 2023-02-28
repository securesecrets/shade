use shade_protocol::{
    basic_staking::{QueryAnswer, Reward},
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

pub fn reward_pool(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::RewardPool {
        rewards: REWARD_POOLS.load(deps.storage)?,
    })
}

pub fn user_balance(deps: Deps, user: Addr) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Balance {
        amount: USER_STAKED
            .may_load(deps.storage, user)?
            .unwrap_or(Uint128::zero()),
    })
}

pub fn user_share(deps: Deps, user: Addr) -> StdResult<QueryAnswer> {
    let total_staked = TOTAL_STAKED.load(deps.storage)?;

    let user_staked = USER_STAKED
        .may_load(deps.storage, user)?
        .unwrap_or(Uint128::zero());

    let user_norm = Uint128::new(user_staked.u128() * 10u128.pow(18));
    let total_norm = Uint128::new(total_staked.u128() * 10u128.pow(18));

    Ok(QueryAnswer::Share {
        share: user_norm / total_norm,
    })
}

pub fn user_rewards(deps: Deps, env: Env, user: Addr) -> StdResult<QueryAnswer> {
    // let user_last_claim = USER_LAST_CLAIM.load(deps.storage, user.clone())?;
    if let Some(user_staked) = USER_STAKED.may_load(deps.storage, user.clone())? {
        if user_staked.is_zero() {
            return Ok(QueryAnswer::Rewards { rewards: vec![] });
        }
        let reward_pools = REWARD_POOLS.load(deps.storage)?;
        let total_staked = TOTAL_STAKED.load(deps.storage)?;
        let now = env.block.time.seconds();

        let mut rewards = vec![];

        for reward_pool in reward_pools {
            let user_reward_per_token_paid = USER_REWARD_PER_TOKEN_PAID
                .may_load(deps.storage, user_pool_key(user.clone(), reward_pool.id))?
                .unwrap_or(Uint128::zero());
            let reward_per_token = reward_per_token(total_staked, now, &reward_pool);
            rewards.push(Reward {
                token: reward_pool.token.address,
                amount: rewards_earned(user_staked, reward_per_token, user_reward_per_token_paid),
            });
        }

        Ok(QueryAnswer::Rewards { rewards })
    } else {
        Ok(QueryAnswer::Rewards { rewards: vec![] })
    }
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
