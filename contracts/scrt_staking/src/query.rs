use shade_protocol::c_std::{
    Api,
    BalanceResponse,
    BankQuery,
    Delegation,
    DistQuery,
    DepsMut,
    FullDelegation,
    Addr,
    Querier,
    RewardsResponse,
    StdError,
    StdResult,
    Storage,
    Uint128,
};

use shade_protocol::{
    contract_interfaces::dao::{adapter, scrt_staking::QueryAnswer},
    utils::asset::scrt_balance,
};

use crate::state::{config_r, self_address_r, unbonding_r};

pub fn config(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: config_r(deps.storage).load()?,
    })
}

pub fn delegations(
    deps: Deps,
) -> StdResult<Vec<Delegation>> {
    deps.querier
        .query_all_delegations(self_address_r(deps.storage).load()?)
}

pub fn rewards(deps: Deps) -> StdResult<Uint128> {
    let query_rewards: RewardsResponse = deps
        .querier
        .query(
            &DistQuery::Rewards {
                delegator: self_address_r(deps.storage).load()?,
            }
            .into(),
        )
        .unwrap_or_else(|_| RewardsResponse {
            rewards: vec![],
            total: vec![],
        });

    if query_rewards.total.is_empty() {
        return Ok(Uint128::zero());
    }

    let denom = query_rewards.total[0].denom.as_str();
    query_rewards
        .total
        .iter()
        .fold(Ok(Uint128::zero()), |racc, d| {
            let acc = racc?;
            if d.denom.as_str() != denom {
                Err(StdError::generic_err(format!(
                    "different denoms in bonds: '{}' vs '{}'",
                    denom, &d.denom
                )))
            } else {
                Ok(acc + d.amount)
            }
        })
}

pub fn balance(
    deps: Deps,
    asset: Addr,
) -> StdResult<adapter::QueryAnswer> {
    let config = config_r(deps.storage).load()?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        )));
    }

    let delegated = Uint128::new(
        delegations(deps)?
            .into_iter()
            .map(|d| d.amount.amount.u128())
            .sum::<u128>(),
    );

    let rewards = rewards(deps)?;

    Ok(adapter::QueryAnswer::Balance {
        amount: delegated + rewards,
    })
}

pub fn claimable(
    deps: Deps,
    asset: Addr,
) -> StdResult<adapter::QueryAnswer> {
    let config = config_r(deps.storage).load()?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        )));
    }

    let scrt_balance: BalanceResponse = deps.querier.query(
        &BankQuery::Balance {
            address: self_address_r(deps.storage).load()?,
            denom: "uscrt".to_string(),
        }
        .into(),
    )?;

    let mut amount = scrt_balance.amount.amount;
    let unbonding = unbonding_r(deps.storage).load()?;

    if amount > unbonding {
        amount = unbonding;
    }

    Ok(adapter::QueryAnswer::Claimable { amount })
}

pub fn unbonding(
    deps: Deps,
    asset: Addr,
) -> StdResult<adapter::QueryAnswer> {
    let config = config_r(deps.storage).load()?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        )));
    }

    Ok(adapter::QueryAnswer::Unbonding {
        amount: unbonding_r(deps.storage).load()?,
    })
}

pub fn unbondable(
    deps: Deps,
    asset: Addr,
) -> StdResult<adapter::QueryAnswer> {
    let config = config_r(deps.storage).load()?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err(format!(
            "Unrecognized Asset {}",
            asset
        )));
    }

    let unbondable = match balance(deps, asset)? {
        adapter::QueryAnswer::Balance { amount } => amount,
        _ => {
            return Err(StdError::generic_err("Failed to query balance"));
        }
    };

    /*TODO: Query current unbondings
     * u >= 7 = 0
     * u <  7 = unbondable
     */
    Ok(adapter::QueryAnswer::Unbondable { amount: unbondable })
}

pub fn reserves(
    deps: Deps,
    asset: Addr,
) -> StdResult<adapter::QueryAnswer> {

    let config = config_r(deps.storage).load()?;

    if asset != config.sscrt.address {
        return Err(StdError::generic_err(format!("Unrecognized Asset {}", asset)));
    }

    let scrt_balance = scrt_balance(deps, self_address_r(deps.storage).load()?)?;

    Ok(adapter::QueryAnswer::Reserves {
        amount: scrt_balance + rewards(&deps)?,
    })
}

// This won't work until cosmwasm 0.16
/*
pub fn delegation(
    deps: Deps,
    validator: Addr,
) -> StdResult<Option<FullDelegation>> {
    let address = self_address_r(deps.storage).load()?;
    deps.querier.query_delegation(address, validator)
}
*/
