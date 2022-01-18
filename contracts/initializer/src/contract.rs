use crate::{query, handle, state::{config_w, shade_w}};
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
    StdResult,
    Storage,
};
use secret_toolkit::utils::InitCallback;
use shade_protocol::initializer::{
    HandleMsg,
    InitMsg,
    Config,
    QueryMsg,
    Snip20InitHistory,
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = Config {
        admin: msg.admin.unwrap_or(env.message.sender.clone()),
        snip20_id: msg.snip20_id,
        snip20_code_hash: msg.snip20_code_hash.clone()
    };
    config_w(&mut deps.storage).save(&state)?;

    // Snip20 configs
    let coin_config = Some(shade_protocol::snip20::InitConfig {
        public_total_supply: Option::from(true),
        enable_deposit: Option::from(false),
        enable_redeem: Option::from(false),
        enable_mint: Option::from(true),
        enable_burn: Option::from(true),
    });

    // Initialize Shade
    let shade_init_msg = shade_protocol::snip20::InitMsg {
        name: "Shade".to_string(),
        admin: Some(msg.shade.admin.unwrap_or_else(|| env.message.sender.clone())),
        symbol: "SHD".to_string(),
        decimals: 8,
        initial_balances: msg.shade.initial_balances.clone(),
        prng_seed: msg.shade.prng_seed,
        config: coin_config,
    };
    shade_w(&mut deps.storage).save(&Snip20InitHistory {
        label: msg.shade.label.clone(),
        balances: msg.shade.initial_balances.clone(),
    })?;

    let messages = vec![shade_init_msg.to_cosmos_msg(
        msg.shade.label,
        msg.snip20_id,
        msg.snip20_code_hash,
        None,
    )?];

    Ok(InitResponse {
        messages,
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::SetAdmin { admin
        } => handle::set_admin(deps, &env, admin),

        HandleMsg::InitSilk { silk, ticker, decimals
        } => handle::init_silk(deps, &env, silk, ticker, decimals)
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Contracts {} => to_binary(&query::contracts(deps)?),
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
    }
}
