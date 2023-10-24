use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_protocol::utils::liquidity_book::tokens::TokenType;

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
