use cosmwasm_std::{Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, to_binary};
use secret_toolkit::snip20::register_receive_msg;
use shade_protocol::governance::{Config, HandleAnswer, RuntimeState};
use shade_protocol::utils::asset::Contract;
use shade_protocol::utils::generic_response::ResponseStatus;
use shade_protocol::utils::storage::default::SingletonStorage;

pub mod assembly;
pub mod proposal;
pub mod assembly_msg;
pub mod profile;
pub mod contract;

pub fn try_set_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    treasury: Option<HumanAddr>,
    vote_token: Option<Contract>,
    funding_token: Option<Contract>
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized())
    }

    let mut messages = vec![];
    let mut config = Config::load(&deps.storage)?;

    // Vote and funding tokens cannot be set to none after being set
    if let Some(vote_token) = vote_token {
        config.vote_token = Some(vote_token.clone());
        messages.push(register_receive_msg(
            env.contract_code_hash.clone(),
            None,
            255,
            vote_token.code_hash,
            vote_token.address
        )?);
    }

    if let Some(funding_token) = funding_token {
        config.funding_token = Some(funding_token.clone());
        messages.push(register_receive_msg(
            env.contract_code_hash.clone(),
            None,
            255,
            funding_token.code_hash,
            funding_token.address
        )?);
    }

    if let Some(treasury) = treasury {
        config.treasury = treasury;
    }

    config.save(&mut deps.storage)?;
    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_set_runtime_state<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    state: RuntimeState
) -> StdResult<HandleResponse> {
    todo!();
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetRuntimeState {
            status: ResponseStatus::Success,
        })?),
    })
}