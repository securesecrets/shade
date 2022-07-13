use shade_protocol::{
    c_std::{to_binary, Api, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage},
    contract_interfaces::governance::{contract::AllowedContract, stored_id::ID, HandleAnswer},
    math_compat::Uint128,
    utils::{asset::Contract, generic_response::ResponseStatus},
};

pub fn try_add_contract<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    name: String,
    metadata: String,
    contract: Contract,
    assemblies: Option<Vec<Uint128>>,
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized());
    }

    let id = ID::add_contract(&mut deps.storage)?;

    if let Some(ref assemblies) = assemblies {
        let assembly_id = ID::assembly(&deps.storage)?;
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
    .save(&mut deps.storage, &id)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddContract {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_set_contract<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    id: Uint128,
    name: Option<String>,
    metadata: Option<String>,
    contract: Option<Contract>,
    disable_assemblies: bool,
    assemblies: Option<Vec<Uint128>>,
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized());
    }

    if id > ID::contract(&deps.storage)? {
        return Err(StdError::generic_err("AllowedContract not found"));
    }

    let mut allowed_contract = AllowedContract::load(&mut deps.storage, &id)?;

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
            let assembly_id = ID::assembly(&deps.storage)?;
            for assembly in assemblies.iter() {
                if assembly > &assembly_id {
                    return Err(StdError::generic_err("Assembly does not exist"));
                }
            }
            allowed_contract.assemblies = Some(assemblies);
        }
    }

    allowed_contract.save(&mut deps.storage, &id)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddContract {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_add_contract_assemblies<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    id: Uint128,
    assemblies: Vec<Uint128>,
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized());
    }

    if id > ID::contract(&deps.storage)? {
        return Err(StdError::generic_err("AllowedContract not found"));
    }

    let mut allowed_contract = AllowedContract::data(&mut deps.storage, &id)?;

    if let Some(mut old_assemblies) = allowed_contract.assemblies {
        let assembly_id = ID::assembly(&deps.storage)?;
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

    AllowedContract::save_data(&mut deps.storage, &id, allowed_contract)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddContract {
            status: ResponseStatus::Success,
        })?),
    })
}
