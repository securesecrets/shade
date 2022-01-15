use cosmwasm_std::{
    Api, Extern, Querier, Storage,
    StdResult, StdError, HumanAddr,
};
use secret_toolkit::snip20;
use shade_protocol::{
    treasury::{
        QueryAnswer
    },
};

use crate::state::{
    config_r, 
    viewing_key_r,
    self_address_r,
    assets_r,
};

pub fn config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::Config { config: config_r(&deps.storage).load()? })
}

pub fn balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract: HumanAddr,
) -> StdResult<QueryAnswer> {

    //TODO: restrict to admin

    return match assets_r(&deps.storage).may_load(contract.to_string().as_bytes())? {
        Some(a) => {
            Ok(snip20::QueryMsg::Balance {
                address: self_address_r(&deps.storage).load()?,
                key: viewing_key_r(&deps.storage).load()?,
            }.query(
                &deps.querier,
                1,
                a.contract.code_hash,
                contract,
            )?)
        }
        None => {
            Err(StdError::not_found(contract.to_string()))
        }
    };

}

pub fn can_rebalance<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::CanRebalance {
        possible: false,
    })
}
