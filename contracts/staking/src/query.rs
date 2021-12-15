use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage, Uint128};
use shade_protocol::staking::QueryAnswer;
use crate::{state::{config_r, stake_state_r}};
use crate::handle::calculate_rewards;
use crate::state::{staker_r, unbonding_r, user_unbonding_r, viewking_key_r};

pub fn config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::Config {
        config: config_r(&deps.storage).load()?
    })
}

pub fn total_staked<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::TotalStaked {
        total: stake_state_r(&deps.storage).load()?.total_tokens,
    })
}

pub fn total_unbonding<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>, start_limit: Option<u64>, end_limit: Option<u64>) -> StdResult<QueryAnswer> {

    let mut total = Uint128::zero();
    let mut queue = unbonding_r(&deps.storage).load()?;

    let start = start_limit.unwrap_or(0u64);

    let end = end_limit.unwrap_or(u64::MAX);

    while let Some(item) = queue.pop() {
        if start <= item.unbond_time && item.unbond_time <= end {
            total += item.amount;
        }
    }

    Ok(QueryAnswer::TotalUnbonding {
        total,
    })
}

pub fn user_stake<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>, address: HumanAddr, key: String, time: u64) -> StdResult<QueryAnswer> {

    if viewking_key_r(&deps.storage).load(address.to_string().as_bytes())? != key {
        return Err(StdError::Unauthorized { backtrace: None })
    }

    let state = stake_state_r(&deps.storage).load()?;
    let user_state = staker_r(&deps.storage).load(address.to_string().as_bytes())?;

    let mut unbonding = Uint128::zero();
    let mut unbonded = Uint128::zero();

    let queue = user_unbonding_r(&deps.storage).may_load(
        address.to_string().as_bytes())?;

    if let Some(mut queue) = queue {
        while !queue.is_empty() {
            let item = queue.pop().unwrap();

            if item.unbond_time > time {
                unbonding += item.amount;
            } else {
                unbonded += item.amount;
            }
        }
    }

    Ok(QueryAnswer::UserStake {
        staked: user_state.tokens_staked,
        pending_rewards: calculate_rewards(&user_state, &state),
        unbonding,
        unbonded
    })
}
