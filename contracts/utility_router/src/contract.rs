use crate::{execute::*, query::*, state::*};
use shade_protocol::{
    admin::helpers::{validate_admin, AdminPermissions},
    c_std::{
        shd_entry_point,
        to_binary,
        Binary,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdResult,
    },
    contract_interfaces::utility_router::{
        error::{critical_admin_error, under_maintenance},
        *,
    },
    utils::{pad_handle_result, pad_query_result},
};
// Used to pad up responses for better privacy.
pub const RESPONSE_BLOCK_SIZE: usize = 256;

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    CONTRACTS.save(
        deps.storage,
        UtilityContract::AdminAuth.into_string(),
        &msg.admin_auth,
    )?;
    STATUS.save(deps.storage, &RouterStatus::Running)?;
    Ok(Response::new())
}

#[shd_entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    if let Some(admin_contract) =
        CONTRACTS.may_load(deps.storage, UtilityContract::AdminAuth.into_string())?
    {
        validate_admin(
            &deps.querier,
            AdminPermissions::UtilityRouterAdmin,
            info.sender.clone(),
            &admin_contract,
        )?;
        pad_handle_result(
            match msg {
                ExecuteMsg::SetStatus { status, .. } => set_status(deps, status),
                ExecuteMsg::SetContract { key, contract, .. } => {
                    let contract = contract.into_valid(deps.api)?;
                    set_contract(deps, info, key, contract)
                }
                ExecuteMsg::SetAddress { key, address, .. } => {
                    let address = deps.api.addr_validate(&address)?;
                    set_address(deps, key, address)
                }
            },
            RESPONSE_BLOCK_SIZE,
        )
    } else {
        Err(critical_admin_error())
    }
}

#[shd_entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let status = STATUS.load(deps.storage)?;
    match status {
        // Continue
        RouterStatus::Running => {}
        // No information queries
        RouterStatus::UnderMaintenance => {
            if let QueryMsg::Status { .. } = msg {
            } else {
                return Err(under_maintenance());
            }
        }
    }

    pad_query_result(
        to_binary(&match msg {
            QueryMsg::Status {} => get_status(deps)?,
            QueryMsg::GetContract { key } => get_contract(deps, key)?,
            QueryMsg::GetAddress { key } => get_address(deps, key)?,
        }),
        RESPONSE_BLOCK_SIZE,
    )
}
