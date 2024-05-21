use shade_protocol::{
    admin::helpers::{validate_admin, AdminPermissions},
    c_std::{to_binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult},
    contract_interfaces::query_auth::{
        auth::{HashedKey, Key, PermitKey},
        Admin, ContractStatus, ExecuteAnswer, RngSeed,
    },
    query_authentication::viewing_keys::ViewingKey,
    utils::{
        generic_response::ResponseStatus::Success,
        storage::plus::{ItemStorage, MapStorage},
    },
};

use shade_protocol::utils::asset::Contract;

fn user_authorized(deps: &Deps, _env: Env, info: &MessageInfo) -> StdResult<()> {
    let admin = Admin::load(deps.storage)?.0;

    if admin == info.sender {
        Ok(())
    } else {
        Err(StdError::generic_err("Unauthorized"))
    }
}

pub fn try_set_admin(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    admin: String,
) -> StdResult<Response> {
    user_authorized(&deps.as_ref(), env, &info)?;

    Admin(deps.api.addr_validate(&admin)?).save(deps.storage)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::SetAdmin { status: Success })?))
}

pub fn try_set_run_state(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    state: ContractStatus,
) -> StdResult<Response> {
    user_authorized(&deps.as_ref(), env, &info)?;

    state.save(deps.storage)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::SetRunState { status: Success })?))
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

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::CreateViewingKey { key: key.0 })?))
}

pub fn try_set_viewing_key(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    key: String,
) -> StdResult<Response> {
    HashedKey(Key(key).hash()).save(deps.storage, info.sender)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::SetViewingKey {
            status: Success,
        })?),
    )
}

pub fn try_block_permit_key(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    key: String,
) -> StdResult<Response> {
    PermitKey::revoke(deps.storage, key, info.sender)?;
    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::BlockPermitKey {
            status: Success,
        })?),
    )
}
