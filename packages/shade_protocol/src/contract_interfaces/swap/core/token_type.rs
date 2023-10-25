use crate::utils::liquidity_book::tokens::TokenType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StableTokenData {
    pub oracle_key: String,
    pub decimals: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StableTokenType {
    pub token: TokenType,
    pub stable_token_data: StableTokenData,
}
