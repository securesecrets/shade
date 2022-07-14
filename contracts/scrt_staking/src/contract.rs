use shade_protocol::c_std::{
    debug_print,
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

use shade_protocol::contract_interfaces::dao::scrt_staking::{
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
    state::{config_w, self_address_w, unbonding_w, viewing_key_r, viewing_key_w},
};

pub fn init(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        admins: match msg.admins {
            None => vec![info.sender.clone()],
            Some(mut admins) => {
                if !admins.contains(&info.sender) {
                    admins.push(info.sender);
                }
                admins
            }
        },
        sscrt: msg.sscrt,
        owner: msg.owner,
        validator_bounds: msg.validator_bounds,
    };

    config_w(deps.storage).save(&config)?;

    self_address_w(deps.storage).save(&env.contract.address)?;
    viewing_key_w(deps.storage).save(&msg.viewing_key)?;
    unbonding_w(deps.storage).save(&Uint128::zero())?;

    Ok(Response {
        messages: vec![
            set_viewing_key_msg(
                viewing_key_r(deps.storage).load()?,
                None,
                1,
                config.sscrt.code_hash.clone(),
                config.sscrt.address.clone(),
            )?,
            register_receive(
                env.contract.code_hash,
                None,
                &config.sscrt
            )?,
        ],
        log: vec![],
    })
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
        ExecuteMsg::Adapter(adapter) => match adapter {
            adapter::SubHandleMsg::Unbond { asset, amount } => {
                handle::unbond(deps, env, info, asset, amount)
            }
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
        QueryMsg::Delegations {} => to_binary(&query::delegations(deps)?),
        QueryMsg::Adapter(adapter) => match adapter {
            adapter::SubQueryMsg::Balance { asset } => to_binary(&query::balance(deps, asset)?),
            adapter::SubQueryMsg::Claimable { asset } => to_binary(&query::claimable(deps, asset)?),
            adapter::SubQueryMsg::Unbonding { asset } => to_binary(&query::unbonding(deps, asset)?),
            adapter::SubQueryMsg::Unbondable { asset } => to_binary(&query::unbondable(deps, asset)?),
            adapter::SubQueryMsg::Reserves { asset } => to_binary(&query::reserves(deps, asset)?),
        }
    }
}
