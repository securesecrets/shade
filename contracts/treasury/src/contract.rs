use cosmwasm_std::{
    debug_print,
    to_binary,
    Api,
    Binary,
    Env,
    Extern,
    HandleResponse,
    InitResponse,
    Querier,
    StdError,
    StdResult,
    Storage,
};

use shade_protocol::contract_interfaces::dao::treasury::{Config, HandleMsg, InitMsg, QueryMsg};

use crate::{
    handle,
    query,
};

use shade_protocol::contract_interfaces::dao::{
    adapter,
    treasury::storage::*,
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    CONFIG.save(&mut deps.storage, &Config {
        admin: msg.admin.unwrap_or(env.message.sender.clone()),
    })?;

    VIEWING_KEY.save(&mut deps.storage, &msg.viewing_key)?;
    SELF_ADDRESS.save(&mut deps.storage, &env.contract.address)?;
    ASSET_LIST.save(&mut deps.storage, &Vec::new())?;
    MANAGERS.save(&mut deps.storage, &Vec::new())?;

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
        HandleMsg::RegisterAsset { contract } => {
            handle::try_register_asset(deps, &env, &contract)
        }
        HandleMsg::RegisterManager { mut contract } => {
            handle::register_manager(deps, &env, &mut contract)
        }
        HandleMsg::Allowance { asset, allowance } => {
            handle::allowance(deps, &env, asset, allowance)
        }
        HandleMsg::Adapter(adapter) => match adapter {
            adapter::SubHandleMsg::Update { asset } => handle::rebalance(deps, &env, asset),
            adapter::SubHandleMsg::Claim { asset } => handle::claim(deps, &env, asset),
            adapter::SubHandleMsg::Unbond { asset, amount } => {
                handle::unbond(deps, &env, asset, amount)
            }
        },
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
        QueryMsg::Allowance { asset, spender } => to_binary(&query::allowance(&deps, asset, spender)?),

        QueryMsg::Adapter(adapter) => match adapter {
            adapter::SubQueryMsg::Balance { asset } => to_binary(&query::balance(&deps, asset)?),
            adapter::SubQueryMsg::Unbonding { asset } => to_binary(&query::unbonding(&deps, asset)?),
            adapter::SubQueryMsg::Unbondable { asset } => to_binary(&query::unbondable(&deps, asset)?),
            adapter::SubQueryMsg::Claimable { asset } => to_binary(&query::claimable(&deps, asset)?),
            adapter::SubQueryMsg::Reserves { asset } => to_binary(&query::reserves(&deps, asset)?),
        }
    }
}
