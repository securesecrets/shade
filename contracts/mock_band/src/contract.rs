use cosmwasm_std::{
    to_binary, Api, Binary, 
    Env, Extern, HandleResponse, InitResponse, 
    Querier, StdResult, StdError, Storage, Uint128,
};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use shade_protocol::band::ReferenceData;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg { }

pub fn init<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse> {
    Ok(InitResponse::default())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg { }

pub fn handle<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _msg: HandleMsg,
) -> StdResult<HandleResponse> { 

    Err(StdError::GenericErr { msg: "Not Implemented".to_string(), backtrace: None})
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetReferenceData {
        base_symbol: String,
        quote_symbol: String,
    },
    GetReferenceDataBulk{
        base_symbols: Vec<String>,
        quote_symbols: Vec<String>,
    },
}
pub fn query<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetReferenceData { base_symbol: _, quote_symbol: _ } => 
            to_binary(&ReferenceData {
              rate: Uint128(1_000_000_000_000_000_000),
              last_updated_base: 1628544285u64,
              last_updated_quote: 3377610u64
            }),
        QueryMsg::GetReferenceDataBulk {
            base_symbols,
            quote_symbols: _
        } => {
            let mut results = Vec::new();
            let data = ReferenceData {
                  rate: Uint128(1_000_000_000_000_000_000),
                  last_updated_base: 1628544285u64,
                  last_updated_quote: 3377610u64
            };

            for _ in base_symbols {
                results.push(data.clone());
            }
            to_binary(&results)
        },
    }
}
