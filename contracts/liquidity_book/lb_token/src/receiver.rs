#![allow(clippy::field_reassign_with_default)] // This is triggered in `#[derive(JsonSchema)]`

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{to_binary, Addr, Binary, CosmosMsg, StdResult, Uint256, WasmMsg};

use crate::state::RESPONSE_BLOCK_SIZE;
use shade_protocol::liquidity_book::lb_token::space_pad;

/// Snip1155ReceiveMsg should be de/serialized under `Snip1155Receive()` variant in a HandleMsg
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Snip1155ReceiveMsg {
    /// the address that sent the `Send` or `BatchSend` message
    pub sender: Addr,
    /// unique token_id `String`
    pub token_id: String,
    /// the previous owner of the tokens being transferred
    pub from: Addr,
    /// amount of tokens being transferred
    pub amount: Uint256,
    /// optional memo
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    /// optional message
    pub msg: Option<Binary>,
}

impl Snip1155ReceiveMsg {
    pub fn new(
        sender: Addr,
        token_id: String,
        from: Addr,
        amount: Uint256,
        memo: Option<String>,
        msg: Option<Binary>,
    ) -> Self {
        Self {
            sender,
            token_id,
            from,
            amount,
            memo,
            msg,
        }
    }

    /// serializes the message, and pads it to 256 bytes
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = ReceiverHandleMsg::Snip1155Receive(self);
        let mut data = to_binary(&msg)?;
        space_pad(RESPONSE_BLOCK_SIZE, &mut data.0);
        Ok(data)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg(self, code_hash: String, contract_addr: Addr) -> StdResult<CosmosMsg> {
        let msg = self.into_binary()?;
        let execute = WasmMsg::Execute {
            msg,
            code_hash,
            contract_addr: contract_addr.to_string(),
            funds: vec![],
        };
        Ok(execute.into())
    }
}

// This is just a helper to properly serialize the above message
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ReceiverHandleMsg {
    Snip1155Receive(Snip1155ReceiveMsg),
}
