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
use shade_protocol::snip20::helpers::register_receive;
use shade_protocol::{
    contract_interfaces::governance::{Config, HandleAnswer, RuntimeState},
    utils::{
        asset::Contract,
        generic_response::ResponseStatus,
        storage::default::SingletonStorage,
    },
};

pub mod assembly;
pub mod assembly_msg;
pub mod contract;
pub mod profile;
pub mod proposal;

pub fn try_set_config(
    deps: DepsMut,
    env: Env,
    treasury: Option<Addr>,
    vote_token: Option<Contract>,
    funding_token: Option<Contract>,
) -> StdResult<Response> {
    if info.sender != env.contract.address {
        return Err(StdError::unauthorized());
    }

    let mut messages = vec![];
    let mut config = Config::load(deps.storage)?;

    // Vote and funding tokens cannot be set to none after being set
    if let Some(vote_token) = vote_token {
        config.vote_token = Some(vote_token.clone());
        messages.push(register_receive(
            env.contract_code_hash.clone(),
            None,
            vote_token
        )?);
    }

    if let Some(funding_token) = funding_token {
        config.funding_token = Some(funding_token.clone());
        messages.push(register_receive(
            env.contract_code_hash.clone(),
            None,
            funding_token
        )?);
    }

    if let Some(treasury) = treasury {
        config.treasury = treasury;
    }

    config.save(deps.storage)?;
    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetConfig {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_set_runtime_state(
    deps: DepsMut,
    env: Env,
    state: RuntimeState,
) -> StdResult<Response> {
    todo!();
    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetRuntimeState {
            status: ResponseStatus::Success,
        })?))
}
