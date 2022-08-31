use super::execute::debt_limit;
use crate::storage::*;
use shade_protocol::{
    c_std::{Deps, StdResult},
    contract_interfaces::mint::liability_mint::QueryAnswer,
};

pub fn config(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: CONFIG.load(deps.storage)?,
    })
}
pub fn token(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Token {
        token: TOKEN.load(deps.storage)?,
    })
}

pub fn liabilities(deps: Deps) -> StdResult<QueryAnswer> {
    let config = CONFIG.load(deps.storage)?;
    let limit = debt_limit(
        deps,
        TOKEN.load(deps.storage)?,
        COLLATERAL.load(deps.storage)?,
        config.debt_ratio,
        config.oracle,
        config.treasury,
    )?;
    Ok(QueryAnswer::Liabilities {
        outstanding: LIABILITIES.load(deps.storage)?,
        limit,
    })
}

pub fn whitelist(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Whitelist {
        whitelist: WHITELIST.load(deps.storage)?,
    })
}
