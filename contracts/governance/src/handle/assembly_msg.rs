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
        assembly::AssemblyMsg,
        stored_id::ID,
        HandleAnswer,
        MSG_VARIABLE,
    },
    utils::{
        flexible_msg::FlexibleMsg,
        generic_response::ResponseStatus,
        storage::default::BucketStorage,
    },
};

pub fn try_add_assembly_msg(
    deps: DepsMut,
    env: Env,
    name: String,
    msg: String,
    assemblies: Vec<Uint128>,
) -> StdResult<Response> {
    if info.sender != env.contract.address {
        return Err(StdError::unauthorized());
    }

    let id = ID::add_assembly_msg(deps.storage)?;

    // Check that assemblys exist
    for assembly in assemblies.iter() {
        if *assembly > ID::assembly(deps.storage)? {
            return Err(StdError::generic_err("Given assembly does not exist"));
        }
    }

    AssemblyMsg {
        name,
        assemblies,
        msg: FlexibleMsg::new(msg, MSG_VARIABLE),
    }
    .save(deps.storage, &id)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddAssemblyMsg {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_set_assembly_msg(
    deps: DepsMut,
    env: Env,
    id: Uint128,
    name: Option<String>,
    msg: Option<String>,
    assemblies: Option<Vec<Uint128>>,
) -> StdResult<Response> {
    if info.sender != env.contract.address {
        return Err(StdError::unauthorized());
    }

    let mut assembly_msg = match AssemblyMsg::may_load(deps.storage, &id)? {
        None => return Err(StdError::generic_err("AssemblyMsg not found")),
        Some(c) => c,
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

    assembly_msg.save(deps.storage, &id)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetAssemblyMsg {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_add_assembly_msg_assemblies(
    deps: DepsMut,
    env: Env,
    id: Uint128,
    assemblies: Vec<Uint128>,
) -> StdResult<Response> {
    if info.sender != env.contract.address {
        return Err(StdError::unauthorized());
    }

    let mut assembly_msg = AssemblyMsg::data(deps.storage, &id)?;

    let assembly_id = ID::assembly(deps.storage)?;
    for assembly in assemblies.iter() {
        if assembly < &assembly_id && !assembly_msg.assemblies.contains(assembly) {
            assembly_msg.assemblies.push(assembly.clone());
        }
    }

    AssemblyMsg::save_data(deps.storage, &id, assembly_msg)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetAssemblyMsg {
            status: ResponseStatus::Success,
        })?),
    })
}
