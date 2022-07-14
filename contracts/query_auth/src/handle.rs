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
use shade_protocol::query_authentication::viewing_keys::ViewingKey;
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

fn user_authorized(deps: Deps, env: Env) -> StdResult<bool> {
    let contract = Admin::load(deps.storage)?.0;

    let authorized_users: AuthorizedUsersResponse = shade_admin::admin::QueryMsg::GetAuthorizedUsers {
        contract_address: env.contract.address.to_string()
    }.query(&deps.querier, contract.code_hash, contract.address)?;

    Ok(authorized_users.authorized_users.contains(&info.sender.to_string()))
}

pub fn try_set_admin(
    deps: DepsMut,
    env: Env,
    admin: Contract,
) -> StdResult<Response> {
    if  !user_authorized(&deps, env)? {
        return Err(StdError::unauthorized());
    }

    Admin(admin).save(deps.storage)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetAdminAuth { status: Success })?),
    })
}

pub fn try_set_run_state(
    deps: DepsMut,
    env: Env,
    state: ContractStatus,
) -> StdResult<Response> {
    if  !user_authorized(&deps, env)? {
        return Err(StdError::unauthorized());
    }

    state.save(deps.storage)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetRunState { status: Success })?),
    })
}

pub fn try_create_viewing_key(
    deps: DepsMut,
    env: Env,
    entropy: String,
) -> StdResult<Response> {
    let seed = RngSeed::load(deps.storage)?.0;

    let key = Key::generate(&env, seed.as_slice(), &entropy.as_ref());

    HashedKey(key.hash()).save(deps.storage, info.sender)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CreateViewingKey { key: key.0 })?),
    })
}

pub fn try_set_viewing_key(
    deps: DepsMut,
    env: Env,
    key: String,
) -> StdResult<Response> {
    HashedKey(Key(key).hash()).save(deps.storage, info.sender)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetViewingKey { status: Success })?),
    })
}

pub fn try_block_permit_key(
    deps: DepsMut,
    env: Env,
    key: String,
) -> StdResult<Response> {
    PermitKey::revoke(deps.storage, key, info.sender)?;
    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::BlockPermitKey {
            status: Success,
        })?),
    })
}