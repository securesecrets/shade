use crate::{handle, query};
use shade_protocol::c_std::{
    to_binary,
    Api,
    Env,
    DepsMut,
    Response,
    Querier,

    StdError,
    StdResult,
    Storage,
};
use shade_protocol::utils::{pad_handle_result, pad_query_result};
use shade_protocol::{
    contract_interfaces::query_auth::{
        Admin,
        ContractStatus,
        ExecuteMsg,
        InstantiateMsg,
        QueryMsg,
        RngSeed,
    },
    utils::storage::plus::ItemStorage,
};

// Used to pad up responses for better privacy.
pub const RESPONSE_BLOCK_SIZE: usize = 256;

pub fn init(
    deps: DepsMut,
    _env: Env,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    Admin(msg.admin_auth)
    .save(deps.storage)?;

    RngSeed::new(msg.prng_seed).save(deps.storage)?;

    ContractStatus::Default.save(deps.storage)?;

    Ok(Response::new())
}

pub fn handle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    // Check what msgs are allowed
    let status = ContractStatus::load(&deps.storage)?;
    match status {
        // Do nothing
        ContractStatus::Default => {}
        // No permit interactions
        ContractStatus::DisablePermit => match msg {
            ExecuteMsg::BlockPermitKey { .. } => return Err(StdError::unauthorized()),
            _ => {}
        },
        // No VK interactions
        ContractStatus::DisableVK => match msg {
            ExecuteMsg::CreateViewingKey { .. } | ExecuteMsg::SetViewingKey { .. } => {
                return Err(StdError::unauthorized());
            }
            _ => {}
        },
        // Nothing
        ContractStatus::DisableAll => match msg {
            ExecuteMsg::CreateViewingKey { .. }
            | ExecuteMsg::SetViewingKey { .. }
            | ExecuteMsg::BlockPermitKey { .. } => return Err(StdError::unauthorized()),
            _ => {}
        },
    }

    pad_handle_result(
        match msg {
            ExecuteMsg::SetAdminAuth { admin, .. } => handle::try_set_admin(deps, env, admin),
            ExecuteMsg::SetRunState { state, .. } => handle::try_set_run_state(deps, env, state),
            ExecuteMsg::SetViewingKey { key, .. } => handle::try_set_viewing_key(deps, env, key),
            ExecuteMsg::CreateViewingKey { entropy, .. } => {
                handle::try_create_viewing_key(deps, env, entropy)
            }
            ExecuteMsg::BlockPermitKey { key, .. } => handle::try_block_permit_key(deps, env, key),
        },
        RESPONSE_BLOCK_SIZE,
    )
}

pub fn query(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
    let status = ContractStatus::load(&deps.storage)?;
    match status {
        // Do nothing
        ContractStatus::Default => {}
        // No permit interactions
        ContractStatus::DisablePermit => {
            if let QueryMsg::ValidatePermit { .. } = msg {
                return Err(StdError::unauthorized());
            }
        }
        // No VK interactions
        ContractStatus::DisableVK => {
            if let QueryMsg::ValidateViewingKey { .. } = msg {
                return Err(StdError::unauthorized());
            }
        }
        // Nothing
        ContractStatus::DisableAll => {
            if let QueryMsg::Config { .. } = msg {
            } else {
                return Err(StdError::unauthorized());
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