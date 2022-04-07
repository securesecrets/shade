use cosmwasm_std::{Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdResult, Storage, to_binary};
use shade_protocol::governance::{HandleAnswer, RuntimeState};
use shade_protocol::utils::asset::Contract;
use shade_protocol::utils::generic_response::ResponseStatus;

pub mod committee;
pub mod proposal;
pub mod committee_msg;
pub mod profile;
pub mod contract;

pub fn try_set_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    treasury: Option<HumanAddr>,
    vote_token: Option<Contract>,
    funding_token: Option<Contract>
) -> StdResult<HandleResponse> {
    todo!();
    Ok(HandleResponse {
        messages: vec![],
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