use shade_protocol::{
    c_std::{Deps, DepsMut, Addr, Binary, Env, MessageInfo, Response, StdResult, Storage, entry_point, to_binary},
    contract_interfaces::{utility_router::*},
    utils::{storage::plus::{ItemStorage, MapStorage}, Query}, Contract, admin::{helpers::{validate_admin, AdminPermissions}, errors::unauthorized_admin}, query_auth::QueryPermit, serde::Serialize,
};

use crate::state::{ProtocolContract, STATUS, CONTRACTS};

pub fn get_status(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Status { contract_status: STATUS.load(deps.storage)})
}

pub fn forward_query(deps: Deps, utility_name: String, query: Binary) -> StdResult<QueryAnswer> {
    match CONTRACTS.may_load(deps.storage, utility_name)? {
        Some(contract) => {
            deps.querier.query_wasm_smart(contract.code_hash, contract.address, &query)?
        },
        None => Err(no_contract_found(utility_name)),
    }
}

pub fn get_contract(deps: Deps, utility_name: String) -> StdResult<QueryAnswer> {
    match CONTRACTS.may_load(deps.storage, utility_name)? {
        Some(contract) => Ok(QueryAnswer::GetContract {status: Success, contract}),
        None => Err(no_contract_found(utility_name)),
    }
}