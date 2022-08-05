use shade_protocol::c_std::{to_binary, Api, Env, DepsMut, Response, Querier, StdError, StdResult, Storage, Deps, MessageInfo};
use shade_protocol::query_authentication::viewing_keys::ViewingKey;
use shade_protocol::{
    contract_interfaces::query_auth::{
        auth::{HashedKey, Key, PermitKey},
        Admin,
        ContractStatus,
        HandleAnswer,
        RngSeed,
        SHADE_QUERY_AUTH_ADMIN,
    },
    utils::{
        generic_response::ResponseStatus::Success,
        storage::plus::{ItemStorage, MapStorage},
    },
};
use shade_protocol::admin::validate_permission;

use shade_protocol::utils::asset::Contract;

fn user_authorized(deps: &Deps, env: Env, info: &MessageInfo) -> StdResult<()> {
    let contract = Admin::load(deps.storage)?.0;

    validate_permission(
        &deps.querier,
        SHADE_QUERY_AUTH_ADMIN,
        &info.sender,
        &contract
    )
}

pub fn try_set_admin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    admin: Contract,
) -> StdResult<Response> {
    user_authorized(&deps.as_ref(), env, &info)?;

    Admin(admin).save(deps.storage)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetAdminAuth { status: Success })?))
}

pub fn try_set_run_state(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    state: ContractStatus,
) -> StdResult<Response> {
    user_authorized(&deps.as_ref(), env, &info)?;

    state.save(deps.storage)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetRunState { status: Success })?))
}

pub fn try_create_viewing_key(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    entropy: String,
) -> StdResult<Response> {
    let seed = RngSeed::load(deps.storage)?.0;

    let key = Key::generate(&info, &env, seed.as_slice(), &entropy.as_ref());

    HashedKey(key.hash()).save(deps.storage, info.sender)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::CreateViewingKey { key: key.0 })?))
}

pub fn try_set_viewing_key(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    key: String,
) -> StdResult<Response> {
    HashedKey(Key(key).hash()).save(deps.storage, info.sender)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetViewingKey { status: Success })?))
}

pub fn try_block_permit_key(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    key: String,
) -> StdResult<Response> {
    PermitKey::revoke(deps.storage, key, info.sender)?;
    Ok(Response::new().set_data(to_binary(&HandleAnswer::BlockPermitKey {
            status: Success,
        })?))
}