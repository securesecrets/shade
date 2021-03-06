use crate::{
    contract_interfaces::{dex::dex, oracles::band},
    utils::{
        asset::Contract,
        price::{normalize_price, translate_price},
    },
};
use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage};
use cosmwasm_math_compat::Uint128;

use schemars::JsonSchema;
use secret_toolkit::{utils::Query, serialization::Base64};
use serde::{Deserialize, Serialize};

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
#[serde(rename_all = "snake_case")]
pub struct Pair {
    pub token_0: TokenType,
    pub token_1: TokenType,
}

/*
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AssetInfo {
    pub token: Token,
}
*/

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokenTypeAmount {
    pub amount: Uint128,
    pub token: TokenType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Swap {
    pub send: SwapOffer,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SwapOffer {
    pub recipient: HumanAddr,
    pub amount: Uint128,
    pub msg: Base64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CallbackMsg {
    pub swap: CallbackSwap,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CallbackSwap {
    pub expected_return: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SwapSimulation {
    pub offer: TokenTypeAmount,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PairQuery {
    /*
    Pool {},
    */
    PairInfo,
    SwapSimulation { offer: TokenTypeAmount },
}

impl Query for PairQuery {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SimulationResponse {
    pub return_amount: Uint128,
    pub spread_amount: Uint128,
    pub commission_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PairInfo {
    pub liquidity_token: Contract,
    pub factory: Contract,
    pub pair: Pair,
    pub amount_0: Uint128,
    pub amount_1: Uint128,
    pub total_liquidity: Uint128,
    pub contract_version: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PairInfoResponse {
    pub pair_info: PairInfo,
}

pub fn is_pair<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    pair: Contract,
) -> StdResult<bool> {
    Ok(
        match (PairQuery::PairInfo).query::<Q, PairInfoResponse>(
            &deps.querier,
            pair.code_hash,
            pair.address.clone(),
        ) {
            Ok(_) => true,
            Err(_) => false,
        },
    )
}

pub fn price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pair: dex::TradingPair,
    sscrt: Contract,
    band: Contract,
) -> StdResult<Uint128> {
    // TODO: This should be passed in to avoid multipl BAND SCRT queries in one query
    let scrt_result = band::reference_data(deps, "SCRT".to_string(), "USD".to_string(), band)?;

    // SCRT-USD / SCRT-symbol
    Ok(translate_price(
        scrt_result.rate,
        normalize_price(
            amount_per_scrt(deps, pair.clone(), sscrt)?,
            pair.asset.token_info.decimals,
        ),
    ))
}

pub fn amount_per_scrt<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pair: dex::TradingPair,
    sscrt: Contract,
) -> StdResult<Uint128> {
    let response: SimulationResponse = PairQuery::SwapSimulation {
        offer: TokenTypeAmount {
            amount: Uint128::new(1_000_000), // 1 sSCRT (6 decimals)
            token: TokenType::CustomToken {
                contract_addr: sscrt.address,
                token_code_hash: sscrt.code_hash,
            },
        },
    }
    .query(
        &deps.querier,
        pair.contract.code_hash,
        pair.contract.address,
    )?;

    Ok(response.return_amount)
}

pub fn pool_cp<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pair: dex::TradingPair,
) -> StdResult<Uint128> {
    let pair_info: PairInfoResponse = PairQuery::PairInfo.query(
        &deps.querier,
        pair.contract.code_hash,
        pair.contract.address,
    )?;

    // Constant Product
    Ok(Uint128::new(
        pair_info.pair_info.amount_0.u128() * pair_info.pair_info.amount_1.u128(),
    ))
}
