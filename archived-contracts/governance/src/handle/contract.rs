use shade_protocol::{
    c_std::{to_binary, DepsMut, Env, MessageInfo, Response, StdResult},
    contract_interfaces::governance::{contract::AllowedContract, stored_id::ID, ExecuteAnswer},
    governance::errors::Error,
    utils::{asset::Contract, generic_response::ResponseStatus},
};

pub fn try_add_contract(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    name: String,
    metadata: String,
    contract: Contract,
    assemblies: Option<Vec<u16>>,
) -> StdResult<Response> {
    let id = ID::add_contract(deps.storage)?;

    if let Some(ref assemblies) = assemblies {
        let assembly_id = ID::assembly(deps.storage)?;
        for assembly in assemblies.iter() {
            if assembly > &assembly_id {
                return Err(Error::item_not_found(vec![
                    &assembly.to_string(),
                    "Assembly",
                ]));
            }
        }
    }

    AllowedContract {
        name,
        metadata,
        contract,
        assemblies,
    }
    .save(deps.storage, id)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::AddContract {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn try_set_contract(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    id: u16,
    name: Option<String>,
    metadata: Option<String>,
    contract: Option<Contract>,
    disable_assemblies: bool,
    assemblies: Option<Vec<u16>>,
) -> StdResult<Response> {
    if id > ID::contract(deps.storage)? {
        return Err(Error::item_not_found(vec![&id.to_string(), "Contract"]));
    }

    let mut allowed_contract = AllowedContract::load(deps.storage, id)?;

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
                    return Err(Error::item_not_found(vec![
                        &assembly.to_string(),
                        "Assembly",
                    ]));
                }
            }
            allowed_contract.assemblies = Some(assemblies);
        }
    }

    allowed_contract.save(deps.storage, id)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::AddContract {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn try_add_contract_assemblies(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    id: u16,
    assemblies: Vec<u16>,
) -> StdResult<Response> {
    if id > ID::contract(deps.storage)? {
        return Err(Error::item_not_found(vec![&id.to_string(), "Contract"]));
    }

    let mut allowed_contract = AllowedContract::data(deps.storage, id)?;

    if let Some(mut old_assemblies) = allowed_contract.assemblies {
        let assembly_id = ID::assembly(deps.storage)?;
        for assembly in assemblies.iter() {
            if assembly <= &assembly_id && !old_assemblies.contains(assembly) {
                old_assemblies.push(assembly.clone());
            }
        }
        allowed_contract.assemblies = Some(old_assemblies);
    } else {
        return Err(Error::contract_disabled(vec![]));
    }

    AllowedContract::save_data(deps.storage, id, allowed_contract)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::AddContractAssemblies {
            status: ResponseStatus::Success,
        })?),
    )
}
