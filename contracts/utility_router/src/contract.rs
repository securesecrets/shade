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
        UtilityContracts::AdminAuth.into_string(),
        &msg.admin_auth,
    )?;
    ADDRESSES.save(
        deps.storage,
        UtilityAddresses::Multisig.into_string(),
        &msg.multisig_address,
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
    match CONTRACTS.may_load(deps.storage, UtilityContracts::AdminAuth.into_string())? {
        Some(admin_contract) => {
            validate_admin(
                &deps.querier,
                AdminPermissions::UtilityRouterAdmin,
                info.sender.clone(),
                &admin_contract,
            )?;
            pad_handle_result(
                match msg {
                    ExecuteMsg::ToggleStatus { status, .. } => toggle_status(deps, status),
                    ExecuteMsg::SetContract {
                        utility_contract_name,
                        contract,
                        ..
                    } => set_contract(deps, utility_contract_name, contract, info.sender),
                    ExecuteMsg::SetAddress {
                        address_name,
                        address,
                        ..
                    } => set_address(deps, address_name, address),
                },
                RESPONSE_BLOCK_SIZE,
            )
        }
        None => Err(critical_admin_error()),
    }
}

#[shd_entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let status = STATUS.load(deps.storage)?;
    match status {
        // Do nothing
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
            // QueryMsg::ForwardQuery { utility_name, query } => forward_query(deps, utility_name, query)?,
            QueryMsg::GetContract { utility_name } => get_contract(deps, utility_name)?,
            QueryMsg::GetAddress { address_name } => get_address(deps, address_name)?,
        }),
        RESPONSE_BLOCK_SIZE,
    )
}
