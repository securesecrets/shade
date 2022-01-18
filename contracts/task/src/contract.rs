use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, 
    Env, Extern, HandleResponse, InitResponse, 
    Querier, StdResult, Storage, Uint128,
};
use shade_protocol::{
    task::{
        InitMsg, HandleMsg,
        QueryMsg, Config,
    },
    band::ReferenceData,
};
use crate::{
    state::{ config_w },
    query, handle,
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    let config = Config {
        admin: match msg.admin {
            None => { env.message.sender.clone() }
            Some(admin) => { admin }
        },
        band: msg.band,
        sscrt: msg.sscrt,
    };

    config_w(&mut deps.storage).save(&config)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {

    match msg {
        HandleMsg::UpdateConfig {
            config,
        } => handle::try_update_config(deps, env, admin, band),

        _ => {
        },
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {

    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
    }

}
