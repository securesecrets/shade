use shade_protocol::c_std::{
    Deps,
    StdResult,
};

use shade_protocol::{
    contract_interfaces::dao::rewards_emission::QueryAnswer,
};




use crate::storage::*;

pub fn config(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: CONFIG.load(deps.storage)?,
    })
}

/*
pub fn pending_allowance(
    deps: Deps,
    asset: Addr,
) -> StdResult<QueryAnswer> {
    let token = TOKEN.load(deps.storage)?;
    let config = CONFIG.load(deps.storage)?;

    let allowance = allowance_query(
        &deps.querier,
        config.treasury,
        SELF_ADDRESS.load(deps.storage)?,
        VIEWING_KEY.load(deps.storage)?,
        &token.contract.clone(),
    )?
    .allowance;

    Ok(QueryAnswer::PendingAllowance { amount: allowance })
}

pub fn balance(
    deps: Deps,
    asset: Addr,
) -> StdResult<adapter::QueryAnswer> {
    let token = TOKEN.may_load(deps.storage)?;

    let balance = balance_query(
        &deps.querier,
        SELF_ADDRESS.load(deps.storage)?,
        VIEWING_KEY.load(deps.storage)?,
        &token.contract.clone(),
    )?
    .amount;

    Ok(adapter::QueryAnswer::Balance { amount: balance })
}
*/
