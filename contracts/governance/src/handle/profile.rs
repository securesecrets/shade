use shade_protocol::{
    c_std::{to_binary, Api, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage},
    contract_interfaces::governance::{
        profile::{Profile, UpdateProfile},
        stored_id::ID,
        HandleAnswer,
    },
    math_compat::Uint128,
    utils::generic_response::ResponseStatus,
};

pub fn try_add_profile<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    profile: Profile,
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized());
    }

    let id = ID::add_profile(&mut deps.storage)?;
    profile.save(&mut deps.storage, &id)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddProfile {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_set_profile<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    id: Uint128,
    new_profile: UpdateProfile,
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized());
    }

    let mut profile = match Profile::may_load(&mut deps.storage, &id)? {
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

    profile.save(&mut deps.storage, &id)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetProfile {
            status: ResponseStatus::Success,
        })?),
    })
}
