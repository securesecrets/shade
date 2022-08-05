use crate::{
    storage::*,
};
use chrono::prelude::*;
use shade_protocol::c_std::{Deps, Uint128};
use shade_protocol::c_std::{Api, DepsMut, Addr, Querier, StdError, StdResult, Storage};
use shade_protocol::contract_interfaces::mint::liability_mint::QueryAnswer;

pub fn config(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: CONFIG.load(deps.storage)?,
    })
}
pub fn token(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Token {
        token: TOKEN.load(deps.storage)?
    })
}

pub fn liabilities(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Liabilities {
        outstanding: LIABILITIES.load(deps.storage)?,
        //TODO: implement limit
        limit: CONFIG.load(deps.storage)?.limit,
    })
}

pub fn whitelist(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Whitelist {
        whitelist: WHITELIST.load(deps.storage)?,
    })
}
