use shade_protocol::{
    c_std::{Deps, StdResult},
    contract_interfaces::utility_router::{error::*, *},
    utils::generic_response::ResponseStatus::Success,
};

use crate::state::{ADDRESSES, CONTRACTS, STATUS};

pub fn get_status(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Status {
        contract_status: STATUS.load(deps.storage)?,
    })
}

/*
pub fn forward_query(deps: Deps, utility_name: String, query: Binary) -> StdResult<QueryAnswer> {
    match CONTRACTS.may_load(deps.storage, utility_name.clone())? {
        Some(contract) => {
            // let query_result = deps.querier.query_wasm_smart(contract.code_hash, contract.address, &query)?;
            // query_result

            let response = to_binary(&deps.querier.query::<>(
                &QueryRequest::Wasm(
                    WasmQuery::Smart {
                        contract_addr: contract.address.to_string(),
                        code_hash: contract.code_hash,
                        msg: query
                    }
                )
            ).unwrap());
            Ok(QueryAnswer::ForwardQuery { status: Success, result: to_binary("data").unwrap() })
            // match deps.querier.query::<Binary>(&QueryRequest::Wasm(WasmQuery::Smart { contract_addr: contract.address.to_string(), code_hash: contract.code_hash, msg: query })) {
            //     Ok(result) => {
            //         // Ok(QueryAnswer::ForwardQuery { status: Success, result })
            //         Err(no_verfication_query_given())
            //     },
            //     Err(e) => Err(e),
            // }
        },
        None => Err(no_contract_found(utility_name)),
    }
}
*/

pub fn get_contract(deps: Deps, key: String) -> StdResult<QueryAnswer> {
    if let Some(contract) = CONTRACTS.may_load(deps.storage, key.clone())? {
        Ok(QueryAnswer::GetContract {
            status: Success,
            contract,
        })
    } else {
        Err(no_contract_found(key))
    }
}

pub fn get_address(deps: Deps, key: String) -> StdResult<QueryAnswer> {
    if let Some(address) = ADDRESSES.may_load(deps.storage, key.clone())? {
        Ok(QueryAnswer::GetAddress {
            status: Success,
            address,
        })
    } else {
        Err(no_address_found(key))
    }
}
