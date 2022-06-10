use cosmwasm_std::{Api, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier, QueryResult, StdError, StdResult, Storage, to_binary};
use secret_toolkit::utils::{pad_handle_result, pad_query_result};
use shade_protocol::contract_interfaces::query_auth::{Admin, RngSeed, ContractStatus, HandleMsg, InitMsg, QueryMsg};
use shade_protocol::utils::storage::plus::ItemStorage;
use crate::handle;
use crate::query;

// Used to pad up responses for better privacy.
pub const RESPONSE_BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    Admin(
        match msg.admin {
            None => env.message.sender,
            Some(admin) => admin
        }
    ).save(&mut deps.storage)?;

    RngSeed::new(msg.prng_seed).save(&mut deps.storage)?;

    ContractStatus::Default.save(&mut deps.storage)?;

    Ok(InitResponse {
        messages: vec![],
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    // Check what msgs are allowed
    let status = ContractStatus::load(&deps.storage)?;
    match status {
        // Do nothing
        ContractStatus::Default => {}
        // No permit interactions
        ContractStatus::DisablePermit => {
            match msg {
                HandleMsg::BlockPermitKey {..} => {
                    return Err(StdError::unauthorized())
                }
                _ => {}
            }
        }
        // No VK interactions
        ContractStatus::DisableVK => {
            match msg {
                HandleMsg::CreateViewingKey {..} | HandleMsg::SetViewingKey {..} => {
                    return Err(StdError::unauthorized())
                }
                _ => {}
            }
        }
        // Nothing
        ContractStatus::DisableAll => {match msg {
            HandleMsg::CreateViewingKey {..} | HandleMsg::SetViewingKey {..} | HandleMsg::BlockPermitKey {..} => {
                return Err(StdError::unauthorized())
            }
            _ => {}
        }}
    }

    pad_handle_result(
        match msg {
            HandleMsg::SetAdmin { admin, padding } => handle::try_set_admin(deps, env, admin),
            HandleMsg::SetRunState { state, padding } => handle::try_set_run_state(deps, env, state),
            HandleMsg::SetViewingKey { key, padding } => handle::try_set_viewing_key(deps, env, key),
            HandleMsg::CreateViewingKey { entropy, padding } => handle::try_create_viewing_key(deps, env, entropy),
            HandleMsg::BlockPermitKey { key, padding } => handle::try_block_permit_key(deps, env, key),
        },
        RESPONSE_BLOCK_SIZE
    )
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    pad_query_result(
        to_binary(
            &match msg {
                QueryMsg::Config { .. } => query::config(deps)?,
                QueryMsg::ValidateViewingKey { user, key } => query::validate_vk(deps, user, key)?,
                QueryMsg::ValidatePermit { permit } => query::validate_permit(deps, permit)?
            }
        ),
        RESPONSE_BLOCK_SIZE
    )
}