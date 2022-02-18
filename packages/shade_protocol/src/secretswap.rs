use crate::utils::asset::Contract;
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
