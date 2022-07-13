use shade_protocol::c_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, Response, InitResponse, Querier,
    StdError, StdResult, Storage, self,
};
use shade_protocol::secret_toolkit::snip20::set_viewing_key_msg;

use crate::{
    handle, query,
};

use shade_protocol::{
    contract_interfaces::sky::sky::{Config, InitMsg, HandleMsg, QueryMsg, ViewingKeys, SelfAddr},
    utils::storage::plus::ItemStorage,
};

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

    state.save(&mut deps.storage)?;
    SelfAddr(env.contract.address).save(&mut deps.storage)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

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

    ViewingKeys(msg.viewing_key).save(&mut deps.storage)?;

    Ok(InitResponse{
        messages,
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<Response> {
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
