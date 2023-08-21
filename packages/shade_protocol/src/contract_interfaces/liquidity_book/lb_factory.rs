use crate::utils::liquidity_book::{
    tokens::TokenType, transfer::space_pad, types::LBPairInformation,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{to_binary, Coin, CosmosMsg, StdResult, Uint128, WasmMsg};
#[cw_serde]
pub enum ExecuteMsg {
    #[serde(rename = "create_lb_pair")]
    CreateLBPair {
        token_x: TokenType,
        token_y: TokenType,
        // u24
        active_id: u32,
        bin_step: u16,
    },
}

impl ExecuteMsg {
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
        space_pad(&mut msg.0, 256);
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

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(LBPairInformationResponse)]
    #[serde(rename = "get_lb_pair_information")]
    GetLBPairInformation {
        token_x: TokenType,
        token_y: TokenType,
        bin_step: u16,
    },
}

#[cw_serde]
pub struct LBPairInformationResponse {
    pub lb_pair_information: LBPairInformation,
}
