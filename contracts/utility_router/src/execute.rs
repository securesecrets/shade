use shade_protocol::{
    admin::helpers::{validate_admin, AdminPermissions},
    c_std::{to_binary, Addr, DepsMut, Response, StdResult},
    contract_interfaces::utility_router::*,
    utils::generic_response::ResponseStatus::Success,
    Contract,
};

use crate::state::{ADDRESSES, CONTRACTS, STATUS};

pub fn toggle_status(deps: DepsMut, status: RouterStatus) -> StdResult<Response> {
    STATUS.update(deps.storage, |_| -> StdResult<_> { Ok(status) })?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetStatus { status: Success })?))
}

pub fn set_contract(
    deps: DepsMut,
    utility_contract_name: String,
    contract: Contract,
    user: Addr,
) -> StdResult<Response> {
    if utility_contract_name == UtilityContracts::AdminAuth.into_string() {
        validate_admin(
            &deps.querier,
            AdminPermissions::UtilityRouterAdmin,
            user,
            &contract,
        )?;
    }
    CONTRACTS.save(deps.storage, utility_contract_name, &contract)?;
    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetContract { status: Success })?))
}

pub fn set_address(deps: DepsMut, address_name: String, address: String) -> StdResult<Response> {
    ADDRESSES.save(deps.storage, address_name, &address)?;
    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetAddress { status: Success })?))
}
