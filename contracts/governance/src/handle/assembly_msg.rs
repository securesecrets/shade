use shade_protocol::{
    c_std::{to_binary, DepsMut, Env, MessageInfo, Response, StdResult},
    contract_interfaces::governance::{
        assembly::AssemblyMsg,
        stored_id::ID,
        ExecuteAnswer,
        MSG_VARIABLE,
    },
    governance::errors::Error,
    utils::{flexible_msg::FlexibleMsg, generic_response::ResponseStatus},
};

pub fn try_add_assembly_msg(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    name: String,
    msg: String,
    assemblies: Vec<u16>,
) -> StdResult<Response> {
    let id = ID::add_assembly_msg(deps.storage)?;

    // Check that assemblys exist
    for assembly in assemblies.iter() {
        if *assembly > ID::assembly(deps.storage)? {
            return Err(Error::item_not_found(vec![
                &assembly.to_string(),
                "Assembly",
            ]));
        }
    }

    AssemblyMsg {
        name,
        assemblies,
        msg: FlexibleMsg::new(msg, MSG_VARIABLE),
    }
    .save(deps.storage, id)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::AddAssemblyMsg {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn try_set_assembly_msg(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    id: u16,
    name: Option<String>,
    msg: Option<String>,
    assemblies: Option<Vec<u16>>,
) -> StdResult<Response> {
    let mut assembly_msg = match AssemblyMsg::may_load(deps.storage, id)? {
        None => return Err(Error::item_not_found(vec![&id.to_string(), "AssemblyMsg"])),
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

    assembly_msg.save(deps.storage, id)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::SetAssemblyMsg {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn try_add_assembly_msg_assemblies(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    id: u16,
    assemblies: Vec<u16>,
) -> StdResult<Response> {
    let mut assembly_msg = AssemblyMsg::data(deps.storage, id)?;

    let assembly_id = ID::assembly(deps.storage)?;
    for assembly in assemblies.iter() {
        if assembly < &assembly_id && !assembly_msg.assemblies.contains(assembly) {
            assembly_msg.assemblies.push(assembly.clone());
        }
    }

    AssemblyMsg::save_data(deps.storage, id, assembly_msg)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::SetAssemblyMsg {
            status: ResponseStatus::Success,
        })?),
    )
}
