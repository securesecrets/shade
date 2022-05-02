use cosmwasm_std::{Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, to_binary};
use cosmwasm_math_compat::Uint128;
use shade_protocol::governance::assembly::AssemblyMsg;
use shade_protocol::governance::{MSG_VARIABLE, HandleAnswer};
use shade_protocol::governance::stored_id::ID;
use shade_protocol::utils::flexible_msg::FlexibleMsg;
use shade_protocol::utils::generic_response::ResponseStatus;
use shade_protocol::utils::storage::BucketStorage;

pub fn try_add_assembly_msg<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    name: String,
    msg: String,
    assemblies: Vec<Uint128>
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized())
    }

    let id = ID::add_assembly_msg(&mut deps.storage)?;

    // Check that assemblys exist
    for assembly in assemblies.iter() {
        if *assembly > ID::assembly(&deps.storage)? {
            return Err(StdError::generic_err("Given assembly does not exist"))
        }
    }

    AssemblyMsg {
        name,
        assemblies,
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
    assemblies: Option<Vec<Uint128>>
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized())
    }

    let mut assembly_msg = match AssemblyMsg::may_load(&mut deps.storage, &id)? {
        None => return Err(StdError::generic_err("AssemblyMsg not found")),
        Some(c) => c
    };

    if let Some(name) = name {
        assembly_msg.name = name;
    }

    if let Some(msg) = msg {
        assembly_msg.msg = FlexibleMsg::new(msg, MSG_VARIABLE);
    }

    if let Some(assemblies) = assemblies {
        assembly_msg.assemblies = assemblies;
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

pub fn try_add_assembly_msg_assemblies<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    id: Uint128,
    assemblies: Vec<Uint128>
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized())
    }

    let mut assembly_msg = AssemblyMsg::data(&mut deps.storage, &id)?;

    let assembly_id = ID::assembly(&deps.storage)?;
    for assembly in assemblies.iter() {
        if assembly < &assembly_id && !assembly_msg.assemblies.contains(assembly) {
            assembly_msg.assemblies.push(assembly.clone());
        }
    }

    AssemblyMsg::save_data(&mut deps.storage, &id, assembly_msg)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetAssemblyMsg {
            status: ResponseStatus::Success,
        })?),
    })
}