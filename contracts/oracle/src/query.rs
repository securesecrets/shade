use crate::state::{config_r, index_r, dex_pairs_r};
use cosmwasm_std::{Api, Extern, Querier, StdResult, Storage, Uint128, StdError};
use shade_protocol::{
    band,
    dex,
    oracle::{IndexElement, QueryAnswer},
};

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: config_r(&deps.storage).load()?,
    })
}

pub fn price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    symbol: String,
) -> StdResult<band::ReferenceData> {

    let config = config_r(&deps.storage).load()?;
    if symbol == "SSCRT" {
        return band::reference_data(deps, "SCRT".to_string(), "USD".to_string(), config.band);
    }

    if let Some(dex_pairs) = dex_pairs_r(&deps.storage).may_load(symbol.as_bytes())? {

        if dex_pairs.len() > 0 {
            
            return Ok(band::ReferenceData {
                rate: dex::aggregate_price(&deps, 
                                           dex_pairs, 
                                           config.sscrt, 
                                           config.band
                )?,
                last_updated_base: 0,
                last_updated_quote: 0,
            });
        }
    }

    // Index
    if let Some(index) = index_r(&deps.storage).may_load(symbol.as_bytes())? {
        return Ok(band::ReferenceData {
            rate: eval_index(deps, index)?,
            last_updated_base: 0,
            last_updated_quote: 0,
        });
    }

    // symbol/USD price from BAND
    band::reference_data(deps, symbol, "USD".to_string(), config.band)
}

pub fn prices<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    symbols: Vec<String>,
) -> StdResult<Vec<Uint128>> {
    let mut band_symbols = vec![];
    let mut band_quotes = vec![];
    let mut results = vec![Uint128(0); symbols.len()];

    let config = config_r(&deps.storage).load()?;

    for (i, sym) in symbols.iter().enumerate() {

        // Aggregate DEX pair prices
        if let Some(dex_pairs) = dex_pairs_r(&deps.storage).may_load(sym.as_bytes())? {
            if dex_pairs.len() > 0 {
                results[i] = dex::aggregate_price(&deps, dex_pairs, 
                                               config.sscrt.clone(), config.band.clone())?;
            }
        }

        // Index
        else if let Some(index) = index_r(&deps.storage).may_load(sym.as_bytes())? {

            results[i] = eval_index(deps, index)?;

        } 
        // BAND
        else {
            band_symbols.push(sym.clone());
            band_quotes.push("USD".to_string());
        }
    }

    // Query all the band prices
    let ref_data = band::reference_data_bulk(deps, band_symbols.clone(), band_quotes, config_r(&deps.storage).load()?.band)?;

    for (data, sym) in ref_data.iter().zip(band_symbols.iter()) {
        let result_index = symbols
            .iter()
            .enumerate()
            .find(|&s| *s.1 == *sym)
            .unwrap()
            .0;
        results[result_index] = data.rate;
    }

    Ok(results)
}

pub fn eval_index<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    index: Vec<IndexElement>,
) -> StdResult<Uint128> {

    let mut weight_sum = Uint128::zero();
    let mut price = Uint128::zero();

    let mut band_bases = vec![];
    let mut band_quotes = vec![];
    let mut band_weights = vec![];
    let config = config_r(&deps.storage).load()?;

    for element in index {
        weight_sum += element.weight;

        // Get dex prices
        if let Some(dex_pairs) = dex_pairs_r(&deps.storage).may_load(element.symbol.as_bytes())? {

            //return Err(StdError::generic_err(format!("EVAL INDEX DEX PAIRS {}", element.symbol)));
            price += dex::aggregate_price(deps, dex_pairs, config.sscrt.clone(), config.band.clone())?.multiply_ratio(element.weight, 10u128.pow(18));
            //return Err(StdError::generic_err(format!("EVAL INDEX DEX PAIRS {}", element.symbol)));
        }

        // Nested index 
        else if let Some(sub_index) = index_r(&deps.storage).may_load(element.symbol.as_bytes())? {
            // TODO: make sure no circular deps
            return Err(StdError::generic_err(format!("EVAL NESTED INDEX {}", element.symbol)));
            price += eval_index(&deps, sub_index)?.multiply_ratio(element.weight, 10u128.pow(18));

        }
        // Setup to query for all at once from BAND
        else {
            //return Err(StdError::generic_err(format!("EVAL INDEX BAND {}", element.symbol)));
            band_weights.push(element.weight);
            band_bases.push(element.symbol.clone());
            band_quotes.push("USD".to_string());
        }
    }

    if band_bases.len() > 0 {
        let ref_data = band::reference_data_bulk(deps, band_bases, band_quotes, config_r(&deps.storage).load()?.band)?;

        for (reference, weight) in ref_data.iter().zip(band_weights.iter()) {
            price += reference.rate.multiply_ratio(*weight, 10u128.pow(18));
        }
    }
    //return Err(StdError::generic_err(format!("Price {}", price)));

    Ok(price.multiply_ratio(10u128.pow(18), weight_sum.u128()))
    //Ok(price.multiply_ratio(1u128, weight_total.u128()))
}
