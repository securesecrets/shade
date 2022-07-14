use shade_protocol::c_std::{
    debug_print,
    to_binary,
    Api,
    Binary,
    Env,
    DepsMut,
    Response,
    Querier,
    StdResult,
    Storage,
    Uint128,
};
use shade_protocol::snip20::helpers::{register_receive_msg, token_info, token_config_query};

use shade_protocol::contract_interfaces::{
    mint::mint_router::{Config, HandleMsg, InitMsg, QueryMsg},
    snip20::helpers::Snip20Asset,
};

use crate::{
    handle,
    query,
    state::{config_w, current_assets_w},
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> StdResult<Response> {
    let config = Config {
        admin: match msg.admin {
            None => info.sender.clone(),
            Some(admin) => admin,
        },
        path: msg.path,
    };

    config_w(&mut deps.storage).save(&config)?;
    //current_assets_w(&mut deps.storage).save(&vec![])?;

    let mut messages = vec![];

    if config.path.len() > 0 {
        //messages.append(&mut handle::update_entry_assets(deps, env, config.path[0].clone())?);
        messages.append(&mut handle::build_path(deps, env, config.path.clone())?);
    }

    Ok(Response {
        messages,
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    msg: HandleMsg,
) -> StdResult<Response> {
    match msg {
        HandleMsg::UpdateConfig { config } => handle::try_update_config(deps, env, config),
        HandleMsg::Receive {
            sender,
            from,
            amount,
            msg,
            ..
        } => handle::receive(deps, env, sender, from, amount, msg),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Assets {} => to_binary(&query::assets(deps)?),
        QueryMsg::Route { asset, amount } => to_binary(&query::route(deps, asset, amount)?),
    }
}
