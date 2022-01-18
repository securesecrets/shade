use cosmwasm_std::{Api, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, to_binary};
use secret_toolkit::utils::InitCallback;
use shade_protocol::generic_response::ResponseStatus::Success;
use shade_protocol::initializer::{HandleAnswer, Snip20ContractInfo, Snip20InitHistory};
use crate::state::{config_r, config_w, silk_r, silk_w};

pub fn set_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    admin: HumanAddr
) -> StdResult<HandleResponse> {
    let mut config = config_r(&deps.storage).load()?;

    if env.message.sender != config.admin {
        return Err(StdError::unauthorized())
    }

    config.admin = admin;

    config_w(&mut deps.storage).save(&config)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetAdmin {
            status: Success,
        })?),
    })
}

pub fn init_silk<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    silk: Snip20ContractInfo,
    ticker: String,
    decimals: u8
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    if env.message.sender != config.admin {
        return Err(StdError::unauthorized())
    }

    if silk_r(&deps.storage).may_load()?.is_some() {
        return Err(StdError::generic_err("Silk already initialized"))
    }

    // Snip20 configs
    let coin_config = Some(shade_protocol::snip20::InitConfig {
        public_total_supply: Option::from(true),
        enable_deposit: Option::from(false),
        enable_redeem: Option::from(false),
        enable_mint: Option::from(true),
        enable_burn: Option::from(true),
    });

    // Initialize Silk
    let silk_init_msg = shade_protocol::snip20::InitMsg {
        name: "Silk".to_string(),
        admin: Some(silk.admin.unwrap_or_else(|| env.message.sender.clone())),
        symbol: ticker,
        decimals,
        initial_balances: silk.initial_balances.clone(),
        prng_seed: silk.prng_seed,
        config: coin_config.clone(),
    };
    silk_w(&mut deps.storage).save(&Snip20InitHistory {
        label: silk.label.clone(),
        balances: silk.initial_balances.clone(),
    })?;
    let messages = vec![silk_init_msg.to_cosmos_msg(
        silk.label.clone(),
        config.snip20_id,
        config.snip20_code_hash,
        None,
    )?];

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::InitSilk {
            status: Success,
        })?),
    })
}