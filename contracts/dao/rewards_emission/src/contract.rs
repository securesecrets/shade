use shade_protocol::c_std::{
    shd_entry_point,
    to_binary,
    Addr,
    Binary,
    Deps,
    DepsMut,
    Env,
    MessageInfo,
    Response,
    StdResult,
};

use shade_protocol::contract_interfaces::dao::rewards_emission::{
    Config,
    ExecuteMsg,
    InstantiateMsg,
    QueryMsg,
};

use shade_protocol::snip20::helpers::fetch_snip20;
//use shade_protocol::contract_interfaces::dao::adapter;

use crate::{execute, query, storage::*};

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let mut admins: Vec<Addr> = msg.admins
        .iter()
        //TODO change unwrap
        .map(|a| deps.api.addr_validate(&a).ok().unwrap())
        .collect();

    if !admins.contains(&info.sender) {
        admins.push(info.sender);
    }

    let config = Config {
        admins,
        treasury: deps.api.addr_validate(&msg.treasury)?,
    };

    CONFIG.save(deps.storage, &config)?;
    SELF_ADDRESS.save(deps.storage, &env.contract.address)?;
    VIEWING_KEY.save(deps.storage, &msg.viewing_key)?;
    TOKEN.save(
        deps.storage,
        &fetch_snip20(&msg.token.into_valid(deps.api)?, &deps.querier)?,
    )?;

    Ok(Response::new())
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
        } => execute::receive(deps, env, info, sender, from, amount, msg),
        ExecuteMsg::UpdateConfig { config } => execute::try_update_config(deps, env, info, config),
        ExecuteMsg::RegisterRewards {
            token,
            distributor,
            amount,
            cycle,
            expiration,
        } => execute::register_rewards(
            deps,
            env,
            info,
            token,
            distributor,
            amount,
            cycle,
            expiration,
        ),
        ExecuteMsg::RefillRewards {} => execute::refill_rewards(deps, env, info),
    }
}

#[shd_entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        //QueryMsg::PendingAllowance { asset } => to_binary(&query::pending_allowance(deps, asset)?),
    }
}
