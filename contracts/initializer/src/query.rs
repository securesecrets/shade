use cosmwasm_std::{Storage, Api, Querier, Extern, StdResult};
use shade_protocol::initializer::QueryAnswer;
use crate::state::config_r;

pub fn query_contracts<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Contracts { contracts: config_r(&deps.storage).load()?.contracts })
}
