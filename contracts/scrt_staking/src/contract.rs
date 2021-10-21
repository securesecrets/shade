use cosmwasm_std::{
    debug_print, to_binary, Api, Binary,
    Env, Extern, HandleResponse, InitResponse, 
    Querier, StdResult, Storage, 
};

use shade_protocol::{
    scrt_staking::{
        Config,
        InitMsg, 
        HandleMsg,
        QueryMsg,
    },
};

use secret_toolkit::{
    snip20::{
        token_info_query,
        register_receive_msg, 
        set_viewing_key_msg,
    },
};

use crate::{
    state::{
        viewing_key_w, viewing_key_r,
        config_w,
        self_address_w, self_address_r,
    },
    handle, query,
};


pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    let mut state = Config {
        owner: match msg.admin {
            None => { env.message.sender.clone() }
            Some(admin) => { admin }
        },
        sscrt: msg.sscrt,
        treasury: msg.treasury,
        validator_bounds: msg.validator_bounds,
    };

    config_w(&mut deps.storage).save(&state)?;

    self_address_w(&mut deps.storage).save(&env.contract.address)?;
    viewing_key_w(&mut deps.storage).save(&msg.viewing_key)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse {
        messages: vec![
            set_viewing_key_msg(
                viewing_key_r(&deps.storage).load()?,
                None,
                1,
                state.sscrt.code_hash.clone(),
                state.sscrt.address.clone(),
            )?,
            register_receive_msg(
                env.contract_code_hash.clone(),
                None,
                256,
                state.sscrt.code_hash.clone(),
                state.sscrt.address.clone(),
            )?,
        ],
        log: vec![]
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
        HandleMsg::UpdateConfig {
            owner,
        } => handle::try_update_config(deps, env, owner),
        // Begin unbonding of a certain amount of scrt
        HandleMsg::Unbond {
            validator,
        } => handle::unbond(deps, env, validator),
        // Collect a completed unbonding/rewards
        HandleMsg::Collect {
            validator,
        } => handle::collect(deps, env, validator),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query::config(deps)?),
        // All delegations
        QueryMsg::Delegations { } => to_binary(&query::delegations(deps)?),
        QueryMsg::Delegation { validator } => to_binary(&query::delegation(deps, validator)?),
    }
}
