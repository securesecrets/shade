use cosmwasm_std::{debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdResult, Storage, Uint128, HumanAddr};
use shade_protocol::{
    airdrop::{
        InitMsg, HandleMsg,
        QueryMsg, Config
    }
};
use crate::{state::{config_w, reward_w, claim_status_w},
            handle::{try_update_config, try_claim},
            query };
use secret_toolkit::snip20::token_info_query;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let config = Config{
        admin: match msg.admin {
            None => { env.message.sender.clone() }
            Some(admin) => { admin }
        },
        airdrop_snip20: msg.airdrop_snip20.clone(),
        start_date: match msg.start_time {
            None => env.block.time,
            Some(date) => date
        },
        end_date: msg.end_time
    };

    config_w(&mut deps.storage).save(&config)?;

    // Store the delegators list
    for reward in msg.rewards {
        let key = reward.address.to_string();

        reward_w(&mut deps.storage).save(key.as_bytes(), &reward)?;
        claim_status_w(&mut deps.storage).save(key.as_bytes(), &false)?;
    }

    Ok(InitResponse {
        messages: vec![],
        log: vec![]
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::UpdateConfig {
            admin, start_date, end_date
        } => try_update_config(deps, env, admin, start_date, end_date),
        HandleMsg::Claim { } => try_claim(deps, &env),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig { } => to_binary(&query::config(&deps)?),
        QueryMsg::GetDates { } => to_binary(&query::dates(&deps)?),
        QueryMsg::GetEligibility { address } => to_binary(
            &query::airdrop_amount(&deps, address)?),
    }
}