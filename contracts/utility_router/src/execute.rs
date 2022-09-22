use shade_protocol::{
    admin::helpers::{validate_admin, AdminPermissions},
    c_std::{to_binary, Addr, DepsMut, MessageInfo, Response, StdResult},
    contract_interfaces::utility_router::*,
    utils::generic_response::ResponseStatus::Success,
    Contract,
};

use crate::state::{ADDRESSES, CONTRACTS, STATUS};

pub fn set_status(deps: DepsMut, status: RouterStatus) -> StdResult<Response> {
    STATUS.save(deps.storage, &status)?;
    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetStatus { status: Success })?))
}

pub fn set_contract(
    deps: DepsMut,
    info: MessageInfo,
    key: String,
    contract: Contract,
) -> StdResult<Response> {
    CONTRACTS.save(deps.storage, key, &contract)?;
    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetContract { status: Success })?))
}

pub fn set_address(deps: DepsMut, key: String, address: Addr) -> StdResult<Response> {
    ADDRESSES.save(deps.storage, key, &address)?;
    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetAddress { status: Success })?))
}
