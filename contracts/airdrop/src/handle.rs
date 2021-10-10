use cosmwasm_std::{debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage, CosmosMsg, HumanAddr, Uint128, from_binary, Empty};
use shade_protocol::asset::Contract;
use crate::state::{config_r, config_w, reward_r, claim_status_w, claim_status_r};
use shade_protocol::airdrop::{HandleAnswer};
use shade_protocol::generic_response::ResponseStatus;
use secret_toolkit::snip20::mint_msg;

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    admin: Option<HumanAddr>,
    start_date: Option<u64>,
    end_date: Option<u64>,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;
    // Check if admin
    if env.message.sender != config.admin {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Save new info
    let mut config = config_w(&mut deps.storage);
    config.update(|mut state| {
        if let Some(admin) = admin {
            state.admin = admin;
        }
        if let Some(start_date) = start_date {
            state.start_date = start_date;
        }
        if let Some(end_date) = end_date {
            state.end_date = Some(end_date);
        }

        Ok(state)
    });

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success } )? )
    })
}

pub fn try_redeem<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    // Check if airdrop started
    if env.block.time < config.start_date {
        return Err(StdError::Unauthorized { backtrace: None })
    }
    if let Some(end_date) = config.end_date {
        if env.block.time > end_date {
            return Err(StdError::Unauthorized { backtrace: None })
        }
    }

    let key = env.message.sender.to_string();

    // Check if user is eligible
    if claim_status_r(&deps.storage).load(key.as_bytes())? {
        return Err(StdError::GenericErr { msg: "Already Claimed".to_string(), backtrace: None })
    }

    // Load the user's reward
    let airdrop = reward_r(&deps.storage).load(key.as_bytes())?;

    // Redeem
    let messages =  vec![mint_msg(env.message.sender.clone(), airdrop.amount,
                                  None, 1,
                                  config.airdrop_snip20.code_hash,
                                  config.airdrop_snip20.address)?];

    // Mark reward as redeemed
    claim_status_w(&mut deps.storage).update(key.as_bytes(), |claimed| {
        Ok(true)
    })?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some( to_binary( &HandleAnswer::Redeem {
            status: ResponseStatus::Success } )? )
    })
}