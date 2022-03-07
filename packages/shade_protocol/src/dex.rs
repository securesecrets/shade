use crate::{
    utils::asset::Contract,
    snip20::Snip20Asset,
    mint,
    secretswap,
    sienna,
    band,
    //shadeswap,
};
use cosmwasm_std::{Uint128, StdResult, Extern, Querier, Api, Storage, StdError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
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

/* give_amount into give_pool
 * returns how much to be received from take_pool
 */
pub fn pool_take_amount(
    give_amount: Uint128,
    give_pool: Uint128,
    take_pool: Uint128,
) -> Uint128 {
    Uint128(
        take_pool.u128() - (
            (give_pool.u128() * take_pool.u128())
            / (give_pool.u128() + give_amount.u128())
        )
    )
}

pub fn aggregate_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pairs: Vec<TradingPair>,
    sscrt: Contract,
    band: Contract,
) -> StdResult<Uint128> {

    // indices will align with <pairs>
    let mut prices = vec![];
    let mut pool_sizes = vec![];

    for pair in pairs {
        match &pair.dex {
            Dex::SecretSwap => {
                prices.push(secretswap::price(&deps, pair.clone(), sscrt.clone(), band.clone())?);
                pool_sizes.push(secretswap::pool_size(&deps, pair)?);
            },
            Dex::SiennaSwap => {
                prices.push(sienna::price(&deps, pair.clone(), sscrt.clone(), band.clone())?);
                pool_sizes.push(sienna::pool_size(&deps, pair)?);
            },
            /*
            ShadeSwap => {
                prices.push(shadeswap::price(&deps, pair.clone(), sscrt.clone(), band.clone())?);
                pool_sizes.push(shadeswap::pool_size(&deps, pair)?);
                return Err(StdErr::generic_err("ShadeSwap Unavailable"));
            },
            */
        }
    }

    /*
    return Err(StdError::generic_err(
        format!("pool_sizes {}", pool_sizes.len())
    ));
    */
    let combined_cp: u128 = pool_sizes.iter().map(|i| i.u128()).sum();
    let mut weighted_sum = 0u128;


    for (price, pool_size) in prices.iter().zip(pool_sizes.iter()) {
        //weighted_sum = weighted_sum + price.multiply_ratio(*pool_size, combined_cp).u128();
        weighted_sum += price.u128() * pool_size.u128();
    }

    Ok(Uint128(weighted_sum / combined_cp))
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
            Dex::SecretSwap => {
                results.push(secretswap::price(&deps, pair.clone(), sscrt.clone(), band.clone())?);
            },
            Dex::SiennaSwap => {
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

pub fn price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pair: TradingPair,
    sscrt: Contract,
    band: Contract,
) -> StdResult<Uint128> {

    match pair.clone().dex {
        Dex::SecretSwap => {
            Ok(secretswap::price(&deps, pair.clone(), sscrt.clone(), band.clone())?)
        },
        Dex::SiennaSwap => {
            Ok(sienna::price(&deps, pair.clone(), sscrt.clone(), band.clone())?)
        },
        /*
        ShadeSwap => {
            return Err(StdErr::generic_err("ShadeSwap Unavailable"));
        },
        */
    }
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
