use shade_protocol::{
    c_std::{
        Addr,
        Api,
        BankQuery,
        Delegation,
        Deps,
        DistributionMsg,
        FullDelegation,
        Querier,
        StdError,
        StdResult,
        Storage,
        Uint128,
    },
    dao::{adapter, scrt_staking::QueryAnswer},
    utils::asset::scrt_balance,
};

use crate::storage::{CONFIG, SELF_ADDRESS, UNBONDING};

pub fn config(deps: Deps, env: Env, info: MessageInfo) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: CONFIG.load(deps.storage)?,
    })
}

pub fn total_staked(deps: Deps, env: Env, info: MessageInfo) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::TotalStaked {
        amount: TOTAL_STAKED.load(&deps)?,
    })
}

pub fn reward_tokens(deps: Deps, env: Env, info: MessageInfo) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::RewardTokens {
        tokens: REWARD_TOKENS.load(&deps)?,
    })
}

pub fn reward_pool(deps: Deps, env: Env, info: MessageInfo) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::RewardPool {
        rewards: REWARD_POOLS.load(deps)?,
    })
}

pub fn user_balance(deps: Deps, env: Env, info: MessageInfo) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Balance {
        amount: USER_STAKED.load(deps, info.sender)?,
    })
}

pub fn user_share(deps: Deps, env: Env, info: MessageInfo) -> StdResult<QueryAnswer> {
    let total_staked = TOTAL_STAKED.load(deps)?;

    let user_staked = USER_STAKED.load(deps)?;

    let user_norm = user_staked * 10u128.pow(18);
    let total_norm = total_staked * 10u128.pow(18);

    Ok(QueryAnswer::Share {
        share: user_norm / total_norm,
    })
}

pub fn user_rewards(deps: Deps, env: Env, info: MessageInfo) -> StdResult<QueryAnswer> {
    // TODO implement rewards calc
    Ok(QueryAnswer::Rewards {
        amount: Uint128::zero(),
    })
}

pub fn user_unbonding(deps: Deps, env: Env, info: MessageInfo) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Unbonding {
        amount: UNBONDING.load(deps)?,
    })
}
