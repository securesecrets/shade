use cosmwasm_std::{debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdResult, Storage, Uint128, HumanAddr};
use shade_protocol::{
    airdrop::{
        InitMsg, HandleMsg,
        QueryMsg, Config
    }
};
use crate::{state::{config_w, sn_delegators_w},
            handle::{try_update_config, try_redeem},
            query };
use shade_protocol::airdrop::{StoredDelegator, ValidatorWeight};
use secret_toolkit::snip20::token_info_query;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let config = Config{
        owner: match msg.admin {
            None => { env.message.sender.clone() }
            Some(admin) => { admin }
        },
        airdrop_snip20: msg.airdrop_snip20.clone(),
        airdrop_decimals: token_info_query(&deps.querier, 1,
                                           msg.airdrop_snip20.code_hash,
                                           msg.airdrop_snip20.address)?.decimals,
        sn_validator_weights: match msg.sn_validator_weights {
            None => vec![],
            Some(weights) => weights
        },
        sn_banned_validators: match msg.sn_banned_validators {
            None => vec![],
            Some(banned) => banned
        },
        sn_whale_cap: msg.sn_whale_cap,
        start_date: match msg.start_date {
            None => env.block.time,
            Some(date) => date
        },
        end_date: msg.end_date
    };

    config_w(&mut deps.storage).save(&config)?;

    let mut delegators = sn_delegators_w(&mut deps.storage);
    // Store the delegators list
    for delegator in msg.sn_snapshot {
        delegators.save(delegator.address.to_string().as_bytes(), &StoredDelegator{
            address: delegator.address,
            delegations: delegator.delegations,
            redeemed: false
        });
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
            admin, airdrop_snip20, sn_validator_weights,
            sn_banned_validators, sn_whale_cap,
            start_date, end_date
        } => try_update_config(deps, env, admin, airdrop_snip20, sn_validator_weights,
                               sn_banned_validators, sn_whale_cap, start_date, end_date),
        HandleMsg::Redeem { } => try_redeem(deps, env),
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
            &query::airdrop_amount(&deps, address)),
    }
}