use cosmwasm_std::{
    Api, Extern, Querier, Storage, StdResult,
};
use secret_toolkit::snip20;
/*
use secret_toolkit::snip20::{
    balance_query, Balance, BalanceResponse,
};
*/
use shade_protocol::{
    treasury::{
        QueryAnswer
    },
    asset::Contract,
};

use crate::state::{
    config_r, 
    viewing_key_r,
    self_address_r,
};

pub fn config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::Config { config: config_r(&deps.storage).load()? })
}

pub fn balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract: Contract,
) -> StdResult<QueryAnswer> {

    //TODO: restrict to admin

    Ok(snip20::QueryMsg::Balance { 
        address: self_address_r(&deps.storage).load()?, 
        key: viewing_key_r(&deps.storage).load()?,
    }.query(
         &deps.querier,
         1,
         contract.code_hash.clone(), 
         contract.address.clone(),
     )?)
}
