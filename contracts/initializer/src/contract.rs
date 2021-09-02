use cosmwasm_std::{debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError, StdResult, Storage, HumanAddr};
use crate::state::{config_w};
use shade_protocol::initializer::{InitMsg, InitializerConfig, HandleMsg, QueryMsg};
use secret_toolkit::utils::InitCallback;
use crate::query::query_contracts;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let mut state = InitializerConfig {
        contracts: vec![],
    };

    let mut messages = vec![];

    // Snip20 configs
    let coin_config = Option::from(shade_protocol::snip20::InitConfig {
        public_total_supply: Option::from(true),
        enable_deposit: Option::from(false),
        enable_redeem: Option::from(false),
        enable_mint: Option::from(true),
        enable_burn: Option::from(true)
    });

    // Initialize Silk
    let silk_init_msg = shade_protocol::snip20::InitMsg {
        name: "Silk".to_string(),
        admin: Option::from(match msg.silk.admin {
            None => env.message.sender.clone(),
            Some(admin) => admin
        }),
        symbol: "SILK".to_string(),
        decimals: 6,
        initial_balances: msg.silk.initial_balances,
        prng_seed: msg.silk.prng_seed,
        config: coin_config.clone()
    };
    state.contracts.push(msg.silk.label.clone());
    messages.push(silk_init_msg.to_cosmos_msg(msg.silk.label.clone(),
                                                    msg.snip20_id,
                                                    msg.snip20_code_hash.clone(),
                                                    None)?);


    // Initialize Shade
    let shade_init_msg = shade_protocol::snip20::InitMsg {
        name: "Shade".to_string(),
        admin: Option::from(match msg.shade.admin {
            None => env.message.sender.clone(),
            Some(admin) => admin
        }),
        symbol: "SHD".to_string(),
        decimals: 6,
        initial_balances: msg.shade.initial_balances,
        prng_seed: msg.shade.prng_seed,
        config: coin_config.clone()
    };
    state.contracts.push(msg.shade.label.clone());
    messages.push(shade_init_msg.to_cosmos_msg(msg.shade.label.clone(),
                                                    msg.snip20_id,
                                                    msg.snip20_code_hash.clone(),
                                                    None)?);

    debug_print!("Contract was initialized by {}", env.message.sender);
    config_w(&mut deps.storage).save(&state)?;
    Ok(InitResponse {
        messages,
        log: vec![]
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    return Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: None
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Contracts {} => to_binary(&query_contracts(deps)?),
    }
}