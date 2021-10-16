use cosmwasm_std::{Api, Extern, Querier, StdError, StdResult, Storage, Uint128};
use shade_protocol::{staking::{QueryMsg, QueryAnswer}, snip20};
use crate::{state::{config_r, total_staked_r}};

pub fn config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::Config {
        config: config_r(&deps.storage).load()?
    })
}

pub fn total_staked<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::TotalStaked {
        total: total_staked_r(&deps.storage).load()?,
    })
}