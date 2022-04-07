use cosmwasm_std::{Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, to_binary, Uint128};
use shade_protocol::governance::committee::Committee;
use shade_protocol::governance::HandleAnswer;
use shade_protocol::governance::profile::Profile;
use shade_protocol::governance::vote::Vote;
use shade_protocol::utils::generic_response::ResponseStatus;
use shade_protocol::utils::storage::BucketStorage;
use crate::state::ID;

pub fn try_committee_vote<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    proposal: Uint128,
    vote: Vote
) -> StdResult<HandleResponse> {
    todo!();
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CommitteeVote {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_committee_proposal<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    committee_id: Uint128,
    metadata: String,
    contract_id: Option<Uint128>,
    committee_msg_id: Option<Uint128>,
    variables: Option<Vec<String>>
) -> StdResult<HandleResponse> {

    // Get committee
    let committee = Committee::may_load(&deps.storage, )

    // Check if public; everyone is allowed
    if committee != Uint128::zero() {

    }

    todo!();
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CommitteeProposal {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_add_committee<S: Storage, A: Api, Q: Querier>(
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

    let id = ID::add_committee(&mut deps.storage)?;

    // Check that profile exists
    if profile > ID::profile(&deps.storage)? {
        return Err(StdError::not_found(Profile))
    }

    Committee {
        name,
        metadata,
        members,
        profile
    }.save(&mut deps.storage, &id)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddCommittee {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_set_committee<S: Storage, A: Api, Q: Querier>(
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

    let mut committee = match Committee::may_load(&mut deps.storage, id.to_string().as_bytes())? {
        None => return Err(StdError::not_found(Committee)),
        Some(c) => c
    };

    if let Some(name) = name {
        committee.name = name;
    }

    if let Some(metadata) = metadata {
        committee.metadata = metadata
    }

    if let Some(members) = members {
        committee.members = members
    }

    if let Some(profile) = profile {
        // Check that profile exists
        if profile > ID::profile(&deps.storage)? {
            return Err(StdError::not_found(Profile))
        }
        committee.profile = profile
    }

    committee.save(&mut deps.storage, id)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetCommittee {
            status: ResponseStatus::Success,
        })?),
    })
}