use shade_protocol::{
    //admin::helpers::{validate_admin, AdminPermissions},
    c_std::{to_binary, Addr, DepsMut, MessageInfo, Response, StdError, StdResult},
    contract_interfaces::utility_router::*,
    utils::generic_response::ResponseStatus::Success,
    Contract,
};

use crate::storage::{ADDRESSES, CONTRACTS, KEYS, STATUS};

pub fn set_status(deps: DepsMut, status: RouterStatus) -> StdResult<Response> {
    STATUS.save(deps.storage, &status)?;
    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetStatus { status: Success })?))
}

pub fn set_contract(
    deps: DepsMut,
    _info: MessageInfo,
    key: String,
    contract: Contract,
) -> StdResult<Response> {
    let mut keys = KEYS.load(deps.storage)?;
    if !keys.contains(&key) {
        keys.push(key.clone());
        KEYS.save(deps.storage, &keys)?;
    }

    CONTRACTS.save(deps.storage, key, &contract)?;
    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetContract { status: Success })?))
}

pub fn set_address(deps: DepsMut, key: String, address: Addr) -> StdResult<Response> {
    let mut keys = KEYS.load(deps.storage)?;
    if !keys.contains(&key) {
        keys.push(key.clone());
        KEYS.save(deps.storage, &keys)?;
    }

    if let Some(_) = CONTRACTS.may_load(deps.storage, key.clone())? {
        return Err(StdError::generic_err("Key already saved as contract"));
    }

    ADDRESSES.save(deps.storage, key, &address)?;
    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetAddress { status: Success })?))
}
