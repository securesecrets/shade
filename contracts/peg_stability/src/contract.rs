use crate::{handle, query};
use shade_protocol::{
    c_std::{
        entry_point,
        to_binary,
        Binary,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
        SubMsg,
    },
    contract_interfaces::{
        dao::adapter,
        peg_stability::{Config, ExecuteMsg, InstantiateMsg, QueryMsg, ViewingKey},
    },
    snip20::helpers::set_viewing_key_msg,
    utils::storage::plus::{GenericItemStorage, ItemStorage},
};

pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        shd_admin: msg.shd_admin.clone(),
        snip20: msg.snip20.clone(),
        pairs: vec![],
        oracle: msg.oracle.clone(),
        treasury: msg.treasury.clone(),
        symbols: vec![],
        payback: msg.payback,
        self_addr: env.contract.address.clone(),
    };
    config.save(deps.storage)?;
    ViewingKey::save(deps.storage, &msg.viewing_key.clone())?;
    Ok(
        Response::new().add_submessage(SubMsg::new(set_viewing_key_msg(
            msg.viewing_key.to_string(),
            None,
            &msg.snip20,
        )?)),
    )
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig {
            shd_admin,
            snip20,
            oracle,
            treasury,
            payback,
            ..
        } => handle::try_update_config(
            deps, env, info, shd_admin, snip20, oracle, treasury, payback,
        ),
        ExecuteMsg::SetPairs { pairs, symbol, .. } => {
            handle::try_set_pairs(deps, env, info, pairs, symbol)
        }
        ExecuteMsg::AppendPairs { pairs, symbol, .. } => {
            handle::try_append_pairs(deps, env, info, pairs, symbol)
        }
        ExecuteMsg::UpdatePair { pair, index, .. } => {
            handle::try_update_pair(deps, env, info, pair, index)
        }
        ExecuteMsg::RemovePair { index, .. } => handle::try_remove_pair(deps, env, info, index),
        ExecuteMsg::Swap { .. } => handle::try_swap(deps, env, info),
    }
}

#[entry_point]
pub fn query(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query::get_config(deps)?),
        QueryMsg::Balance {} => to_binary(&query::get_balance(deps)?),
        QueryMsg::GetPairs {} => to_binary(&query::get_pairs(deps)?),
        QueryMsg::Profitable {} => to_binary(&query::profitable(deps)?),
    }
}
