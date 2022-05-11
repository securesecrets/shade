use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    StdResult, Storage, StdError, Uint128,
};

use shade_protocol::{
    adapter,
    treasury::{Config, HandleMsg, InitMsg, QueryMsg},
};

use crate::{
    handle, query,
    state::{
        allowances_w, asset_list_w, config_w, self_address_w,
        viewing_key_w, managers_w, total_unbonding_w,
        account_list_w,
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
        sscrt: msg.sscrt,
    })?;

    viewing_key_w(&mut deps.storage).save(&msg.viewing_key)?;
    self_address_w(&mut deps.storage).save(&env.contract.address)?;
    asset_list_w(&mut deps.storage).save(&Vec::new())?;
    managers_w(&mut deps.storage).save(&Vec::new())?;
    account_list_w(&mut deps.storage).save(&Vec::new())?;

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
        HandleMsg::RegisterAsset { contract, reserves } => handle::try_register_asset(deps, &env, &contract, reserves),
        HandleMsg::RegisterManager { mut contract } => handle::register_manager(deps, &env, &mut contract ),
        HandleMsg::Allowance { asset, allowance } => handle::allowance(deps, &env, asset, allowance),
        HandleMsg::AddAccount { holder } => handle::add_account(deps, &env, holder),
        HandleMsg::CloseAccount { holder } => handle::close_account(deps, &env, holder),
        HandleMsg::Adapter(adapter) => match adapter {
            adapter::SubHandleMsg::Update { asset } => handle::rebalance(deps, &env, asset),
            adapter::SubHandleMsg::Claim { asset } => handle::claim(deps, &env, asset),
            adapter::SubHandleMsg::Unbond { asset, amount } => handle::unbond(deps, &env, asset, amount),
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
        QueryMsg::Allowances { asset } => to_binary(&query::allowances(deps, asset)?),
        QueryMsg::Allowance { asset, spender } => to_binary(&query::allowance(&deps, &asset, &spender)?),
        QueryMsg::Accounts { } => to_binary(&query::accounts(&deps)?),
        QueryMsg::Account { holder } => to_binary(&query::account(&deps, holder)?),

        QueryMsg::Adapter(adapter) => match adapter {
            adapter::SubQueryMsg::Balance { asset } => to_binary(&query::balance(&deps, &asset)?),
            adapter::SubQueryMsg::Unbonding { asset } => to_binary(&query::unbonding(&deps, &asset)?),
            adapter::SubQueryMsg::Unbondable { asset } => to_binary(&StdError::generic_err("Not Implemented")),
            adapter::SubQueryMsg::Claimable { asset } => to_binary(&query::claimable(&deps, &asset)?),
        }
    }
}
