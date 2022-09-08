use shade_protocol::{
    c_std::{Deps, DepsMut, Addr, Binary, Env, MessageInfo, Response, StdResult, Storage, entry_point, to_binary, from_binary},
    contract_interfaces::{utility_router::*},
    utils::storage::plus::ItemStorage, Contract, admin::{helpers::{validate_admin, AdminPermissions}, errors::unauthorized_admin}, serde::Serialize,
};
use shade_protocol::utils::storage::plus::MapStorage;

use crate::state::{STATUS, ProtocolContract, CONTRACTS};

pub fn toggle_status(deps: DepsMut, status: RouterStatus) -> StdResult<Response> {
    STATUS.update(deps.storage, |_| -> StdResult<_> { Ok(status) })?;
    
    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetRunState { status: Success })?))
}

pub fn set_contract<T: Serialize>(deps: DepsMut, utility_contract_name: String, contract: Contract, user: Addr, query: Option<T>) -> StdResult<Response> {
    if utility_contract_name == UtilityContracts::AdminAuth.into_string() {
        verify_admin(deps, contract, user)?;
        CONTRACTS.save(deps.storage, UtilityContracts::AdminAuth.into_string(), &contract)?;
        Ok(Response::new().set_data(to_binary(&HandleAnswer::SetContract { status: Success })?))
    }
    match CONTRACTS.may_load(&deps.storage, utility_contract_name)? {
        Some(_) => {
            if let Some(query) = query {
                verify_with_query(deps, contract, query);
                CONTRACTS.save(deps.storage, utility_contract_name, &contract)?
            } else {
                Err(no_verfication_query_given())
            }

        },
        None => CONTRACTS.save(deps.storage, utility_contract_name, &contract)?,
    }
}

fn verify_admin(deps: DepsMut, contract: Contract, user: Addr) -> StdResult<()> {
    match validate_admin(&deps.querier, AdminPermissions::UtilityRouterAdmin, user, &admin_auth){
        Ok(_) => Ok(()),
        Err(_) => unauthorized_admin(admin, AdminPermissions::UtilityRouterAdmin),
    }
}

fn verify_with_query<T: Serialize>(deps: DepsMut, contract: Contract, query: T) -> StdResult<Response> {
    deps.querier.query_wasm_smart(contract.code_hash, contract.address, &query)?
}