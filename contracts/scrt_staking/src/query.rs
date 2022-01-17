use cosmwasm_std::{
    Api,
    Delegation,
    Extern,
    FullDelegation,
    HumanAddr,
    Querier,
    StdResult,
    Storage,
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
    let address = self_address_r(&deps.storage).load()?;
    deps.querier.query_all_delegations(address)
}

pub fn delegation<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    validator: HumanAddr,
) -> StdResult<Option<FullDelegation>> {
    let address = self_address_r(&deps.storage).load()?;
    deps.querier.query_delegation(address, validator)
}
