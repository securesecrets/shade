use cosmwasm_std::{Storage, Api, Querier, Extern, StdResult, to_binary, Binary};
use shade_protocol::initializer::{QueryMsg, ContractsAnswer};
use crate::state::config_r;

pub fn query_contracts<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<ContractsAnswer> {
    Ok(ContractsAnswer { contracts: config_r(&deps.storage).load()?.contracts })
}