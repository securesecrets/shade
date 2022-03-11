use crate::{
    utils::asset::Contract,
    mint,
    dex,
    band,
};
use cosmwasm_std::{Uint128, HumanAddr, StdResult, StdError, Extern, Querier, Api, Storage};
use schemars::JsonSchema;
use secret_toolkit::utils::Query;
use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Token {
    pub contract_addr: HumanAddr,
    pub token_code_hash: String,
    pub viewing_key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AssetInfo {
    pub token: Token,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Asset {
    pub amount: Uint128,
    pub info: AssetInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Simulation {
    pub offer_asset: Asset,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SimulationResponse {
    pub return_amount: Uint128,
    pub spread_amount: Uint128,
    pub commission_amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PairResponse {
    pub asset_infos: Vec<AssetInfo>,
    pub contract_addr: HumanAddr,
    pub liquidity_token: HumanAddr,
    pub token_code_hash: String,
    pub asset0_volume: Uint128,
    pub asset1_volume: Uint128,
    pub factory: Contract,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PoolResponse {
    pub assets: Vec<Asset>,
    pub total_share: Uint128,
}

pub fn is_pair<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    pair: Contract,
) -> StdResult<bool> {

    Ok(match (PairQuery::Pair {}).query::<Q, PairResponse>(
        &deps.querier,
        pair.code_hash,
        pair.address.clone(),
    ) {
        Ok(_) => true,
        Err(_) => false,
    })
}

pub fn price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pair: dex::TradingPair,
    sscrt: Contract,
    band: Contract,
) -> StdResult<Uint128> {

    let scrt_result = band::reference_data(
        deps, 
        "SCRT".to_string(), 
        "USD".to_string(), 
        band
    )?;

    // SCRT-USD / SCRT-symbol
    Ok(mint::translate_price(scrt_result.rate, 
         mint::normalize_price(
             amount_per_scrt(deps, pair.clone(), sscrt)?, 
             pair.asset.token_info.decimals
         )
    ))
}

pub fn amount_per_scrt<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pair: dex::TradingPair,
    sscrt: Contract,
) -> StdResult<Uint128> {

    let response: SimulationResponse = PairQuery::Simulation {
        offer_asset: Asset {
            amount: Uint128(1_000_000), // 1 sSCRT (6 decimals)
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
        pair.contract.code_hash,
        pair.contract.address,
    )?;

    Ok(response.return_amount)
}

pub fn pool_cp<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pair: dex::TradingPair,
) -> StdResult<Uint128> {

    let pool: PoolResponse = PairQuery::Pool {}.query(
        &deps.querier,
        pair.contract.code_hash,
        pair.contract.address,
    )?;

    // Constant Product
    Ok(Uint128(pool.assets[0].amount.u128() * pool.assets[1].amount.u128()))
}
