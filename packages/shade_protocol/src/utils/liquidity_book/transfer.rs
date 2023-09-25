//! ### Liquidity Book msgs Helper Library
//! Author: Haseeb
//!
//!
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_binary, Binary, Coin, CosmosMsg, StdResult, Uint128, WasmMsg};

const BLOCK_SIZE: usize = 256;
/// SNIP20 token handle messages
#[cw_serde]
pub enum HandleMsg {
    // Basic SNIP20 functions
    Transfer {
        recipient: String,
        amount: Uint128,
        memo: Option<String>,
        padding: Option<String>,
    },
    TransferFrom {
        owner: String,
        recipient: String,
        amount: Uint128,
        memo: Option<String>,
        padding: Option<String>,
    },
    Send {
        recipient: String,
        recipient_code_hash: Option<String>,
        amount: Uint128,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>,
    },
    RegisterReceive {
        code_hash: String,
        padding: Option<String>,
    },
    CreateViewingKey {
        entropy: String,
        padding: Option<String>,
    },
    SetViewingKey {
        key: String,
        padding: Option<String>,
    },
}

impl HandleMsg {
    /// Returns a StdResult<CosmosMsg> used to execute a SNIP20 contract function
    ///
    /// # Arguments
    ///
    /// * `block_size` - pad the message to blocks of this size
    /// * `callback_code_hash` - String holding the code hash of the contract being called
    /// * `contract_addr` - address of the contract being called
    /// * `send_amount` - Optional Uint128 amount of native coin to send with the callback message
    ///                 NOTE: Only a Deposit message should have an amount sent with it
    pub fn to_cosmos_msg(
        &self,
        code_hash: String,
        contract_addr: String,
        send_amount: Option<Uint128>,
    ) -> StdResult<CosmosMsg> {
        let mut msg = to_binary(self)?;
        space_pad(&mut msg.0, BLOCK_SIZE);
        let mut funds = Vec::new();
        if let Some(amount) = send_amount {
            funds.push(Coin {
                amount,
                denom: String::from("uscrt"),
            });
        }
        let execute = WasmMsg::Execute {
            contract_addr,
            code_hash,
            msg,
            funds,
        };
        Ok(execute.into())
    }
}

pub fn space_pad(message: &mut Vec<u8>, block_size: usize) -> &mut Vec<u8> {
    let len = message.len();
    let surplus = len % block_size;
    if surplus == 0 {
        return message;
    }

    let missing = block_size - surplus;
    message.reserve(missing);
    message.extend(std::iter::repeat(b' ').take(missing));
    message
}

#[cw_serde]
pub enum QueryMsg {
    Balance { address: String, key: String },
}

#[cw_serde]
pub enum QueryAnswer {
    Balance { amount: Uint128 },
}
