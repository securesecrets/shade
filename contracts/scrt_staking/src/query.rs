use cosmwasm_std::{
    Api, BalanceResponse, BankQuery, Delegation, DistQuery, Extern, FullDelegation, HumanAddr,
    Querier, RewardsResponse, StdError, StdResult, Storage, Uint128,
};

use shade_protocol::scrt_staking::QueryAnswer;

use crate::state::{config_r, self_address_r};

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: config_r(&deps.storage).load()?,
    })
}

pub fn delegations<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Vec<Delegation>> {
    deps.querier
        .query_all_delegations(self_address_r(&deps.storage).load()?)
}

pub fn rewards<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Uint128> {

    let scrt_balance: BalanceResponse = deps.querier.query(
        &BankQuery::Balance {
            address: self_address_r(&deps.storage).load()?,
            denom: "uscrt".to_string(),
        }
        .into(),
    )?;

    let query_rewards: RewardsResponse = deps.querier
        .query(
            &DistQuery::Rewards {
                delegator: self_address_r(&deps.storage).load()?,
            }
            .into(),
        )
        .unwrap_or_else(|_| RewardsResponse {
            rewards: vec![],
            total: vec![],
        });

    if query_rewards.total.is_empty() {
        return Ok(scrt_balance.amount.amount);
    }

    let denom = query_rewards.total[0].denom.as_str();
    query_rewards.total.iter().fold(Ok(Uint128::zero()), |racc, d| {
        let acc = racc?;
        if d.denom.as_str() != denom {
            Err(StdError::generic_err(format!(
                "different denoms in bonds: '{}' vs '{}'",
                denom, &d.denom
            )))
        } else {
            Ok(acc + d.amount + scrt_balance.amount.amount)
        }
    })
}

// This won't work until cosmwasm 0.16ish
/*
pub fn delegation<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    validator: HumanAddr,
) -> StdResult<Option<FullDelegation>> {
    let address = self_address_r(&deps.storage).load()?;
    deps.querier.query_delegation(address, validator)
}
*/
