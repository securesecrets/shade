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

use shade_protocol::contract_interfaces::dao::scrt_staking::{
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
    state::{config_w, self_address_w, unbonding_w, viewing_key_r, viewing_key_w},
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let config = Config {
        admins: match msg.admins {
            None => vec![env.message.sender.clone()],
            Some(mut admins) => {
                if !admins.contains(&env.message.sender) {
                    admins.push(env.message.sender);
                }
                admins
            }
        },
        sscrt: msg.sscrt,
        owner: msg.owner,
        validator_bounds: msg.validator_bounds,
    };

    config_w(&mut deps.storage).save(&config)?;

    self_address_w(&mut deps.storage).save(&env.contract.address)?;
    viewing_key_w(&mut deps.storage).save(&msg.viewing_key)?;
    unbonding_w(&mut deps.storage).save(&Uint128::zero())?;

    Ok(InitResponse {
        messages: vec![
            set_viewing_key_msg(
                viewing_key_r(&deps.storage).load()?,
                None,
                1,
                config.sscrt.code_hash.clone(),
                config.sscrt.address.clone(),
            )?,
            register_receive_msg(
                env.contract_code_hash,
                None,
                256,
                config.sscrt.code_hash,
                config.sscrt.address,
            )?,
        ],
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
        HandleMsg::Adapter(adapter) => match adapter {
            adapter::SubHandleMsg::Unbond { asset, amount } => {
                handle::unbond(deps, env, asset, amount)
            }
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
