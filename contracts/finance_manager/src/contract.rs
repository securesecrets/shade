use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    StdResult, StdError, Storage,
};

use shade_protocol::{
    manager,
    finance_manager::{
        Config, HandleMsg, InitMsg, QueryMsg
    },
};

use crate::{
    handle, query,
    state::{
        allocations_w, asset_list_w, config_w, self_address_w,
        viewing_key_w,
    },
};
use chrono::prelude::*;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    config_w(&mut deps.storage).save(&Config {
        admin: msg.admin.unwrap_or(env.message.sender.clone()),
        treasury: msg.treasury,
    })?;

    viewing_key_w(&mut deps.storage).save(&msg.viewing_key)?;
    self_address_w(&mut deps.storage).save(&env.contract.address)?;
    asset_list_w(&mut deps.storage).save(&Vec::new())?;

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
        /*
        HandleMsg::Receive {
            sender,
            from,
            amount,
            msg,
            ..
        } => handle::receive(deps, env, sender, from, amount, msg),
        */
        HandleMsg::UpdateConfig {
            config
        } => handle::try_update_config(deps, env, config),
        HandleMsg::RegisterAsset {
            contract
        } => handle::try_register_asset(deps, &env, &contract),
        HandleMsg::Allocate {
            asset,
            allocation
        } => handle::allocate(deps, &env, asset, allocation),

        HandleMsg::Manager(m) => match m {
            manager::HandleMsg::Unbond {
                asset,
                amount
            } => Err(StdError::generic_err("Not Implemented")),
            manager::HandleMsg::Claim {} => Err(StdError::generic_err("Not Implemented")),
            manager::HandleMsg::Rebalance {
                asset
            } => handle::rebalance(deps, &env, asset),
        }
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {

    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Assets {} => to_binary(&query::assets(deps)?),
        QueryMsg::Allocations {
            asset
        } => to_binary(&query::allocations(deps, asset)?),
        QueryMsg::PendingAllowance {
            asset
        } => to_binary(&query::pending_allowance(deps, asset)?),

        QueryMsg::Manager(m) => match m {
            manager::QueryMsg::Balance {
                asset
            } => to_binary(&query::outstanding_balance(deps, &asset)?),
            manager::QueryMsg::Unbonding { asset } => to_binary(&query::unbonding(deps, asset)?),
            manager::QueryMsg::Claimable { asset } => to_binary(&query::claimable(deps, asset)?),
        }
    }

}
