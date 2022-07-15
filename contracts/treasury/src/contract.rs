use shade_protocol::c_std::{

    to_binary,
    Api,
    Binary,
    Env,
    DepsMut,
    Response,
    Querier,
    StdError,
    StdResult,
    Storage,
    Uint128,
};

use shade_protocol::contract_interfaces::dao::treasury::{Config, ExecuteMsg, InstantiateMsg, QueryMsg};

use crate::{
    handle,
    query,
    state::{
        allowances_w,
        asset_list_w,
        config_w,
        managers_w,
        self_address_w,
        viewing_key_w,
    },
};
use chrono::prelude::*;
use shade_protocol::contract_interfaces::dao::adapter;

pub fn init(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    config_w(deps.storage).save(&Config {
        admin: msg.admin.unwrap_or(info.sender.clone()),
    })?;

    viewing_key_w(deps.storage).save(&msg.viewing_key)?;
    self_address_w(deps.storage).save(&env.contract.address)?;
    asset_list_w(deps.storage).save(&Vec::new())?;
    managers_w(deps.storage).save(&Vec::new())?;
    //account_list_w(deps.storage).save(&Vec::new())?;

    deps.api.debug("Contract was initialized by {}", info.sender);

    Ok(Response::new())
}

pub fn handle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive {
            sender,
            from,
            amount,
            msg,
            ..
        } => handle::receive(deps, env, info, sender, from, amount, msg),
        ExecuteMsg::UpdateConfig { config } => handle::try_update_config(deps, env, info, config),
        ExecuteMsg::RegisterAsset { contract, reserves } => {
            handle::try_register_asset(deps, &env, &contract, reserves)
        }
        ExecuteMsg::RegisterManager { mut contract } => {
            handle::register_manager(deps, &env, &mut contract)
        }
        ExecuteMsg::Allowance { asset, allowance } => {
            handle::allowance(deps, &env, asset, allowance)
        }
        ExecuteMsg::Adapter(adapter) => match adapter {
            adapter::SubHandleMsg::Update { asset } => handle::rebalance(deps, &env, asset),
            adapter::SubHandleMsg::Claim { asset } => handle::claim(deps, &env, asset),
            adapter::SubHandleMsg::Unbond { asset, amount } => {
                handle::unbond(deps, &env, asset, amount)
            }
        },
    }
}

pub fn query(
    deps: Deps,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Assets {} => to_binary(&query::assets(deps)?),
        QueryMsg::Allowances { asset } => to_binary(&query::allowances(deps, asset)?),
        QueryMsg::Allowance { asset, spender } => to_binary(&query::allowance(&deps, &asset, &spender)?),

        QueryMsg::Adapter(adapter) => match adapter {
            adapter::SubQueryMsg::Balance { asset } => to_binary(&query::balance(&deps, &asset)?),
            adapter::SubQueryMsg::Unbonding { asset } => to_binary(&query::unbonding(&deps, &asset)?),
            adapter::SubQueryMsg::Unbondable { asset } => to_binary(&StdError::generic_err("Not Implemented")),
            adapter::SubQueryMsg::Claimable { asset } => to_binary(&query::claimable(&deps, &asset)?),
            adapter::SubQueryMsg::Reserves { asset } => to_binary(&query::reserves(&deps, &asset)?),
        }
    }
}
