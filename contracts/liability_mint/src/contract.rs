use shade_protocol::c_std::{
    entry_point, to_binary, Api, Binary, Deps, DepsMut, Env, MessageInfo, Querier, Response,
    StdResult, Storage, Uint128,
};
use shade_protocol::snip20::helpers::{fetch_snip20, register_receive, token_config, token_info};

use shade_protocol::contract_interfaces::{
    mint::liability_mint::{Config, ExecuteMsg, InstantiateMsg, QueryMsg},
    snip20::helpers::Snip20Asset,
};

use crate::{execute, query, storage::*};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let config = Config {
        admin: match msg.admin {
            None => info.sender.clone(),
            Some(admin) => admin,
        },
        token: msg.token,
        debt_ratio: msg.debt_ratio,
        oracle: msg.oracle,
        treasury: msg.treasury,
    };

    CONFIG.save(deps.storage, &config)?;
    TOKEN.save(deps.storage, &fetch_snip20(&msg.token, &deps.querier)?)?;
    LIABILITIES.save(deps.storage, &Uint128::zero())?;
    WHITELIST.save(deps.storage, &Vec::new())?;
    COLLATERAL.save(deps.storage, &Vec::new())?;

    deps.api
        .debug(&format!("Contract was initialized by {}", info.sender));

    Ok(Response::new().add_message(register_receive(env.contract.code_hash, None, &msg.token)?))
}

#[entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig { config } => execute::try_update_config(deps, env, info, config),
        ExecuteMsg::Receive {
            sender,
            from,
            amount,
            msg,
            ..
        } => execute::receive(deps, env, info, sender, from, amount, msg),
        ExecuteMsg::AddWhitelist { address } => execute::add_whitelist(deps, env, info, address),
        ExecuteMsg::RemoveWhitelist { address } => execute::rm_whitelist(deps, env, info, address),
        ExecuteMsg::AddCollateral { asset } => execute::add_collateral(deps, env, info, asset),
        ExecuteMsg::RemoveCollateral { asset } => execute::rm_collateral(deps, env, info, asset),
        ExecuteMsg::Mint { amount } => execute::mint(deps, env, info, amount),
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Token {} => to_binary(&query::token(deps)?),
        QueryMsg::Liabilities {} => to_binary(&query::liabilities(deps)?),
        QueryMsg::Whitelist {} => to_binary(&query::whitelist(deps)?),
    }
}
