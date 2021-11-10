use cosmwasm_std::{
    Api,
    Extern,
    Querier, StdResult, Storage,
    Uint128,
};
use secret_toolkit::utils::Query;
use shade_protocol::{
    oracle::{
        QueryAnswer, SswapPair
    },
    band::{ 
        BandQuery, ReferenceData,
    },
    secretswap::{
        PairQuery,
        SimulationResponse,
        Asset,
        AssetInfo,
        Token,
    },

};
use crate::state::{
    config_r,
    hard_coded_r,
    sswap_pairs_r,
};
use std::convert::TryFrom;

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config { config: config_r(&deps.storage).load()? })
}

pub fn get_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    symbol: String,
) -> StdResult<ReferenceData> {

    if symbol == "SSCRT" {
        return reference_data(deps, "SCRT".to_string(), "USD".to_string());
    }

    if let Some(reference_data) = hard_coded_r(&deps.storage).may_load(&symbol.as_bytes())? {
        return Ok(reference_data);
    }

    // symbol registered sswap pair
    if let Some(sswap_pair) = sswap_pairs_r(&deps.storage).may_load(symbol.as_bytes())? {

        let trade_price = sswap_simulate(&deps, sswap_pair)?;

        let scrt_result = reference_data(deps, "SCRT".to_string(), "USD".to_string())?;

        //return Err(StdError::NotFound { kind: translate_price(scrt_result.rate, trade_price).to_string(), backtrace: None });

        return Ok(ReferenceData {
            // SCRT-USD / SCRT-symbol
            rate: translate_price(scrt_result.rate, trade_price),
            last_updated_base: 0,
            last_updated_quote: 0
        });
    }

    // symbol/USD price from BAND
    reference_data(deps, symbol, "USD".to_string())
}

/* Translate price from symbol/sSCRT -> symbol/USD
 *
 * scrt_price: SCRT/USD price from BAND
 * trade_price: SCRT/token trade amount from 1 sSCRT (normalized to price * 10^18)
 * return: token/USD price
 */
pub fn translate_price(
    scrt_price: Uint128, 
    trade_price: Uint128
) -> Uint128 {

    scrt_price.multiply_ratio(10u128.pow(18), trade_price)
}

/* Normalize the price from snip20 amount with decimals to BAND rate
 * amount: unsigned quantity received in trade for 1sSCRT
 * decimals: number of decimals for received snip20
 */
pub fn normalize_price(
    amount: Uint128, 
    decimals: u8
) -> Uint128 {

    (amount.u128() * 10u128.pow(18u32 - u32::try_from(decimals).unwrap())).into()
}

// Secret Swap interactions

pub fn sswap_simulate<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    sswap_pair: SswapPair,
) -> StdResult<Uint128> {

    let config = config_r(&deps.storage).load()?;

    let response: SimulationResponse = PairQuery::Simulation {
        offer_asset: Asset {
            amount: Uint128(1_000_000), // 1 sSCRT (6 decimals)
            info: AssetInfo {
                token: Token {
                    contract_addr: config.sscrt.address,
                    token_code_hash: config.sscrt.code_hash,
                    viewing_key: "SecretSwap".to_string(),
                }
            }
        }
    }.query(
        &deps.querier,
        sswap_pair.pair.code_hash,
        sswap_pair.pair.address,
    )?;

    return Ok(normalize_price(response.return_amount, sswap_pair.asset.token_info.decimals));
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
