use shade_protocol::c_std::{
    to_binary,
    Api,
    Env,
    Extern,
    HandleResponse,
    Querier,
    StdError,
    StdResult,
    Storage,
};
use shade_protocol::query_authentication::viewing_keys::ViewingKey;
use shade_protocol::secret_toolkit::utils::Query;
use shade_admin::admin::AuthorizedUsersResponse;
use shade_protocol::{
    contract_interfaces::query_auth::{
        auth::{HashedKey, Key, PermitKey},
        Admin,
        ContractStatus,
        HandleAnswer,
        RngSeed,
    },
    utils::{
        generic_response::ResponseStatus::Success,
        storage::plus::{ItemStorage, MapStorage},
    },
};
use shade_protocol::utils::asset::Contract;

fn user_authorized<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, env: Env) -> StdResult<bool> {
    let contract = Admin::load(&deps.storage)?.0;

    let authorized_users: AuthorizedUsersResponse = shade_admin::admin::QueryMsg::GetAuthorizedUsers {
        contract_address: env.contract.address.to_string()
    }.query(&deps.querier, contract.code_hash, contract.address)?;

    Ok(authorized_users.authorized_users.contains(&env.message.sender.to_string()))
}

pub fn try_set_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    admin: Contract,
) -> StdResult<HandleResponse> {
    if  !user_authorized(&deps, env)? {
        return Err(StdError::unauthorized());
    }

    Admin(admin).save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetAdminAuth { status: Success })?),
    })
}

pub fn try_set_run_state<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    state: ContractStatus,
) -> StdResult<HandleResponse> {
    if  !user_authorized(&deps, env)? {
        return Err(StdError::unauthorized());
    }

    state.save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetRunState { status: Success })?),
    })
}

pub fn try_create_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    entropy: String,
) -> StdResult<HandleResponse> {
    let seed = RngSeed::load(&deps.storage)?.0;

    let key = Key::generate(&env, seed.as_slice(), &entropy.as_ref());

    HashedKey(key.hash()).save(&mut deps.storage, env.message.sender)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CreateViewingKey { key: key.0 })?),
    })
}

pub fn try_set_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    key: String,
) -> StdResult<HandleResponse> {
    HashedKey(Key(key).hash()).save(&mut deps.storage, env.message.sender)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetViewingKey { status: Success })?),
    })
}

pub fn try_block_permit_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    key: String,
) -> StdResult<HandleResponse> {
    PermitKey::revoke(&mut deps.storage, key, env.message.sender)?;
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BlockPermitKey {
            status: Success,
        })?),
    })
}