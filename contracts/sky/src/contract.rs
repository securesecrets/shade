use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    StdError, StdResult, Storage, self,
};
use secret_toolkit::snip20::set_viewing_key_msg;

use crate::{
    handle, query,
    state::{config_w, viewing_key_w, self_address_w},
};

use shade_protocol::contract_interfaces::sky::sky::{Config, InitMsg, HandleMsg, QueryMsg};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = Config {
        admin: match msg.admin{
            None => env.message.sender.clone(),
            Some(admin) => admin,
        },
        mint_addr: msg.mint_addr,
        market_swap_addr: msg.market_swap_addr,
        shd_token: msg.shd_token.clone(),
        silk_token: msg.silk_token.clone(),
        treasury: msg.treasury,
        limit: msg.limit,
    };

    config_w(&mut deps.storage).save(&state)?;
    self_address_w(&mut deps.storage).save(&env.contract.address)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

    let mut messages = vec![
        set_viewing_key_msg(
            msg.viewing_key.clone(), 
            None, 
            1, 
            msg.shd_token.contract.code_hash.clone(), 
            msg.shd_token.contract.address.clone(),    
        ).unwrap(),
        set_viewing_key_msg(
            msg.viewing_key.clone(), 
            None, 
            1, 
            msg.silk_token.contract.code_hash.clone(), 
            msg.silk_token.contract.address.clone()
        ).unwrap()
    ];

    viewing_key_w(&mut deps.storage).save(&msg.viewing_key)?;

    Ok(InitResponse{
        messages,
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::UpdateConfig{ config } => handle::try_update_config(deps, env, config),
        HandleMsg::ArbPeg{ amount } => handle::try_execute(deps, env, amount),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query::config(deps)?),
        QueryMsg::GetMarketRate {} => to_binary(&query::market_rate(deps)?),
        QueryMsg::IsProfitable { amount } => to_binary( &query::trade_profitability(deps, amount)?),
        QueryMsg::Balance{} => to_binary(&query::get_balances(deps)?)
    }
}
