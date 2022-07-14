use shade_protocol::c_std::Uint128;
use shade_protocol::c_std::{
    to_binary,
    Api,
    Env,
    DepsMut,
    Response,
    Addr,
    Querier,
    StdError,
    StdResult,
    Storage,
};
use shade_protocol::{
    contract_interfaces::governance::{
        profile::{Profile, UpdateProfile, UpdateVoteProfile, VoteProfile},
        stored_id::ID,
        HandleAnswer,
    },
    utils::{generic_response::ResponseStatus, storage::default::BucketStorage},
};

pub fn try_add_profile<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    profile: Profile,
) -> StdResult<Response> {
    if info.sender != env.contract.address {
        return Err(StdError::unauthorized());
    }

    let id = ID::add_profile(deps.storage)?;
    profile.save(deps.storage, &id)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddProfile {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_set_profile<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    id: Uint128,
    new_profile: UpdateProfile,
) -> StdResult<Response> {
    if info.sender != env.contract.address {
        return Err(StdError::unauthorized());
    }

    let mut profile = match Profile::may_load(deps.storage, &id)? {
        None => return Err(StdError::generic_err("Profile not found")),
        Some(p) => p,
    };

    if let Some(name) = new_profile.name {
        profile.name = name;
    }

    if let Some(enabled) = new_profile.enabled {
        profile.enabled = enabled;
    }

    if new_profile.disable_assembly {
        profile.assembly = None;
    } else if let Some(assembly) = new_profile.assembly {
        profile.assembly = Some(assembly.update_profile(&profile.assembly)?)
    }

    if new_profile.disable_funding {
        profile.funding = None;
    } else if let Some(funding) = new_profile.funding {
        profile.funding = Some(funding.update_profile(&profile.funding)?)
    }

    if new_profile.disable_token {
        profile.token = None;
    } else if let Some(token) = new_profile.token {
        profile.token = Some(token.update_profile(&profile.token)?)
    }

    if let Some(cancel_deadline) = new_profile.cancel_deadline {
        profile.cancel_deadline = cancel_deadline;
    }

    profile.save(deps.storage, &id)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetProfile {
            status: ResponseStatus::Success,
        })?),
    })
}
