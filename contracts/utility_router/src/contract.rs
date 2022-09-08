use cosmwasm_std::{
    entry_point, to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Storage,
};
use shade_protocol::{
    c_std::{Deps, DepsMut, Addr, Binary, Env, MessageInfo, Response, StdResult, Storage, entry_point, to_binary},
    contract_interfaces::utility_router::*,
    utils::storage::plus::ItemStorage, admin::{helpers::{validate_admin, AdminPermissions}, errors::unauthorized_admin},
};
use crate::{execute::*, query::*, state::*};
use shade_protocol::utils::storage::plus::MapStorage;
// Used to pad up responses for better privacy.
pub const RESPONSE_BLOCK_SIZE: usize = 256;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    CONTRACTS.save(deps.storage, UtilityContracts::AdminAuth.into_string(), &msg.admin_auth)?;
    STATUS.save(deps.storage, &RouterStatus::Running)?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    is_admin(deps.as_ref(), info.sender, &env)?;
    pad_handle_result(
        match msg {
            ExecuteMsg::ToggleStatus { status, ..} => set_run_state(deps, status),
            ExecuteMsg::SetContract { utility_contract_name, contract, query, .. } => set_contract(deps, contract_name, contract, info.sender, query),
        },
        RESPONSE_BLOCK_SIZE,
    )
}

fn is_admin(deps: Deps, user: Addr, env: &Env) -> StdResult<()> {
    match ProtocolContract::may_load(deps.storage, UtilityContracts::AdminAuth)? {
        Some(admin_auth) => {
            match validate_admin(&deps.querier, AdminPermissions::UtilityRouterAdmin, user, admin_auth) {
                Ok(_) => Ok(_),
                Err(_) => Err(unauthorized_admin(user, AdminPermissions::UtilityRouterAdmin))
            }
        },
        None => Err(critical_admin_error())
    }
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    pad_query_result(
        match msg {
            QueryMsg::Status {  } => get_status(deps),
            QueryMsg::ForwardQuery { utility_name, query } => forward_query(dpes, utility_name, query),
            QueryMsg::GetContract { utility_name } => get_contract(deps, utility_name)
        },
        RESPONSE_BLOCK_SIZE,
    )
}