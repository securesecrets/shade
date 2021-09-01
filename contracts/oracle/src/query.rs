use cosmwasm_std::{
    debug_print,
    Api,
    Extern,
    Querier, StdResult, Storage,
};
use secret_toolkit::utils::Query;
use shade_protocol::{
    oracle::{QueryAnswer},
    band::{BandQuery, ReferenceData},
};
use crate::state::{
    config_r,
    hard_coded_r,
};

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config { config: config_r(&deps.storage).load()? })
}

pub fn get_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    symbol: String,
) -> StdResult<ReferenceData> {
    match hard_coded_r(&deps.storage).may_load(&symbol.as_bytes())? {
        Some(reference_data) => {
            return Ok(reference_data);
        }
        None => {}
    }

    if symbol == "SSCRT" {
        return Ok(reference_data(deps, "SCRT".to_string(), "USD".to_string())?)
    }

    Ok(reference_data(deps, symbol, "USD".to_string())?)
}

// BAND interactions
pub fn reference_data<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    base_symbol: String,
    quote_symbol: String,
) -> StdResult<ReferenceData> {

    let config_r = config_r(&deps.storage).load()?;

    Ok(BandQuery::GetReferenceData {
            base_symbol,
            quote_symbol,
    }.query(
        &deps.querier,
        config_r.band.code_hash,
        config_r.band.address,
    )?)
}

pub fn reference_data_bulk<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    base_symbols: Vec<String>,
    quote_symbols: Vec<String>,
) -> StdResult<Vec<ReferenceData>> {

    let config_r = config_r(&deps.storage).load()?;

    Ok(BandQuery::GetReferenceDataBulk {
        base_symbols,
        quote_symbols,
    }.query(
        &deps.querier,
        config_r.band.code_hash,
        config_r.band.address,
    )?)
}

