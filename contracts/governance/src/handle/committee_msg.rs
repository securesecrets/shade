use cosmwasm_std::{Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, to_binary, Uint128};
use shade_protocol::governance::committee::CommitteeMsg;
use shade_protocol::governance::{MSG_VARIABLE, HandleAnswer};
use shade_protocol::utils::flexible_msg::FlexibleMsg;
use shade_protocol::utils::generic_response::ResponseStatus;
use shade_protocol::utils::storage::BucketStorage;
use crate::state::ID;

pub fn try_add_committee_msg<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    name: String,
    msg: String,
    committees: Vec<Uint128>
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized())
    }

    let id = ID::add_committee_msg(&mut deps.storage)?;

    // Check that committees exist
    for committee in committees {
        if committee > ID::committee(&deps.storage)? {
            return Err(StdError::generic_err("Given committee does not exist"))
        }
    }

    CommitteeMsg {
        name,
        committees,
        msg: FlexibleMsg::new(msg, MSG_VARIABLE)
    }.save(&mut deps.storage, &id)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddCommitteeMsg {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_set_committee_msg<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    id: Uint128,
    name: Option<String>,
    msg: Option<String>,
    committees: Option<Vec<Uint128>>
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized())
    }

    let mut committee_msg = match CommitteeMsg::may_load(&mut deps.storage, id.to_string().as_bytes())? {
        None => return Err(StdError::not_found(CommitteeMsg)),
        Some(c) => c
    };

    if let Some(name) = name {
        committee_msg.name = name;
    }

    if let Some(msg) = msg {
        committee_msg.msg = FlexibleMsg::new(msg, MSG_VARIABLE);
    }

    if let Some(committees) = committees {
        committee_msg.committees = committees;
    }

    committee_msg.save(&mut deps.storage, &id)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetCommitteeMsg {
            status: ResponseStatus::Success,
        })?),
    })
}