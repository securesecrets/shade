use crate::{handle, query};
use shade_protocol::{
    c_std::{
        shd_entry_point,
        to_binary,
        Binary,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
    },
    contract_interfaces::query_auth::{
        Admin,
        ContractStatus,
        ExecuteMsg,
        InstantiateMsg,
        QueryMsg,
        RngSeed,
    },
    utils::{pad_handle_result, pad_query_result, storage::plus::ItemStorage},
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
    Admin(msg.admin_auth).save(deps.storage)?;

    RngSeed::new(msg.prng_seed).save(deps.storage)?;

    ContractStatus::Default.save(deps.storage)?;

    Ok(Response::new())
}

#[shd_entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    // Check what msgs are allowed
    let status = ContractStatus::load(deps.storage)?;
    match status {
        // Do nothing
        ContractStatus::Default => {}
        // No permit interactions
        ContractStatus::DisablePermit => match msg {
            ExecuteMsg::BlockPermitKey { .. } => return Err(StdError::generic_err("unauthorized")),
            _ => {}
        },
        // No VK interactions
        ContractStatus::DisableVK => match msg {
            ExecuteMsg::CreateViewingKey { .. } | ExecuteMsg::SetViewingKey { .. } => {
                return Err(StdError::generic_err("unauthorized"));
            }
            _ => {}
        },
        // Nothing
        ContractStatus::DisableAll => match msg {
            ExecuteMsg::CreateViewingKey { .. }
            | ExecuteMsg::SetViewingKey { .. }
            | ExecuteMsg::BlockPermitKey { .. } => {
                return Err(StdError::generic_err("unauthorized"));
            }
            _ => {}
        },
    }

    pad_handle_result(
        match msg {
            ExecuteMsg::SetAdminAuth { admin, .. } => handle::try_set_admin(deps, env, info, admin),
            ExecuteMsg::SetRunState { state, .. } => {
                handle::try_set_run_state(deps, env, info, state)
            }
            ExecuteMsg::SetViewingKey { key, .. } => {
                handle::try_set_viewing_key(deps, env, info, key)
            }
            ExecuteMsg::CreateViewingKey { entropy, .. } => {
                handle::try_create_viewing_key(deps, env, info, entropy)
            }
            ExecuteMsg::BlockPermitKey { key, .. } => {
                handle::try_block_permit_key(deps, env, info, key)
            }
        },
        RESPONSE_BLOCK_SIZE,
    )
}

#[shd_entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    let status = ContractStatus::load(deps.storage)?;
    match status {
        // Do nothing
        ContractStatus::Default => {}
        // No permit interactions
        ContractStatus::DisablePermit => {
            if let QueryMsg::ValidatePermit { .. } = msg {
                return Err(StdError::generic_err("unauthorized"));
            }
        }
        // No VK interactions
        ContractStatus::DisableVK => {
            if let QueryMsg::ValidateViewingKey { .. } = msg {
                return Err(StdError::generic_err("unauthorized"));
            }
        }
        // Nothing
        ContractStatus::DisableAll => {
            if let QueryMsg::Config { .. } = msg {
            } else {
                return Err(StdError::generic_err("unauthorized"));
            }
        }
    }

    pad_query_result(
        to_binary(&match msg {
            QueryMsg::Config { .. } => query::config(deps)?,
            QueryMsg::ValidateViewingKey { user, key } => query::validate_vk(deps, user, key)?,
            QueryMsg::ValidatePermit { permit } => query::validate_permit(deps, permit)?,
        }),
        RESPONSE_BLOCK_SIZE,
    )
}
