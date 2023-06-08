use crate::{handle, query};
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
    contract_interfaces::{
        peg_stability::{Config, ExecuteAnswer, ExecuteMsg, InstantiateMsg, QueryMsg, ViewingKey},
    },
    snip20::helpers::set_viewing_key_msg,
    utils::{
        generic_response::ResponseStatus,
        storage::plus::{GenericItemStorage, ItemStorage},
    },
};

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        admin_auth: msg.admin_auth.clone(),
        snip20: msg.snip20.clone(),
        pairs: vec![],
        oracle: msg.oracle.clone(),
        treasury: msg.treasury.clone(),
        symbols: vec![],
        payback: msg.payback,
        self_addr: env.contract.address.clone(),
        dump_contract: msg.dump_contract,
    };
    config.save(deps.storage)?;
    ViewingKey::save(deps.storage, &msg.viewing_key.clone())?;
    Ok(Response::new()
        .add_message(set_viewing_key_msg(
            msg.viewing_key.to_string(),
            None,
            &msg.snip20,
        )?)
        .set_data(to_binary(&ExecuteAnswer::Init {
            status: ResponseStatus::Success,
        })?))
}

#[shd_entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig {
            admin_auth,
            snip20,
            oracle,
            treasury,
            payback,
            dump_contract,
            ..
        } => handle::try_update_config(
            deps,
            env,
            info,
            admin_auth,
            snip20,
            oracle,
            treasury,
            payback,
            dump_contract,
        ),
        ExecuteMsg::SetPairs { pairs, .. } => handle::try_set_pairs(deps, env, info, pairs),
        ExecuteMsg::AppendPairs { pairs, .. } => handle::try_append_pairs(deps, env, info, pairs),
        ExecuteMsg::RemovePair { pair_address, .. } => {
            handle::try_remove_pair(deps, env, info, pair_address)
        }
        ExecuteMsg::Swap { .. } => handle::try_swap(deps, env, info),
    }
}

#[shd_entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query::get_config(deps)?),
        QueryMsg::Balance {} => to_binary(&query::get_balance(deps)?),
        QueryMsg::GetPairs {} => to_binary(&query::get_pairs(deps)?),
        QueryMsg::Profitable {} => to_binary(&query::profitable(deps)?),
    }
}
