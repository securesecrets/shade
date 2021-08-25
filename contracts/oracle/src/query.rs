use cosmwasm_std::{
    Api,
    Extern,
    Querier, StdResult, Storage,
};
use shade_protocol::{
    oracle::{
        QueryMsg, QueryAnswer, 
    },
    band::{ 
        BandQuery, ReferenceData,
    },
    secret_swap::{
        SecretSwapQuery,
        SimulationResponse,
        Simulation,
        OfferAsset,
        AssetInfo,
        Token,
    },
    msg_traits::Query,
};

use crate::state::{
    config_r,
    hard_coded_r,
    sswap_assets_r,
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

    let mut query_symbols: Vec<String> = Vec::new();
    let mut query_bases: Vec<String> = Vec::new();
    let mut results: Vec<ReferenceData> = Vec::new();

    //TODO: This data will be un-ordered relative to input, :it should be ordered
    for symbol in symbols {
        match hard_coded_r(&deps.storage).may_load(&symbol.as_bytes())? {
            Some(reference_data) => {
                results.push(reference_data);
            }
            None => { 
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
    results.append(&mut reference_data_bulk(&deps, query_symbols, query_bases)?);

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

// Secret Swap price in */SCRT
/*
pub fn secret_swap_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    symbol: String,
)-> StdResult<SimulationResponse>
{
    //TODO: 
    assets_r
    Ok(Simulation {
        offer_asset: OfferAsset {
            amount: 1,
            info: AssetInfo {
                token: Token {
                    "addr".to_string(),
                    "efbaf03ba2f8b21c231874fd8f9f1c69203f585cae481691812d8289916eff7a".to_string(),
                    "SecretSwap".to_string(),
                }
            }
        }
    }.query(
        &deps.querier,
        1,

    )?)
}
*/
