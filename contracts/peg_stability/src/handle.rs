use shade_protocol::{
    admin::validate_admin,
    c_std::{Decimal, DepsMut, Env, MessageInfo, Response, StdResult, Uint128},
    contract_interfaces::{
        peg_stability::{Config, ExecuteAnswer},
        sky::cycles::ArbPair,
    },
    utils::{asset::Contract, storage::plus::ItemStorage},
};
pub fn try_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    shd_admin: Option<Contract>,
    snip20: Option<Contract>,
    treasury: Option<Contract>,
    oracle: Option<Contract>,
    payback: Option<Decimal>,
) -> StdResult<Response> {
    //Admin-only
    let mut config = Config::load(deps.storage)?;
    validate_admin(
        &deps.querier,
        env.contract.address.to_string(),
        info.sender.to_string(),
        &config.shd_admin,
    )?;
    Ok(Response::new())
}

pub fn try_set_pairs(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pairs: Vec<ArbPair>,
    symbol: Option<String>,
) -> StdResult<Response> {
    Ok(Response::new())
}

pub fn try_append_pairs(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pairs: Vec<ArbPair>,
    symbol: Option<String>,
) -> StdResult<Response> {
    Ok(Response::new())
}

pub fn try_update_pair(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pair: ArbPair,
    index: Uint128,
) -> StdResult<Response> {
    Ok(Response::new())
}

pub fn try_remove_pair(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    index: Uint128,
) -> StdResult<Response> {
    Ok(Response::new())
}

pub fn try_swap(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    Ok(Response::new())
}
