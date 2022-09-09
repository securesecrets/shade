use crate::state::{config_r, dex_pairs_r, index_r};
use shade_protocol::c_std::{Uint128, Uint512};
use shade_protocol::c_std::{self, Api, DepsMut, Querier, StdError, StdResult, Storage};
use shade_protocol::contract_interfaces::{
    dex::dex,
    oracles::{
        band,
        oracle::{IndexElement, QueryAnswer},
    },
};
use std::convert::TryFrom;

pub fn config(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: config_r(deps.storage).load()?,
    })
}

pub fn price(
    deps: Deps,
    symbol: String,
) -> StdResult<band::ReferenceData> {
    let config = config_r(deps.storage).load()?;
    if symbol == "SSCRT" {
        return band::reference_data(deps, "SCRT".to_string(), "USD".to_string(), config.band);
    }

    if let Some(dex_pairs) = dex_pairs_r(deps.storage).may_load(symbol.as_bytes())? {
        if dex_pairs.len() > 0 {
            return Ok(band::ReferenceData {
                rate: dex::aggregate_price(&deps, dex_pairs, config.sscrt, config.band)?,
                last_updated_base: 0,
                last_updated_quote: 0,
            });
        }
    }

    // Index
    if let Some(index) = index_r(deps.storage).may_load(symbol.as_bytes())? {
        return Ok(band::ReferenceData {
            rate: eval_index(deps, index)?,
            last_updated_base: 0,
            last_updated_quote: 0,
        });
    }

    // symbol/USD price from BAND
    band::reference_data(deps, symbol, "USD".to_string(), config.band)
}

pub fn prices(
    deps: Deps,
    symbols: Vec<String>,
) -> StdResult<Vec<Uint128>> {
    let mut band_symbols = vec![];
    let mut band_quotes = vec![];
    let mut results = vec![Uint128::zero(); symbols.len()];

    let config = config_r(deps.storage).load()?;

    for (i, sym) in symbols.iter().enumerate() {
        // Aggregate DEX pair prices
        if let Some(dex_pairs) = dex_pairs_r(deps.storage).may_load(sym.as_bytes())? {
            if dex_pairs.len() > 0 {
                results[i] = dex::aggregate_price(
                    &deps,
                    dex_pairs,
                    config.sscrt.clone(),
                    config.band.clone(),
                )?;
            }
        }
        // Index
        else if let Some(index) = index_r(deps.storage).may_load(sym.as_bytes())? {
            results[i] = eval_index(deps, index)?;
        }
        // BAND
        else {
            band_symbols.push(sym.clone());
            band_quotes.push("USD".to_string());
        }
    }

    // Query all the band prices
    let ref_data = band::reference_data_bulk(
        deps,
        band_symbols.clone(),
        band_quotes,
        config_r(deps.storage).load()?.band,
    )?;

    for (data, sym) in ref_data.iter().zip(band_symbols.iter()) {
        let result_index = symbols
            .iter()
            .enumerate()
            .find(|&s| *s.1 == *sym)
            .unwrap()
            .0;
        results[result_index] = data.rate;
    }

    Ok(results
        .iter()
        .map(|r| Uint128::new(r.u128()))
        .collect())
}

pub fn eval_index(
    deps: Deps,
    index: Vec<IndexElement>,
) -> StdResult<Uint128> {
    let mut weight_sum = Uint512::zero();
    let mut price = Uint512::zero();

    let mut band_bases = vec![];
    let mut band_quotes = vec![];
    let mut band_weights = vec![];
    let config = config_r(deps.storage).load()?;

    for element in index {
        weight_sum += Uint512::from(element.weight.u128());

        // Get dex prices
        if let Some(dex_pairs) = dex_pairs_r(deps.storage).may_load(element.symbol.as_bytes())? {
            return Err(StdError::generic_err(format!(
                "EVAL INDEX DEX PAIRS {}",
                element.symbol
            )));

            // NOTE: unreachable?
            // price +=
            //     dex::aggregate_price(deps, dex_pairs, config.sscrt.clone(), config.band.clone())?
            //         .multiply_ratio(element.weight, 10u128.pow(18))
        }
        // Nested index
        else if let Some(sub_index) =
            index_r(deps.storage).may_load(element.symbol.as_bytes())?
        {
            // TODO: make sure no circular deps
            return Err(StdError::generic_err(format!(
                "EVAL NESTED INDEX {}",
                element.symbol
            )));
            // NOTE: unreachable?
            // price += eval_index(&deps, sub_index)?.multiply_ratio(element.weight, 10u128.pow(18))
        }
        // Setup to query for all at once from BAND
        else {
            band_weights.push(element.weight);
            band_bases.push(element.symbol.clone());
            band_quotes.push("USD".to_string());
        }
    }

    if band_bases.len() > 0 {
        let ref_data = band::reference_data_bulk(
            deps,
            band_bases,
            band_quotes,
            config_r(deps.storage).load()?.band,
        )?;

        for (reference, weight) in ref_data.iter().zip(band_weights.iter()) {
            price += Uint512::from(reference.rate.u128()) * Uint512::from(weight.u128())
                / Uint512::from(10u128.pow(18));
        }
    }

    Ok(Uint128::new(
        Uint128::try_from(
            price
                .checked_mul(Uint512::from(10u128.pow(18)))?
                .checked_div(weight_sum)?,
        )?
        .u128(),
    ))
}
