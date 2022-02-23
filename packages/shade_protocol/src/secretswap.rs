use crate::{
    utils::asset::Contract,
    mint,
    dex,
    band,
};
use cosmwasm_std::{HumanAddr, Uint128, StdResult, StdError, Extern, Querier, Api, Storage};
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

    Ok(match (PairQuery::Pair {}).query::<Q, Result<PairResponse, StdError>>(
        &deps.querier,
        pair.code_hash,
        pair.address,
    ) {
        Ok(_) => true,
        Err(_) => false,
    })
}

pub fn price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    sswap_pair: dex::TradingPair,
    sscrt: Contract,
    band: Contract,
) -> StdResult<Uint128> {

    let scrt_result = band::reference_data(deps, "SCRT".to_string(), "USD".to_string(), band)?;

    //return Err(StdError::NotFound { kind: translate_price(scrt_result.rate, trade_price).to_string(), backtrace: None });

    // SCRT-USD / SCRT-symbol
    Ok(mint::translate_price(scrt_result.rate, simulate(deps, sswap_pair, sscrt)?))
}

pub fn simulate<S: Storage, A: Api, Q: Querier>(
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

    Ok(mint::normalize_price(
        response.return_amount,
        pair.asset.token_info.decimals,
    ))
}

pub fn pool_size<S: Storage, A: Api, Q: Querier>(
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
