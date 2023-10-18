use cosmwasm_std::StdResult;
use cosmwasm_std::{MessageInfo, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_protocol::utils::liquidity_book::tokens::TokenType;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenAmount {
    pub token: TokenType,
    pub amount: Uint128,
}

impl TokenAmount {
    pub fn assert_sent_native_token_balance(&self, info: &MessageInfo) -> StdResult<()> {
        self.token
            .assert_sent_native_token_balance(info, self.amount)
    }
}
