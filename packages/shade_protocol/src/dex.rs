use crate::{
    utils::asset::Contract,
    snip20::Snip20Asset,
    mint,
    secretswap,
    sienna,
    band,
    //shadeswap,
};
use cosmwasm_std::{HumanAddr, Uint128, StdResult, StdError, Extern, Querier, Api, Storage};
use schemars::JsonSchema;
use secret_toolkit::utils::Query;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Dex {
    SecretSwap,
    SiennaSwap,
    //ShadeSwap,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TradingPair{
    pub dex: Dex,
    pub contract: Contract,
    pub asset: Snip20Asset,
}

pub fn aggregate_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pairs: Vec<TradingPair>,
    sscrt: Contract,
    band: Contract,
) -> StdResult<Uint128> {

    // indices will align with <pairs>
    let mut prices = vec![];
    let mut weights = vec![];
    let weight_sum = Uint128(0);

    for pair in pairs {
        match &pair.dex {
            SecretSwap => {
                prices.push(secretswap::price(&deps, pair.clone(), sscrt.clone(), band.clone())?);
                weights.push(secretswap::pool_size(&deps, pair)?);
            },
            SiennaSwap => {
                prices.push(sienna::price(&deps, pair.clone(), sscrt.clone(), band.clone())?);
                weights.push(sienna::pool_size(&deps, pair)?);
            },
            /*
            ShadeSwap => {
                return Err(StdErr::generic_err("ShadeSwap Unavailable"));
            },
            */
        }
    }

    let weight_sum = Uint128(weights.iter().map(|i| i.u128()).sum());
    let mut weighted_sum = Uint128(0);

    for (p, w) in prices.iter().zip(weights.iter()) {
        weighted_sum = weighted_sum + (p.multiply_ratio(*w, weight_sum));
    }

    Ok(weighted_sum)
}

pub fn best_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pairs: Vec<TradingPair>,
    sscrt: Contract,
    band: Contract,
) -> StdResult<(Uint128, TradingPair)> {

    // indices will align with <pairs>
    let mut results = vec![];

    for pair in &pairs {
        match pair.clone().dex {
            SecretSwap => {
                results.push(secretswap::price(&deps, pair.clone(), sscrt.clone(), band.clone())?);
            },
            SiennaSwap => {
                results.push(sienna::price(&deps, pair.clone(), sscrt.clone(), band.clone())?);
            },
            /*
            ShadeSwap => {
                return Err(StdErr::generic_err("ShadeSwap Unavailable"));
            },
            */
        }
    }
    let max_amount = results.iter().max().unwrap();
    let index = results.iter().position(|e| e == max_amount).unwrap();
    let scrt_result = band::reference_data(deps, "SCRT".to_string(), "USD".to_string(), band)?;

    Ok((mint::translate_price(scrt_result.rate, *max_amount), pairs[index].clone()))
}

/*
pub fn best_pair<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pairs: Vec<AMMPair>,
    input_asset: Contract,
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
                },
            },
        },
    }
    .query(
        &deps.querier,
        pair.contract.code_hash,
        pair.contract.address,
    )?;

    Ok(mint::normalize_price(
        response.return_amount,
        pair.asset.token_info.decimals,
    ))
}

pub fn price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pair: Pair,
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
                },
            },
        },
    }
    .query(
        &deps.querier,
        pair.contract.code_hash,
        pair.contract.address,
    )?;

    Ok(mint::normalize_price(
        response.return_amount,
        pair.asset.token_info.decimals,
    ))
}
*/
