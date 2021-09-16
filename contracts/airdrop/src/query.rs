use cosmwasm_std::{Api, Extern, Querier, StdError, StdResult, Storage, HumanAddr, Uint128};
use shade_protocol::airdrop::{QueryAnswer};
use crate::{state::config_r,
            handle::calculate_airdrop };

pub fn config<S: Storage, A: Api, Q: Querier>
(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config { config: config_r(&deps.storage).load()? })
}

pub fn dates<S: Storage, A: Api, Q: Querier>
(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    let config = config_r(&deps.storage).load()?;
    Ok(QueryAnswer::Dates { start: config.start_date, end: config.end_date })
}

pub fn airdrop_amount<S: Storage, A: Api, Q: Querier>
(deps: &Extern<S, A, Q>, address: HumanAddr) -> StdResult<QueryAnswer> {
    let mut total: Uint128;

    match calculate_airdrop(&deps, address) {
        Ok(amount) => total = amount,
        Err(_) => total = Uint128(0),
    };

    Ok(QueryAnswer::Eligibility { amount: total })
}