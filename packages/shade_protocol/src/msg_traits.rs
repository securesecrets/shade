use serde::{Deserialize, Serialize};
use cosmwasm_std::{CosmosMsg, StdResult, to_binary, WasmMsg, HumanAddr, Uint128, Coin, Querier, QueryRequest, WasmQuery, StdError};
use secret_toolkit::utils::space_pad;
use serde::de::DeserializeOwned;

pub trait Init<'a>: Serialize + Deserialize<'a> + Clone + PartialEq  {
    fn to_cosmos_msg(
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

pub trait Handle<'a>: Serialize + Deserialize<'a> + Clone + PartialEq {
    fn to_cosmos_msg(
        &self,
        mut block_size: usize,
        callback_code_hash: String,
        contract_addr: HumanAddr,
        send_amount: Option<Uint128>,
    ) -> StdResult<CosmosMsg> {
        // can not have block size of 0
        if block_size == 0 {
            block_size = 1;
        }
        let mut msg = to_binary(self)?;
        space_pad(&mut msg.0, block_size);
        let mut send = Vec::new();
        if let Some(amount) = send_amount {
            send.push(Coin {
                amount,
                denom: String::from("uscrt"),
            });
        }
        let execute = WasmMsg::Execute {
            msg,
            contract_addr,
            callback_code_hash,
            send,
        };
        Ok(execute.into())
    }
}

pub trait Query: Serialize + Clone {
    fn query<Q: Querier, T: DeserializeOwned>(
        &self,
        querier: &Q,
        mut block_size: usize,
        callback_code_hash: String,
        contract_addr: HumanAddr,
    ) -> StdResult<T> {
        // can not have block size of 0
        if block_size == 0 {
            block_size = 1;
        }
        let mut msg = to_binary(self)?;
        space_pad(&mut msg.0, block_size);
        querier
            .query(&QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr,
                callback_code_hash,
                msg,
            }))
            .map_err(|_err| {
                StdError::generic_err(format!("Error performing query"))
            })
    }
}
