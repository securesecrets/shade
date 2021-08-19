use cosmwasm_std::{
    to_binary, Api,
    Env, Extern, HandleResponse,
    Querier, StdResult, StdError, Storage,
    HumanAddr,
};
use shade_protocol::{
    oracle::{
        HandleAnswer,
    },
    asset::Contract,
    generic_response::ResponseStatus,
};
use crate::state::{
    config_w, config_r,
};


pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Option<HumanAddr>,
    band: Option<Contract>,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    if env.message.sender != config.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Save new info
    let mut config = config_w(&mut deps.storage);
    config.update(|mut state| {
        if let Some(owner) = owner {
            state.owner = owner;
        }
        if let Some(band) = band {
            state.band = band;
        }

        Ok(state)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::UpdateConfig{
            status: ResponseStatus::Success } )? )
    })
}
