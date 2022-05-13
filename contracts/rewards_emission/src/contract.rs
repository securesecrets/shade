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
    Uint128,
};

use shade_protocol::contract_interfaces::dao::rewards_emission::{
    Config,
    HandleMsg,
    InitMsg,
    QueryMsg,
};

use secret_toolkit::snip20::{register_receive_msg, set_viewing_key_msg};
use shade_protocol::contract_interfaces::dao::adapter;

use crate::{
    handle,
    query,
    state::{config_w, self_address_w, viewing_key_r, viewing_key_w},
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let mut config = msg.config;

    if !config.admins.contains(&env.message.sender) {
        config.admins.push(env.message.sender);
    }

    config_w(&mut deps.storage).save(&config)?;

    self_address_w(&mut deps.storage).save(&env.contract.address)?;
    viewing_key_w(&mut deps.storage).save(&msg.viewing_key)?;

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
        HandleMsg::RegisterAsset { asset } => handle::register_asset(deps, env, &asset),
        HandleMsg::RefillRewards { rewards } => handle::refill_rewards(deps, env, rewards),

        HandleMsg::Adapter(adapter) => match adapter {
            // Maybe should return an Ok still?
            adapter::SubHandleMsg::Unbond { asset, amount } => {
                Err(StdError::generic_err("Cannot unbond from rewards"))
            }
            // If error on unbond, also error on claim
            adapter::SubHandleMsg::Claim { asset } => handle::claim(deps, env, asset),
            adapter::SubHandleMsg::Update { asset } => handle::update(deps, env, asset),
        },
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
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
        },
    }
}
