use crate::{
    contract_interfaces::{
        dex::{secretswap, sienna},
        mint::mint,
        oracles::band,
        snip20::helpers::Snip20Asset,
    },
    utils::{
        asset::Contract,
        price::{normalize_price, translate_price},
    },
};
use cosmwasm_std::{self, Api, Extern, Querier, StdError, StdResult, Storage, HumanAddr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_math_compat::{Uint128, Uint512};
use std::convert::TryFrom;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    CustomToken {
        contract_addr: HumanAddr,
        token_code_hash: String,
    },
    NativeToken {
        denom: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Dex {
    SecretSwap,
    SiennaSwap,
    //ShadeSwap,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TradingPair {
    pub dex: Dex,
    pub contract: Contract,
    pub asset: Snip20Asset,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TradingPairNoAsset {
    pub dex: Dex,
    pub contract: Contract,
}

/* give_amount into give_pool
 * returns how much to be received from take_pool
 */

pub fn pool_take_amount(
    give_amount: Uint128,
    give_pool: Uint128,
    take_pool: Uint128,
) -> Uint128 {
    Uint128::new(
        take_pool.u128() - give_pool.u128() * take_pool.u128() / (give_pool + give_amount).u128(),
    )
}

pub fn aggregate_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pairs: Vec<TradingPair>,
    sscrt: Contract,
    band: Contract,
) -> StdResult<Uint128> {
    // indices will align with <pairs>
    let mut amounts_per_scrt = vec![];
    let mut pool_sizes: Vec<Uint512> = vec![];

    for pair in pairs.clone() {
        match &pair.dex {
            Dex::SecretSwap => {
                amounts_per_scrt.push(Uint512::from(
                    normalize_price(
                        secretswap::amount_per_scrt(&deps, pair.clone(), sscrt.clone())?,
                        pair.asset.token_info.decimals,
                    )
                    .u128(),
                ));
                pool_sizes.push(Uint512::from(secretswap::pool_cp(&deps, pair)?.u128()));
            }
            Dex::SiennaSwap => {
                amounts_per_scrt.push(Uint512::from(
                    normalize_price(
                        sienna::amount_per_scrt(&deps, pair.clone(), sscrt.clone())?,
                        pair.asset.token_info.decimals,
                    )
                    .u128(),
                ));
                pool_sizes.push(Uint512::from(sienna::pool_cp(&deps, pair)?.u128()));
            } /*
              ShadeSwap => {
                  prices.push(shadeswap::price(&deps, pair.clone(), sscrt.clone(), band.clone())?);
                  pool_sizes.push(shadeswap::pool_size(&deps, pair)?);
                  return Err(StdErr::generic_err("ShadeSwap Unavailable"));
              },
              */
        }
    }

    let mut combined_cp: Uint512 = pool_sizes.iter().sum();

    let weighted_sum: Uint512 = amounts_per_scrt
        .into_iter()
        .zip(pool_sizes.into_iter())
        .map(|(a, s)| a * s / combined_cp)
        .sum();

    // Translate price from SHD/SCRT -> SHD/USD
    // And normalize to <price> * 10^18
    let price = translate_price(
        band::reference_data(deps, "SCRT".to_string(), "USD".to_string(), band)?.rate,
        Uint128::new(Uint128::try_from(weighted_sum)?.u128()),
    );

    Ok(price)
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
                results.push(secretswap::price(
                    &deps,
                    pair.clone(),
                    sscrt.clone(),
                    band.clone(),
                )?);
            }
            Dex::SiennaSwap => {
                results.push(sienna::price(
                    &deps,
                    pair.clone(),
                    sscrt.clone(),
                    band.clone(),
                )?);
            } /*
              ShadeSwap => {
                  return Err(StdErr::generic_err("ShadeSwap Unavailable"));
              },
              */
        }
    }
    let max_amount = results.iter().max().unwrap();
    let index = results.iter().position(|e| e == max_amount).unwrap();
    let scrt_result = band::reference_data(deps, "SCRT".to_string(), "USD".to_string(), band)?;

    Ok((
        translate_price(scrt_result.rate, *max_amount),
        pairs[index].clone(),
    ))
}

pub fn price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pair: TradingPair,
    sscrt: Contract,
    band: Contract,
) -> StdResult<Uint128> {
    match pair.clone().dex {
        Dex::SecretSwap => Ok(secretswap::price(
            &deps,
            pair.clone(),
            sscrt.clone(),
            band.clone(),
        )?),
        Dex::SiennaSwap => Ok(sienna::price(
            &deps,
            pair.clone(),
            sscrt.clone(),
            band.clone(),
        )?),
        /*
        ShadeSwap => {
            return Err(StdErr::generic_err("ShadeSwap Unavailable"));
        },
        */
    }
}
