use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use secret_toolkit::utils::space_pad;
use cosmwasm_std::{HumanAddr, CosmosMsg, WasmMsg, to_binary, StdResult};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
}

impl InitMsg {
    pub fn to_cosmos_msg(
        &self,
        mut block_size: usize,
        code_id: u64,
        callback_code_hash: String,
        label: String,
    ) -> StdResult<CosmosMsg> {
        // can not have block size of 0
        if block_size == 0 {
            block_size = 1;
        }
        let mut msg = to_binary(self)?;
        space_pad(&mut msg.0, block_size);
        let execute = WasmMsg::Instantiate {
            code_id,
            callback_code_hash,
            msg,
            send: vec![],
            label
        };
        Ok(execute.into())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetSilkPrice {} 
}


// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PriceResponse {
    pub price: i128,
}
