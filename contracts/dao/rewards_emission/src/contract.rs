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

use shade_protocol::contract_interfaces::dao::rewards_emission::{
    Config,
    ExecuteMsg,
    InstantiateMsg,
    QueryMsg,
};

use shade_protocol::snip20::helpers::{register_receive, set_viewing_key_msg};
use shade_protocol::contract_interfaces::dao::adapter;

use crate::{
    handle,
    query,
    state::{config_w, self_address_w, viewing_key_r, viewing_key_w},
};

pub fn init(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let mut config = msg.config;

    if !config.admins.contains(&info.sender) {
        config.admins.push(info.sender);
    }

    config_w(deps.storage).save(&config)?;

    self_address_w(deps.storage).save(&env.contract.address)?;
    viewing_key_w(deps.storage).save(&msg.viewing_key)?;

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
        ExecuteMsg::RegisterAsset { asset } => handle::register_asset(deps, env, info, &asset),
        ExecuteMsg::RefillRewards { rewards } => handle::refill_rewards(deps, env, info, rewards),

        ExecuteMsg::Adapter(adapter) => match adapter {
            // Maybe should return an Ok still?
            adapter::SubHandleMsg::Unbond { asset, amount } => {
                Err(StdError::generic_err("Cannot unbond from rewards"))
            }
            // If error on unbond, also error on claim
            adapter::SubHandleMsg::Claim { asset } => handle::claim(deps, env, info, asset),
            adapter::SubHandleMsg::Update { asset } => handle::update(deps, env, info, asset),
        },
    }
}

pub fn query(
    deps: Deps,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::PendingAllowance { asset } => to_binary(&query::pending_allowance(deps, asset)?),
        QueryMsg::Adapter(adapter) => match adapter {
            adapter::SubQueryMsg::Balance { asset } => to_binary(&query::balance(deps, asset)?),
            // Unbonding disabled
            adapter::SubQueryMsg::Claimable { asset } => {
                to_binary(&adapter::QueryAnswer::Claimable {
                    amount: Uint128::zero(),
                })
            }
            adapter::SubQueryMsg::Unbonding { asset } => {
                to_binary(&adapter::QueryAnswer::Unbonding {
                    amount: Uint128::zero(),
                })
            }
            adapter::SubQueryMsg::Unbondable { asset } => {
                to_binary(&adapter::QueryAnswer::Unbondable {
                    amount: Uint128::zero(),
                })
            }
            adapter::SubQueryMsg::Reserves { asset } => {
                to_binary(&adapter::QueryAnswer::Reserves {
                    amount: Uint128::zero(),
                })
            }
        },
    }
}
