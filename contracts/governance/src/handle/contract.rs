use shade_protocol::{
    c_std::{
        to_binary,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
        Uint128,
    },
    contract_interfaces::governance::{contract::AllowedContract, stored_id::ID, HandleAnswer},
    utils::{asset::Contract, generic_response::ResponseStatus},
};

pub fn try_add_contract(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    metadata: String,
    contract: Contract,
    assemblies: Option<Vec<Uint128>>,
) -> StdResult<Response> {
    if info.sender != env.contract.address {
        return Err(StdError::generic_err("unauthorized"));
    }

    let id = ID::add_contract(deps.storage)?;

    if let Some(ref assemblies) = assemblies {
        let assembly_id = ID::assembly(deps.storage)?;
        for assembly in assemblies.iter() {
            if assembly > &assembly_id {
                return Err(StdError::generic_err("Assembly does not exist"));
            }
        }
    }

    AllowedContract {
        name,
        metadata,
        contract,
        assemblies,
    }
    .save(deps.storage, &id)?;

    Ok(
        Response::new().set_data(to_binary(&HandleAnswer::AddContract {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn try_set_contract(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: Uint128,
    name: Option<String>,
    metadata: Option<String>,
    contract: Option<Contract>,
    disable_assemblies: bool,
    assemblies: Option<Vec<Uint128>>,
) -> StdResult<Response> {
    if info.sender != env.contract.address {
        return Err(StdError::generic_err("unauthorized"));
    }

    if id > ID::contract(deps.storage)? {
        return Err(StdError::generic_err("AllowedContract not found"));
    }

    let mut allowed_contract = AllowedContract::load(deps.storage, &id)?;

    if let Some(name) = name {
        allowed_contract.name = name;
    }

    if let Some(metadata) = metadata {
        allowed_contract.metadata = metadata;
    }

    if let Some(contract) = contract {
        allowed_contract.contract = contract;
    }

    if disable_assemblies {
        allowed_contract.assemblies = None;
    } else {
        if let Some(assemblies) = assemblies {
            let assembly_id = ID::assembly(deps.storage)?;
            for assembly in assemblies.iter() {
                if assembly > &assembly_id {
                    return Err(StdError::generic_err("Assembly does not exist"));
                }
            }
            allowed_contract.assemblies = Some(assemblies);
        }
    }

    allowed_contract.save(deps.storage, &id)?;

    Ok(
        Response::new().set_data(to_binary(&HandleAnswer::AddContract {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn try_add_contract_assemblies(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: Uint128,
    assemblies: Vec<Uint128>,
) -> StdResult<Response> {
    if info.sender != env.contract.address {
        return Err(StdError::generic_err("unauthorized"));
    }

    if id > ID::contract(deps.storage)? {
        return Err(StdError::generic_err("AllowedContract not found"));
    }

    let mut allowed_contract = AllowedContract::data(deps.storage, &id)?;

    if let Some(mut old_assemblies) = allowed_contract.assemblies {
        let assembly_id = ID::assembly(deps.storage)?;
        for assembly in assemblies.iter() {
            if assembly <= &assembly_id && !old_assemblies.contains(assembly) {
                old_assemblies.push(assembly.clone());
            }
        }
        allowed_contract.assemblies = Some(old_assemblies);
    } else {
        return Err(StdError::generic_err(
            "Assembly support is disabled in this contract",
        ));
    }

    AllowedContract::save_data(deps.storage, &id, allowed_contract)?;

    Ok(
        Response::new().set_data(to_binary(&HandleAnswer::AddContract {
            status: ResponseStatus::Success,
        })?),
    )
}
