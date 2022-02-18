use crate::utils::asset::Contract;
use cosmwasm_std::{HumanAddr, Uint128, StdResult, StdError, Extern, Querier, Api, Storage};
use schemars::JsonSchema;
use secret_toolkit::utils::Query;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CustomToken {
    pub contract_addr: HumanAddr,
    pub token_code_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    CustomToken {
        custom_token: CustomToken,
    },
    NativeToken {
        denom: String,
    },
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Pair {
    pub token_0: CustomToken,
    pub token_1: CustomToken,
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
pub struct SwapSimulation {
    pub offer: TokenType,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PairQuery {
    /*
    Pair {},
    Pool {},
    Simulation { offer_asset: Asset },
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
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PairInfoResponse {
    pub pair_info: PairInfo,
}

/*
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PoolResponse {
    pub assets: Vec<Asset>,
    pub total_share: Uint128,
}
*/

pub fn is_pair<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    pair: Contract,
) -> StdResult<bool> {
    
    Ok(match PairQuery::PairInfo.query::<Q, Result<PairInfoResponse, StdError>>(
        &deps.querier,
        pair.code_hash,
        pair.address,
    )? {
        Ok(_) => true,
        Err(_) => false,
    })
}
