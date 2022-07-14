use shade_protocol::c_std::{
    to_binary,
    Api,
    Binary,
    Env,
    DepsMut,
    Response,
    Querier,
    StdError,
    StdResult,
    Storage,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shade_protocol::contract_interfaces::oracles::band::{InstantiateMsg, ReferenceData};
use shade_protocol::c_std::Uint128;

use shade_protocol::storage::{bucket, bucket_read, Bucket, ReadonlyBucket};

pub static PRICE: &[u8] = b"prices";

pub fn price_r<S: Storage>(storage: &S) -> ReadonlyBucket<S, Uint128> {
    bucket_read(PRICE, storage)
}

pub fn price_w<S: Storage>(storage: &mut S) -> Bucket<S, Uint128> {
    bucket(PRICE, storage)
}

pub fn init<S: Storage, A: Api, Q: Querier>(
    _deps: DepsMut,
    _env: Env,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::default())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    MockPrice { symbol: String, price: Uint128 },
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    _env: Env,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    return match msg {
        ExecuteMsg::MockPrice { symbol, price } => {
            price_w(&mut deps.storage).save(symbol.as_bytes(), &price)?;
            Ok(Response::default())
        }
    };
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetReferenceData {
        base_symbol: String,
        quote_symbol: String,
    },
    GetReferenceDataBulk {
        base_symbols: Vec<String>,
        quote_symbols: Vec<String>,
    },
}
pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetReferenceData {
            base_symbol,
            quote_symbol: _,
        } => {
            if let Some(price) = price_r(&deps.storage).may_load(base_symbol.as_bytes())? {
                return to_binary(&ReferenceData {
                    rate: price,
                    last_updated_base: 0,
                    last_updated_quote: 0,
                });
            }
            Err(StdError::generic_err("Missing Price Feed"))
        }
        QueryMsg::GetReferenceDataBulk {
            base_symbols,
            quote_symbols: _,
        } => {
            let mut results = Vec::new();

            for sym in base_symbols {
                if let Some(price) = price_r(&deps.storage).may_load(sym.as_bytes())? {
                    results.push(ReferenceData {
                        rate: price,
                        last_updated_base: 0,
                        last_updated_quote: 0,
                    });
                } else {
                    return Err(StdError::GenericErr {
                        msg: "Missing Price Feed".to_string(),
                        backtrace: None,
                    });
                }
            }
            to_binary(&results)
        }
    }
}
