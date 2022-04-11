use cosmwasm_std::{Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, to_binary};
use secret_cosmwasm_math_compat::Uint128;
use shade_protocol::governance::assembly::AssemblyMsg;
use shade_protocol::governance::{MSG_VARIABLE, HandleAnswer};
use shade_protocol::utils::flexible_msg::FlexibleMsg;
use shade_protocol::utils::generic_response::ResponseStatus;
use shade_protocol::utils::storage::BucketStorage;
use crate::state::ID;

pub fn try_add_assembly_msg<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    name: String,
    msg: String,
    assemblys: Vec<Uint128>
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized())
    }

    let id = ID::add_assembly_msg(&mut deps.storage)?;

    // Check that assemblys exist
    for assembly in assemblys {
        if assembly > ID::assembly(&deps.storage)? {
            return Err(StdError::generic_err("Given assembly does not exist"))
        }
    }

    AssemblyMsg {
        name,
        assemblys,
        msg: FlexibleMsg::new(msg, MSG_VARIABLE)
    }.save(&mut deps.storage, &id)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddAssemblyMsg {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_set_assembly_msg<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    id: Uint128,
    name: Option<String>,
    msg: Option<String>,
    assemblys: Option<Vec<Uint128>>
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized())
    }

    let mut assembly_msg = match AssemblyMsg::may_load(&mut deps.storage, id.to_string().as_bytes())? {
        None => return Err(StdError::not_found(AssemblyMsg)),
        Some(c) => c
    };

    if let Some(name) = name {
        assembly_msg.name = name;
    }

    if let Some(msg) = msg {
        assembly_msg.msg = FlexibleMsg::new(msg, MSG_VARIABLE);
    }

    if let Some(assemblys) = assemblys {
        assembly_msg.assemblys = assemblys;
    }

    assembly_msg.save(&mut deps.storage, &id)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetAssemblyMsg {
            status: ResponseStatus::Success,
        })?),
    })
}