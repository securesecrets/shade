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
    dao::{
        adapter,
        stkd_scrt::{Config, ExecuteMsg, InstantiateMsg, QueryMsg},
    },
    snip20::helpers::{register_receive, set_viewing_key_msg},
    utils::generic_response::ResponseStatus,
};

use crate::{execute, query, storage::*};

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        admin_auth: msg.admin_auth.into_valid(deps.api)?,
        sscrt: msg.sscrt.into_valid(deps.api)?,
        owner: deps.api.addr_validate(msg.owner.as_str())?,
        staking_derivatives: msg.staking_derivatives.into_valid(deps.api)?,
    };

    CONFIG.save(deps.storage, &config)?;

    SELF_ADDRESS.save(deps.storage, &env.contract.address)?;
    VIEWING_KEY.save(deps.storage, &msg.viewing_key)?;

    Ok(Response::new().add_messages(vec![
        set_viewing_key_msg(msg.viewing_key, None, &config.sscrt)?,
        register_receive(env.contract.code_hash, None, &config.sscrt)?,
    ]))
}

#[shd_entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive {
            sender,
            from,
            amount,
            msg,
            ..
        } => {
            let sender = deps.api.addr_validate(&sender)?;
            let from = deps.api.addr_validate(&from)?;
            execute::receive(deps, env, info, sender, from, amount, msg)
        }
        ExecuteMsg::UpdateConfig { config } => execute::try_update_config(deps, env, info, config),
        ExecuteMsg::Adapter(adapter) => match adapter {
            adapter::SubExecuteMsg::Unbond { asset, amount } => {
                let asset = deps.api.addr_validate(&asset)?;
                execute::unbond(deps, env, info, asset, amount)
            }
            adapter::SubExecuteMsg::Claim { asset } => {
                let asset = deps.api.addr_validate(&asset)?;
                execute::claim(deps, env, info, asset)
            }
            adapter::SubExecuteMsg::Update { asset: _ } => Ok(Response::new().set_data(to_binary(
                &adapter::ExecuteAnswer::Update {
                    status: ResponseStatus::Success,
                },
            )?)),
        },
    }
}

#[shd_entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Adapter(adapter) => match adapter {
            adapter::SubQueryMsg::Balance { asset } => {
                let asset = deps.api.addr_validate(&asset)?;
                to_binary(&query::balance(deps, env, asset)?)
            }
            adapter::SubQueryMsg::Claimable { asset } => {
                let asset = deps.api.addr_validate(&asset)?;
                to_binary(&query::claimable(deps, env, asset)?)
            }
            adapter::SubQueryMsg::Unbonding { asset } => {
                let asset = deps.api.addr_validate(&asset)?;
                to_binary(&query::unbonding(deps, env, asset)?)
            }
            adapter::SubQueryMsg::Unbondable { asset } => {
                let asset = deps.api.addr_validate(&asset)?;
                to_binary(&query::unbondable(deps, env, asset)?)
            }
            adapter::SubQueryMsg::Reserves { asset } => {
                let asset = deps.api.addr_validate(&asset)?;
                to_binary(&query::reserves(deps, env, asset)?)
            }
        },
    }
}
