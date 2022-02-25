use crate::state::{config_r, index_r, sswap_pairs_r, sienna_pairs_r};
use cosmwasm_std::{Api, Extern, Querier, StdResult, Storage, Uint128};
use shade_protocol::{
    band,
    dex,
    oracle::{IndexElement, QueryAnswer},
    secretswap,
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


    // Find registered paired for this asset & sSCRT
    let mut dex_pairs = vec![];
    if let Some(sswap_pair) = sswap_pairs_r(&deps.storage).may_load(symbol.as_bytes())? {
        dex_pairs.push(sswap_pair);
    }

    if let Some(sienna_pair) = sienna_pairs_r(&deps.storage).may_load(symbol.as_bytes())? {
        dex_pairs.push(sienna_pair);
    }

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

    // Index
    if let Some(index) = index_r(&deps.storage).may_load(symbol.as_bytes())? {
        return Ok(band::ReferenceData {
            rate: eval_index(deps, &symbol, index)?,
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

        let mut dex_pairs = vec![];

        if let Some(sswap_pair) = sswap_pairs_r(&deps.storage).may_load(sym.as_bytes())? {
            dex_pairs.push(sswap_pair);
        }
        if let Some(sienna_pair) = sienna_pairs_r(&deps.storage).may_load(sym.as_bytes())? {
            dex_pairs.push(sienna_pair);
        }

        // Aggregate DEX pair prices
        if dex_pairs.len() > 0 {
            results[i] = dex::aggregate_price(&deps, dex_pairs, 
                                           config.sscrt.clone(), config.band.clone())?;
        }
        // Index
        else if let Some(index) = index_r(&deps.storage).may_load(sym.as_bytes())? {

            results[i] = eval_index(deps, sym, index)?;

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
    symbol: &str,
    index: Vec<IndexElement>,
) -> StdResult<Uint128> {
    let mut weight_total = Uint128::zero();
    let mut price = Uint128::zero();

    let mut band_bases = vec![];
    let mut band_quotes = vec![];
    let mut band_weights = vec![];
    let config = config_r(&deps.storage).load()?;

    for element in index {
        weight_total += element.weight;

        if let Some(sswap_pair) = sswap_pairs_r(&deps.storage).may_load(symbol.as_bytes())? {
            price += secretswap::price(deps, sswap_pair, config.sscrt.clone(), config.band.clone())?.multiply_ratio(element.weight, 10u128.pow(18));
        } else {
            band_weights.push(element.weight);
            band_bases.push(element.symbol.clone());
            band_quotes.push("USD".to_string());
        }
    }

    let ref_data = band::reference_data_bulk(deps, band_bases, band_quotes, config_r(&deps.storage).load()?.band)?;

    for (reference, weight) in ref_data.iter().zip(band_weights.iter()) {
        price += reference.rate.multiply_ratio(*weight, 10u128.pow(18));
    }

    Ok(price.multiply_ratio(10u128.pow(18), weight_total))
}
