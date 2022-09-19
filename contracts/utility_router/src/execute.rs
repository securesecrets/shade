use shade_protocol::{
    c_std::{DepsMut, Addr, Binary, Response, StdResult, to_binary, QueryRequest, WasmQuery},
    contract_interfaces::{utility_router::{*, error::no_verfication_query_given}},
    Contract, admin::{helpers::{validate_admin, AdminPermissions}}
};
use shade_protocol::utils::generic_response::ResponseStatus::Success;

use crate::state::{STATUS, CONTRACTS, ADDRESSES};

pub fn toggle_status(deps: DepsMut, status: RouterStatus) -> StdResult<Response> {
    STATUS.update(deps.storage, |_| -> StdResult<_> { Ok(status) })?;
    
    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetStatus { status: Success })?))
}

pub fn set_contract(deps: DepsMut, utility_contract_name: String, contract: Contract, user: Addr, query: Option<Binary>) -> StdResult<Response> {
    if utility_contract_name == UtilityContracts::AdminAuth.into_string() {
        validate_admin(&deps.querier, AdminPermissions::UtilityRouterAdmin, user, &contract)?;
        CONTRACTS.save(deps.storage, UtilityContracts::AdminAuth.into_string(), &contract)?;
        return Ok(Response::new().set_data(to_binary(&HandleAnswer::SetContract { status: Success })?))
    }
    match CONTRACTS.may_load(deps.storage, utility_contract_name.clone())? {
        Some(_) => {
            if let Some(query) = query {
                verify_with_query(&deps, contract.clone(), query)?;
                CONTRACTS.save(deps.storage, utility_contract_name, &contract)?;
                Ok(Response::new().set_data(to_binary(&HandleAnswer::SetContract { status: Success })?))
            } else {
                Err(no_verfication_query_given())
            }

        },
        None => {
            CONTRACTS.save(deps.storage, utility_contract_name, &contract)?;
            Ok(Response::new().set_data(to_binary(&HandleAnswer::SetContract { status: Success })?))
        }
    }
}

pub fn set_address(deps: DepsMut, address_name: String, address: String) -> StdResult<Response> {
    ADDRESSES.save(deps.storage, address_name, &address)?;
    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetAddress { status: Success })?))
}

fn verify_with_query(deps: &DepsMut, contract: Contract, query: Binary) -> StdResult<Response> {
    deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart { contract_addr: contract.address.to_string(), code_hash: contract.code_hash, msg: query }))
}