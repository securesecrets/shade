use crate::{
    msg::QueryAnswer,
    stake::{calculate_rewards, shares_per_token},
    state::ReadonlyBalances,
    state_staking::{
        DailyUnbondingQueue,
        TotalShares,
        TotalTokens,
        TotalUnbonding,
        UnbondingQueue,
        UserCooldown,
        UserShares,
    },
};
use shade_protocol::c_std::Uint128;
use shade_protocol::c_std::{to_binary, Api, Binary, DepsMut, Addr, Querier, StdResult, Storage};
use shade_protocol::{
    contract_interfaces::staking::snip20_staking::stake::{StakeConfig, VecQueue},
    utils::storage::default::{BucketStorage, SingletonStorage},
};

pub fn stake_config(deps: Deps) -> StdResult<Binary> {
    to_binary(&QueryAnswer::StakedConfig {
        config: StakeConfig::load(deps.storage)?,
    })
}

pub fn total_staked(deps: Deps) -> StdResult<Binary> {
    to_binary(&QueryAnswer::TotalStaked {
        tokens: TotalTokens::load(deps.storage)?.0,
        shares: TotalShares::load(deps.storage)?.0,
    })
}

pub fn stake_rate(deps: Deps) -> StdResult<Binary> {
    to_binary(&QueryAnswer::StakeRate {
        shares: shares_per_token(
            &StakeConfig::load(deps.storage)?,
            &Uint128::new(1),
            &TotalTokens::load(deps.storage)?.0,
            &TotalShares::load(deps.storage)?.0,
        )?,
    })
}

pub fn unfunded(
    deps: Deps,
    start: u64,
    total: u64,
) -> StdResult<Binary> {
    let mut total_bonded = Uint128::zero();

    let queue = DailyUnbondingQueue::load(deps.storage)?.0;

    let mut count = 0;
    for item in queue.0.iter() {
        if item.release >= start {
            if count >= total {
                break;
            }
            total_bonded += item.unbonding.checked_sub(item.funded)?;
            count += 1;
        }
    }

    to_binary(&QueryAnswer::Unfunded {
        total: total_bonded,
    })
}

pub fn unbonding(deps: Deps) -> StdResult<Binary> {
    to_binary(&QueryAnswer::Unbonding {
        total: TotalUnbonding::load(deps.storage)?.0,
    })
}

pub fn staked(
    deps: Deps,
    account: Addr,
    time: Option<u64>,
) -> StdResult<Binary> {
    let tokens = ReadonlyBalances::from_storage(deps.storage)
        .account_amount(&deps.api.canonical_address(&account)?);

    let shares = UserShares::load(deps.storage, account.as_str().as_bytes())?.0;

    let (rewards, _) = calculate_rewards(
        &StakeConfig::load(deps.storage)?,
        Uint128::new(tokens),
        shares,
        TotalTokens::load(deps.storage)?.0,
        TotalShares::load(deps.storage)?.0,
    )?;

    let queue = UnbondingQueue::may_load(deps.storage, account.as_str().as_bytes())?
        .unwrap_or_else(|| UnbondingQueue(VecQueue::new(vec![])));

    let mut unbonding = Uint128::zero();
    let mut unbonded = Uint128::zero();

    for item in queue.0.0.iter() {
        if let Some(time) = time {
            if item.release <= time {
                unbonded += item.amount;
            } else {
                unbonding += item.amount;
            }
        } else {
            unbonding += item.amount;
        }
    }

    to_binary(&QueryAnswer::Staked {
        tokens: Uint128::new(tokens),
        shares,
        pending_rewards: rewards,
        unbonding,
        unbonded: time.map(|_| unbonded),
        cooldown: UserCooldown::may_load(deps.storage, account.as_str().as_bytes())?
            .unwrap_or(UserCooldown {
                total: Default::default(),
                queue: VecQueue(vec![]),
            })
            .queue,
    })
}
