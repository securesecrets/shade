#![allow(clippy::field_reassign_with_default)] // This is triggered in `#[derive(JsonSchema)]`

use shade_protocol::schemars::JsonSchema;
use shade_protocol::serde::{Deserialize, Serialize};

use shade_protocol::c_std::Uint128;
use shade_protocol::c_std::{to_binary, Binary, CosmosMsg, Addr, StdResult, WasmMsg};

use crate::{contract::RESPONSE_BLOCK_SIZE, msg::space_pad};

/// Snip20ReceiveMsg should be de/serialized under `Receive()` variant in a HandleMsg
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Snip20ReceiveMsg {
    pub sender: Addr,
    pub from: Addr,
    pub amount: Uint128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    pub msg: Option<Binary>,
}

impl Snip20ReceiveMsg {
    pub fn new(
        sender: Addr,
        from: Addr,
        amount: Uint128,
        memo: Option<String>,
        msg: Option<Binary>,
    ) -> Self {
        Self {
            sender,
            from,
            amount,
            memo,
            msg,
        }
    }

    /// serializes the message, and pads it to 256 bytes
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = ReceiverHandleMsg::Receive(self);
        let mut data = to_binary(&msg)?;
        space_pad(RESPONSE_BLOCK_SIZE, &mut data.0);
        Ok(data)
    }

    /// creates a cosmos_msg sending this struct to the named contract
    pub fn into_cosmos_msg(
        self,
        callback_code_hash: String,
        contract_addr: Addr,
    ) -> StdResult<CosmosMsg> {
        let msg = self.into_binary()?;
        let execute = WasmMsg::Execute {
            msg,
            callback_code_hash,
            contract_addr,
            send: vec![],
        };
        Ok(execute.into())
    }
}

// This is just a helper to properly serialize the above message
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "snake_case")]
enum ReceiverHandleMsg {
    Receive(Snip20ReceiveMsg),
}
