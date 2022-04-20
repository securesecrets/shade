use cosmwasm_std::{Api, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage, to_binary};
use cosmwasm_math_compat::Uint128;
use shade_protocol::governance::contract::AllowedContract;
use shade_protocol::governance::HandleAnswer;
use shade_protocol::governance::stored_id::ID;
use shade_protocol::utils::asset::Contract;
use shade_protocol::utils::generic_response::ResponseStatus;

pub fn try_add_contract<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    name: String,
    metadata: String,
    contract: Contract
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized())
    }

    let id = ID::add_contract(&mut deps.storage)?;
    AllowedContract {
        name,
        metadata,
        contract
    }.save(&mut deps.storage, &id)?;

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
    contract: Option<Contract>
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized())
    }

    if id > ID::contract(&deps.storage)? {
        return Err(StdError::generic_err("AllowedContract not found"))
    }

    let mut allowedContract = AllowedContract::load(&mut deps.storage, &id)?;

    if let Some(name) = name {
        allowedContract.name = name;
    }

    if let Some(metadata) = metadata {
        allowedContract.metadata = metadata;
    }

    if let Some(contract) = contract {
        allowedContract.contract = contract;
    }

    allowedContract.save(&mut deps.storage, &id)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddContract {
            status: ResponseStatus::Success,
        })?),
    })
}