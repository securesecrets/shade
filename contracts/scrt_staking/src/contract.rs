use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    StdResult, StdError,
    Storage,
};

use shade_protocol::{
    adapter,
    scrt_staking::{Config, HandleMsg, InitMsg, QueryMsg},
};

use secret_toolkit::snip20::{register_receive_msg, set_viewing_key_msg};

use crate::{
    handle, query,
    state::{config_w, self_address_w, viewing_key_r, viewing_key_w},
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let config = Config {
        admin: match msg.admin {
            None => env.message.sender.clone(),
            Some(admin) => admin,
        },
        sscrt: msg.sscrt,
        treasury: msg.treasury,
        validator_bounds: msg.validator_bounds,
    };

    config_w(&mut deps.storage).save(&config)?;

    self_address_w(&mut deps.storage).save(&env.contract.address)?;
    viewing_key_w(&mut deps.storage).save(&msg.viewing_key)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

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
        HandleMsg::UpdateConfig { admin } => handle::try_update_config(deps, env, admin),
        HandleMsg::Adapter(adapter) => match adapter {
            adapter::HandleMsg::Unbond { amount } => handle::unbond(deps, env, amount),
            adapter::HandleMsg::Claim { } => handle::claim(deps, env),
        },

        /*
        HandleMsg::Adapter(adapter::Receive {
            sender,
            from,
            amount,
            msg,
            ..
        } => handle::receive(deps, env, sender, from, amount, msg)),
        HandleMsg::Adapter(adapter::HandleMsg::Unbond { amount } => handle::unbond(deps, env, amount)),
        HandleMsg::Adapter(adapter::HandleMsg::Claim { } => handle::claim(deps, env)),
        */
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        // All delegations
        QueryMsg::Delegations {} => to_binary(&query::delegations(deps)?),
        QueryMsg::Adapter(adapter) => match adapter {
            adapter::QueryMsg::Balance {} => {Err(StdError::generic_err("not implemented"))},
            adapter::QueryMsg::Rewards {} => to_binary(&query::rewards(deps)?),
            adapter::QueryMsg::Unbondings {} => {Err(StdError::generic_err("not implemented"))},
        }
    }
}
