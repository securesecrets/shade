use cosmwasm_std::{
    debug_print,
    Api,
    Extern,
    Querier, StdResult, Storage,
};
use shade_protocol::{
    oracle::{
        QueryAnswer, 
    },
    band::{ 
        BandQuery, ReferenceData,
    },
    msg_traits::Query,
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

pub fn get_prices<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    symbols: Vec<String>,
) -> StdResult<Vec<ReferenceData>> {
    debug_print!("GET PRICES");

    let mut query_symbols: Vec<String> = Vec::new();
    let mut query_bases: Vec<String> = Vec::new();
    let mut results: Vec<ReferenceData> = Vec::new();
    debug_print!("START");

    //TODO: This data will be un-ordered relative to input, :it should be ordered
    for symbol in symbols {
        match hard_coded_r(&deps.storage).may_load(&symbol.as_bytes())? {
            Some(reference_data) => {
                debug_print!("HARD CODED");
                results.push(reference_data);
            }
            None => {
                debug_print!("HERE");
                query_bases.push("USD".to_string());
                if symbol == "SSCRT" {
                    query_symbols.push("SCRT".to_string());
                }
                else {
                    query_symbols.push(symbol);
                }
            }
        }
    }
    if query_symbols.len() > 0 {
        results.append(&mut reference_data_bulk(&deps, query_symbols, query_bases)?)
    }

    Ok(results)
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
        1,
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
        1,
        config_r.band.code_hash,
        config_r.band.address,
    )?)
}

