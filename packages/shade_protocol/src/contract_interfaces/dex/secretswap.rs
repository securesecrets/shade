use crate::{
    contract_interfaces::{dex::dex, mint::mint, oracles::band},
    utils::{
        asset::Contract,
        price::{normalize_price, translate_price},
    },
};
use crate::c_std::{Addr, StdError, StdResult, Deps, DepsMut, Uint128};

use crate::utils::Query;
use crate::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Token {
    pub contract_addr: Addr,
    pub token_code_hash: String,
    pub viewing_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AssetInfo {
    pub token: Token,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Asset {
    pub amount: Uint128,
    pub info: AssetInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Simulation {
    pub offer_asset: Asset,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PairQuery {
    Pair {},
    Pool {},
    Simulation { offer_asset: Asset },
    //ReverseSimulation {},
}

impl Query for PairQuery {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct SimulationResponse {
    pub return_amount: Uint128,
    pub spread_amount: Uint128,
    pub commission_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct PairResponse {
    pub asset_infos: Vec<AssetInfo>,
    pub contract_addr: Addr,
    pub liquidity_token: Addr,
    pub token_code_hash: String,
    pub asset0_volume: Uint128,
    pub asset1_volume: Uint128,
    pub factory: Contract,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct PoolResponse {
    pub assets: Vec<Asset>,
    pub total_share: Uint128,
}

pub fn is_pair(
    deps: DepsMut,
    pair: Contract,
) -> StdResult<bool> {
    Ok(
        match (PairQuery::Pair {}).query::<PairResponse>(
            &deps.querier,
            &pair
        ) {
            Ok(_) => true,
            Err(_) => false,
        },
    )
}

pub fn price(
    deps: &Deps,
    pair: dex::TradingPair,
    sscrt: Contract,
    band: Contract,
) -> StdResult<Uint128> {
    let scrt_result = band::reference_data(deps, "SCRT".to_string(), "USD".to_string(), band)?;

    // SCRT-USD / SCRT-symbol
    Ok(translate_price(
        scrt_result.rate,
        normalize_price(
            amount_per_scrt(&deps, pair.clone(), sscrt)?,
            pair.asset.token_info.decimals,
        ),
    ))
}

pub fn amount_per_scrt(
    deps: &Deps,
    pair: dex::TradingPair,
    sscrt: Contract,
) -> StdResult<Uint128> {
    let response: SimulationResponse = PairQuery::Simulation {
        offer_asset: Asset {
            amount: Uint128::new(1_000_000), // 1 sSCRT (6 decimals)
            info: AssetInfo {
                token: Token {
                    contract_addr: sscrt.address,
                    token_code_hash: sscrt.code_hash,
                    viewing_key: "SecretSwap".to_string(),
                },
            },
        },
    }
    .query(
        &deps.querier,
        &pair.contract
    )?;

    Ok(response.return_amount)
}

pub fn pool_cp(
    deps: &Deps,
    pair: dex::TradingPair,
) -> StdResult<Uint128> {
    let pool: PoolResponse = PairQuery::Pool {}.query(
        &deps.querier,
        &pair.contract
    )?;

    // Constant Product
    Ok(Uint128::new(
        pool.assets[0].amount.u128() * pool.assets[1].amount.u128(),
    ))
}
