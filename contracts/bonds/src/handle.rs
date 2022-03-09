use cosmwasm_std::{
    debug_print, from_binary, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse,
    HumanAddr, Querier, StdError, StdResult, Storage, Uint128,
};

use shade_protocol::bonds::{Config, HandleAnswer};
use shade_protocol::utils::generic_response::ResponseStatus;

use crate::state::{config_r, config_w};

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: Config,
) -> StdResult<HandleResponse> {
    let cur_config = config_r(&deps.storage).load()?;

    // Admin-only
    if env.message.sender != cur_config.admin {
        return Err(StdError::unauthorized());
    }

    config_w(&mut deps.storage).save(&config)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}