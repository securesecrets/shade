use cosmwasm_std::{Api, Extern, Querier, StdError, StdResult, Storage, Uint128};
use shade_protocol::{staking::{QueryMsg, QueryAnswer}, snip20};
use crate::{state::{config_r, stake_state_r}};

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