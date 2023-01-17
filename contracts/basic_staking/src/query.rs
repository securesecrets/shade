use shade_protocol::{
    basic_staking::QueryAnswer,
    c_std::{Addr, Deps, Env, StdResult, Uint128},
};

use crate::storage::*;

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
            .map(|contract| contract.address)
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
        amount: USER_STAKED.load(deps.storage, user)?,
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
    // TODO implement rewards calc
    Ok(QueryAnswer::Rewards {
        amount: Uint128::zero(),
    })
}

pub fn user_unbonding(deps: Deps, env: Env, user: Addr) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Unbonding {
        unbondings: USER_UNBONDINGS.load(deps.storage, user)?,
    })
}
