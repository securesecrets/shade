//#! 
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{StdResult, QuerierWrapper};
use crate::{
    Contract,
    BLOCK_SIZE,
    utils::{Query},
};
use std::collections::HashMap;

#[cw_serde]
#[derive(Default)]
pub struct OraclePrice {
    pub key: String,
    pub data: ReferenceData,
}

pub type PriceResponse = OraclePrice;
pub type PricesResponse = Vec<OraclePrice>;

#[cw_serde]
pub enum RouterQueryMsg {
    GetOracle { key: String },
    GetOracles { keys: Vec<String> },
}

impl Query for RouterQueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
pub enum OracleQueryMsg {
    GetPrice { key: String },
    GetPrices { keys: Vec<String> },
}

impl Query for OracleQueryMsg {
    const BLOCK_SIZE: usize = BLOCK_SIZE;
}

#[cw_serde]
pub struct OracleResponse {
    pub key: String,
    pub oracle: Contract,
}

/// Gets the oracle for the key from the router & calls GetPrice on it.
///
/// Has a query depth of 1.
pub fn query_price(
    router: &Contract,
    querier: &QuerierWrapper,
    key: String,
) -> StdResult<OraclePrice> {
    let oracle_resp: OracleResponse =
        RouterQueryMsg::GetOracle { key: key.clone() }.query(querier, router)?;
    query_oracle_price(&oracle_resp.oracle, querier, key)
}

/// Groups the keys by their respective oracles and sends bulk GetPrices queries to each of those oracles.
///
/// Done to reduce impact on query depth.
pub fn query_prices(
    router: &Contract,
    querier: &QuerierWrapper,
    keys: Vec<String>,
) -> StdResult<Vec<OraclePrice>> {
    let oracle_resps: Vec<OracleResponse> =
        RouterQueryMsg::GetOracles { keys }.query(querier, router)?;
    let mut map: HashMap<Contract, Vec<String>> = HashMap::new();
    let mut prices: Vec<OraclePrice> = vec![];

    for resp in oracle_resps {
        // Get the current vector of symbols at that oracle and add the current key to it
        map.entry(resp.oracle).or_insert(vec![]).push(resp.key);
    }

    for (oracle, keys) in map {
        if keys.len() == 1 {
            let queried_price = query_oracle_price(&oracle, querier, keys[0].clone())?;
            prices.push(queried_price);
        } else {
            let mut queried_prices = query_oracle_prices(&oracle, querier, keys)?;
            prices.append(&mut queried_prices);
        }
    }
    Ok(prices)
}

pub fn query_oracle_price(
    oracle: &Contract,
    querier: &QuerierWrapper,
    key: String,
) -> StdResult<OraclePrice> {
    let resp: PriceResponse = OracleQueryMsg::GetPrice { key }.query(querier, oracle)?;
    Ok(resp.price)
}

pub fn query_oracle_prices(
    oracle: &Contract,
    querier: &QuerierWrapper,
    keys: Vec<String>,
) -> StdResult<Vec<OraclePrice>> {
    let resp: PricesResponse = OracleQueryMsg::GetPrices { keys }.query(querier, oracle)?;
    Ok(resp.prices)
}
