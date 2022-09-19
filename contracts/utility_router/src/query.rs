use shade_protocol::{
    c_std::{Deps, Binary, StdResult, QueryRequest, WasmQuery},
    contract_interfaces::{utility_router::{*, error::*}},
    utils::{generic_response::ResponseStatus::Success}
};

use crate::state::{STATUS, CONTRACTS, ADDRESSES};

pub fn get_status(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Status { contract_status: STATUS.load(deps.storage)?})
}

pub fn forward_query(deps: Deps, utility_name: String, query: Binary) -> StdResult<QueryAnswer> {
    match CONTRACTS.may_load(deps.storage, utility_name.clone())? {
        Some(contract) => {
            // let query_result = deps.querier.query_wasm_smart(contract.code_hash, contract.address, &query)?;
            // query_result
            match deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart { contract_addr: contract.address.to_string(), code_hash: contract.code_hash, msg: query })) {
                Ok(result) => Ok(QueryAnswer::ForwardQuery { status: Success, result }),
                Err(e) => Err(e),
            }
        },
        None => Err(no_contract_found(utility_name)),
    }
}

pub fn get_contract(deps: Deps, utility_name: String) -> StdResult<QueryAnswer> {
    match CONTRACTS.may_load(deps.storage, utility_name.clone())? {
        Some(contract) => Ok(QueryAnswer::GetContract {status: Success, contract}),
        None => Err(no_contract_found(utility_name)),
    }
}

pub fn get_address(deps: Deps, address_name: String) -> StdResult<QueryAnswer> {
    match ADDRESSES.may_load(deps.storage, address_name.clone())? {
        Some(addr) => Ok(QueryAnswer::GetAddress {status: Success, address: addr}),
        None => Err(no_address_found(address_name))
    }
}