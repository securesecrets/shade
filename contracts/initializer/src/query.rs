use crate::state::{config_r, shade_r, silk_r};
use cosmwasm_std::{Api, Extern, Querier, StdResult, Storage};
use shade_protocol::initializer::QueryAnswer;

pub fn contracts<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Contracts {
        shade: shade_r(&deps.storage).load()?,
        silk: silk_r(&deps.storage).may_load()?
    })
}

pub fn config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: config_r(&deps.storage).load()?
    })
}
