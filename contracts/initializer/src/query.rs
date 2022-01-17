use crate::state::config_r;
use cosmwasm_std::{Api, Extern, Querier, StdResult, Storage};
use shade_protocol::initializer::QueryAnswer;

pub fn query_contracts<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Contracts {
        contracts: config_r(&deps.storage).load()?.contracts,
    })
}
