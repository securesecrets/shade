use shade_protocol::c_std::{
    debug_print, to_binary, Api, Binary, Env, DepsMut, Response, Querier,
    StdError, StdResult, Storage, self,
};
use shade_protocol::snip20::helpers::set_viewing_key_msg;

use crate::{
    handle, query,
};

use shade_protocol::{
    contract_interfaces::sky::sky::{Config, InstantiateMsg, ExecuteMsg, QueryMsg, ViewingKeys, SelfAddr},
    utils::storage::plus::ItemStorage,
};

pub fn init(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = Config {
        admin: match msg.admin{
            None => info.sender.clone(),
            Some(admin) => admin,
        },
        mint_addr: msg.mint_addr,
        market_swap_addr: msg.market_swap_addr,
        shd_token: msg.shd_token.clone(),
        silk_token: msg.silk_token.clone(),
        treasury: msg.treasury,
        limit: msg.limit,
    };

    state.save(deps.storage)?;
    SelfAddr(env.contract.address).save(deps.storage)?;

    debug_print!("Contract was initialized by {}", info.sender);

    let mut messages = vec![
        set_viewing_key_msg(
            msg.viewing_key.clone(), 
            None, 
            1, 
            msg.shd_token.contract.code_hash.clone(), 
            msg.shd_token.contract.address.clone(),    
        )?,
        set_viewing_key_msg(
            msg.viewing_key.clone(), 
            None, 
            1, 
            msg.silk_token.contract.code_hash.clone(), 
            msg.silk_token.contract.address.clone()
        )?
    ];

    ViewingKeys(msg.viewing_key).save(deps.storage)?;

    Ok(Response{
        messages,
        log: vec![],
    })
}

pub fn handle(
    deps: DepsMut,
    env: Env,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig{ config } => handle::try_update_config(deps, env, config),
        ExecuteMsg::ArbPeg{ amount } => handle::try_execute(deps, env, amount),
    }
}

pub fn query(
    deps: Deps,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query::config(deps)?),
        QueryMsg::GetMarketRate {} => to_binary(&query::market_rate(deps)?),
        QueryMsg::IsProfitable { amount } => to_binary( &query::trade_profitability(deps, amount)?),
        QueryMsg::Balance{} => to_binary(&query::get_balances(deps)?)
    }
}
