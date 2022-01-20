use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    StdResult, Storage,
};

use shade_protocol::treasury::{Config, HandleMsg, InitMsg, QueryMsg};

use crate::{
    handle, query,
    state::{config_w, self_address_w, viewing_key_w},
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    config_w(&mut deps.storage).save(&Config {
        admin: msg.admin.unwrap_or(env.message.sender.clone()),
    })?;

    viewing_key_w(&mut deps.storage).save(&msg.viewing_key)?;
    self_address_w(&mut deps.storage).save(&env.contract.address)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse {
        messages: vec![],
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Receive {
            sender,
            from,
            amount,
            msg,
            ..
        } => handle::receive(deps, env, sender, from, amount, msg),
        HandleMsg::UpdateConfig { config } => handle::try_update_config(deps, env, config),
        HandleMsg::RegisterAsset { contract, reserves } => {
            handle::try_register_asset(deps, &env, &contract, reserves)
        }
        HandleMsg::RegisterAllocation { asset, allocation } => {
            handle::register_allocation(deps, &env, asset, allocation)
        } /*
          HandleMsg::Rebalance {
          } => handle::rebalance(deps, &env),
          */
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Allocations { asset } => to_binary(&query::allocations(deps, asset)?),
        QueryMsg::Balance { asset } => to_binary(&query::balance(&deps, &asset)?),
    }
}
