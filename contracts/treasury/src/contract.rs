use shade_protocol::{
    c_std::{
        to_binary, Api, Binary,
        Env, DepsMut, Response,
        Querier, StdError, StdResult,
        Storage, Uint128, entry_point,
        MessageInfo,
        Deps,
    },
    contract_interfaces::dao::{
        treasury::{
            Config, ExecuteMsg, InstantiateMsg, QueryMsg,
            storage::*,
        },
        adapter,
    },
};

use crate::{
    handle,
    query,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    CONFIG.save(deps.storage, &Config {
        admin: msg.admin.unwrap_or(info.sender.clone()),
    })?;

    VIEWING_KEY.save(deps.storage, &msg.viewing_key)?;
    SELF_ADDRESS.save(deps.storage, &env.contract.address)?;
    ASSET_LIST.save(deps.storage, &Vec::new())?;
    MANAGERS.save(deps.storage, &Vec::new())?;

    //deps.api.debug("Contract was initialized by {}", info.sender);

    Ok(Response::new())
}

#[entry_point]
pub fn execute(
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
        ExecuteMsg::RegisterAsset { contract } => {
            handle::try_register_asset(deps, &env, info, &contract)
        }
        ExecuteMsg::RegisterManager { mut contract } => {
            handle::register_manager(deps, &env, info, &mut contract)
        }
        ExecuteMsg::Allowance { asset, allowance } => {
            handle::allowance(deps, &env, info, asset, allowance)
        }
        ExecuteMsg::Adapter(adapter) => match adapter {
            adapter::SubExecuteMsg::Update { asset } => handle::rebalance(deps, &env, asset),
            adapter::SubExecuteMsg::Claim { asset } => handle::claim(deps, &env, info, asset),
            adapter::SubExecuteMsg::Unbond { asset, amount } => {
                handle::unbond(deps, &env, info, asset, amount)
            }
        },
    }
}

#[entry_point]
pub fn query(
    deps: Deps,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Assets {} => to_binary(&query::assets(deps)?),
        QueryMsg::Allowances { asset } => to_binary(&query::allowances(deps, asset)?),
        QueryMsg::Allowance { asset, spender } => to_binary(&query::allowance(deps, asset, spender)?),

        QueryMsg::Adapter(adapter) => match adapter {
            adapter::SubQueryMsg::Balance { asset } => to_binary(&query::balance(deps, asset)?),
            adapter::SubQueryMsg::Unbonding { asset } => to_binary(&query::unbonding(deps, asset)?),
            adapter::SubQueryMsg::Unbondable { asset } => to_binary(&query::unbondable(deps, asset)?),
            adapter::SubQueryMsg::Claimable { asset } => to_binary(&query::claimable(deps, asset)?),
            adapter::SubQueryMsg::Reserves { asset } => to_binary(&query::reserves(deps, asset)?),
        }
    }
}
