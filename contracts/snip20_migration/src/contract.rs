use shade_protocol::{
    c_std::{
        shd_entry_point,
        to_binary,
        Binary,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdResult,
    },
    contract_interfaces::snip20_migration::{Config, ExecuteMsg, InstantiateMsg, QueryMsg},
    snip20::helpers::register_receive,
    utils::{asset::Contract, pad_handle_result, pad_query_result, storage::plus::ItemStorage},
};

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::default())
}

#[shd_entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::SetConfig {} => Ok(Response::default()),
        ExecuteMsg::Recieve {} => Ok(Response::default()),
    }
}

#[shd_entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&{}),
    }
}
