use cosmwasm_std::{Api, Extern, Querier, StdResult, Storage, HumanAddr};
use shade_protocol::airdrop::{QueryAnswer};
use crate::{state::{config_r, reward_r}};
use crate::state::claim_status_r;

pub fn config<S: Storage, A: Api, Q: Querier>
(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config { config: config_r(&deps.storage).load()?
    })
}

pub fn dates<S: Storage, A: Api, Q: Querier>
(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    let config = config_r(&deps.storage).load()?;
    Ok(QueryAnswer::Dates { start: config.start_date, end: config.end_date
    })
}

pub fn airdrop_amount<S: Storage, A: Api, Q: Querier>
(deps: &Extern<S, A, Q>, address: HumanAddr) -> StdResult<QueryAnswer> {
    let key = address.to_string();
    Ok(QueryAnswer::Eligibility {
        amount: reward_r(&deps.storage).load(key.as_bytes())?.amount,
        claimed: claim_status_r(&deps.storage).load(key.as_bytes())?
    })
}
