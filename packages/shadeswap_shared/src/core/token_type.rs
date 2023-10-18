use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, ContractInfo, CosmosMsg, Deps, MessageInfo, StdError,
    StdResult, Uint128, Uint256, WasmMsg,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_protocol::Contract;
use shade_protocol::{
    snip20::{
        helpers::{balance_query, token_info},
        ExecuteMsg::Send,
    },
    utils::liquidity_book::tokens::TokenType,
};

use super::TokenAmount;

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
