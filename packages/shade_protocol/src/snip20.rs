use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::asset::Contract;
use secret_toolkit::{snip20::TokenInfo,
                    utils::Query};
use cosmwasm_std::{StdResult, StdError, Querier};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Snip20Asset {
    pub contract: Contract,
    pub token_info: TokenInfo,
    pub token_config: Option<TokenConfig>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokenConfig {
    pub public_total_supply: bool,
    pub deposit_enabled: bool,
    pub redeem_enabled: bool,
    pub mint_enabled: bool,
    pub burn_enabled: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Snip20Query {
    TokenConfig {},
}

impl Query for Snip20Query {
    const BLOCK_SIZE: usize = 256;
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
pub struct TokenConfigResponse {
    pub token_config: TokenConfig,
}

pub fn token_config_query<Q: Querier>(
    querier: &Q,
    contract: Contract,
) -> StdResult<TokenConfig> {
    let answer: TokenConfigResponse = Snip20Query::TokenConfig{}.query(querier,
                                                               contract.code_hash,
                                                               contract.address)?;
    Ok(answer.token_config)
}
