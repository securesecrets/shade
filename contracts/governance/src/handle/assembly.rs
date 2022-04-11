use cosmwasm_std::{Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, to_binary};
use secret_cosmwasm_math_compat::Uint128;
use shade_protocol::governance::assembly::Assembly;
use shade_protocol::governance::HandleAnswer;
use shade_protocol::governance::profile::Profile;
use shade_protocol::governance::vote::Vote;
use shade_protocol::utils::generic_response::ResponseStatus;
use shade_protocol::utils::storage::BucketStorage;
use crate::state::ID;

pub fn try_assembly_vote<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    proposal: Uint128,
    vote: Vote
) -> StdResult<HandleResponse> {
    todo!();
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AssemblyVote {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_assembly_proposal<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    assembly_id: Uint128,
    metadata: String,
    contract_id: Option<Uint128>,
    assembly_msg_id: Option<Uint128>,
    variables: Option<Vec<String>>
) -> StdResult<HandleResponse> {

    // Get assembly
    let assembly = Assembly::may_load(&deps.storage, )

    // Check if public; everyone is allowed
    if assembly != Uint128::zero() {

    }

    todo!();
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AssemblyProposal {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_add_assembly<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    name: String,
    metadata: String,
    members: Vec<HumanAddr>,
    profile: Uint128
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized())
    }

    let id = ID::add_assembly(&mut deps.storage)?;

    // Check that profile exists
    if profile > ID::profile(&deps.storage)? {
        return Err(StdError::not_found(Profile))
    }

    Assembly {
        name,
        metadata,
        members,
        profile
    }.save(&mut deps.storage, &id)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddAssembly {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_set_assembly<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    id: Uint128,
    name: Option<String>,
    metadata: Option<String>,
    members: Option<Vec<HumanAddr>>,
    profile: Option<Uint128>
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized())
    }

    let mut assembly = match Assembly::may_load(&mut deps.storage, id.to_string().as_bytes())? {
        None => return Err(StdError::not_found(Assembly)),
        Some(c) => c
    };

    if let Some(name) = name {
        assembly.name = name;
    }

    if let Some(metadata) = metadata {
        assembly.metadata = metadata
    }

    if let Some(members) = members {
        assembly.members = members
    }

    if let Some(profile) = profile {
        // Check that profile exists
        if profile > ID::profile(&deps.storage)? {
            return Err(StdError::not_found(Profile))
        }
        assembly.profile = profile
    }

    assembly.save(&mut deps.storage, id)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetAssembly {
            status: ResponseStatus::Success,
        })?),
    })
}