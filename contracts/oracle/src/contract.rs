use cosmwasm_std::{debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdResult, StdError, Storage, Uint128, WasmQuery};
use crate::state::{config, config_read};//, price_read};
use crate::struct_types::{ReferenceData};
use shade_protocol::{
    oracle::{InitMsg, HandleMsg, QueryMsg, QueryAnswer, OracleConfig, PriceResponse},
    asset::{Contract},
    msg_traits::{Init, Query},
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = OracleConfig {
        owner: match msg.admin {
            None => { env.message.sender.clone() }
            Some(admin) => { admin }
        },
        band: msg.band,
    };

    config(&mut deps.storage).save(&state)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetScrtPrice {} => to_binary(&query_scrt_price(deps)?),
        QueryMsg::GetPrice { symbol } => Err(StdError::generic_err(symbol)), //to_binary(&query_price(deps, symbol)?),
        QueryMsg::GetConfig {} => to_binary(&query_config(deps)?),
        QueryMsg::GetReferenceData { base_symbol, quote_symbol } => 
            to_binary(&query_reference_data(deps, base_symbol, quote_symbol)?),
    }
}

fn query_scrt_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
)  -> StdResult<ReferenceData> {
    Ok(query_reference_data(&deps, "SCRT".to_string(), "USD".to_string())?)
}

/*
fn query_price<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    symbol: String,
)  -> StdResult<PriceResponse> {
    match price_read(&deps.storage).get(&symbol.as_bytes()) {
        Some(data) => {
            Ok(PriceResponse { price: bincode::deserialize(&data).unwrap() })
        },
        _ => Err(StdError::generic_err(format!(
            "PRICE_NOT_AVAILABLE_FOR_KEY:{}",
            symbol
        ))),
    }
}
*/

fn query_config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config { config: config_read(&deps.storage).load()? })
}

// cross-contract query
fn query_reference_data<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    base_symbol: String,
    quote_symbol: String,
) -> StdResult<ReferenceData> {

    let config_read = config_read(&deps.storage).load()?;
    let reference_data: ReferenceData = QueryMsg::GetReferenceData {
        base_symbol,
        quote_symbol
    }.query(
        &deps.querier,
        //block_size
        1,
        config_read.band.code_hash,
        config_read.band.address)?;
    debug_print!("SCRT/USD {}", reference_data.rate);
    Ok(reference_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockStorage, MockApi, MockQuerier};
    use cosmwasm_std::{coins, from_binary};

    fn create_contract(address: &str, code_hash: &str) -> Contract {
        let env = mock_env(address.to_string(), &[]);
        return Contract{
            address: env.message.sender,
            code_hash: code_hash.to_string()
        }
    }

    fn dummy_init(admin: &str) -> Extern<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies(20, &[]);
        let msg = InitMsg {
            admin: None,
            band: create_contract("", ""),
        };
        let env = mock_env(admin.to_string(), &coins(1000, "earth"));
        let _res = init(&mut deps, env, msg).unwrap();

        return deps
    }

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);

        let msg = InitMsg {
            admin: None,
            band: create_contract("", ""),
        };
        let env = mock_env("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());
    }

    /*
    #[test]
    fn query_price() {
        let deps = dummy_init("admin");

        // Query the price
        let res = query(&deps, QueryMsg::GetScrtPrice {}).unwrap();
        let value: PriceResponse = from_binary(&res).unwrap();
        let expected_price = Uint128(10u64.pow(18) as u128);
        assert_eq!(expected_price, value.price);
    }
    */
}
