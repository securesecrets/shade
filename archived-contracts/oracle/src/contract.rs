use crate::{handle, query, state::config_w};
use shade_protocol::c_std::{

    to_binary,
    Api,
    Binary,
    Env,
    DepsMut,
    Response,
    Querier,
    StdResult,
    Storage,
};
use shade_protocol::contract_interfaces::oracles::oracle::{
    ExecuteMsg,
    InstantiateMsg,
    OracleConfig,
    QueryMsg,
};

pub fn init(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = OracleConfig {
        admin: match msg.admin {
            None => info.sender.clone(),
            Some(admin) => admin,
        },
        band: msg.band,
        sscrt: msg.sscrt,
    };

    config_w(deps.storage).save(&state)?;

    deps.api.debug("Contract was initialized by {}", info.sender);

    Ok(Response::default())
}

pub fn handle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig { admin, band } => {
            handle::try_update_config(deps, env, info, admin, band)
        }
        ExecuteMsg::RegisterPair { pair } => handle::register_pair(deps, env, info, pair),
        ExecuteMsg::UnregisterPair { symbol, pair } => {
            handle::unregister_pair(deps, env, info, symbol, pair)
        }
        ExecuteMsg::RegisterIndex { symbol, basket } => {
            handle::register_index(deps, env, info, symbol, basket)
        }
    }
}

pub fn query(
    deps: Deps,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Price { symbol } => to_binary(&query::price(deps, symbol)?),
        QueryMsg::Prices { symbols } => to_binary(&query::prices(deps, symbols)?),
    }
}
