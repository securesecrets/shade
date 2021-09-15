use cosmwasm_std::{debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage, CosmosMsg, HumanAddr, Uint128, from_binary, Empty};
use shade_protocol::asset::Contract;
use crate::state::{config_r, config_w};
use shade_protocol::airdrop::HandleAnswer;
use shade_protocol::generic_response::ResponseStatus;


pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    admin: Option<HumanAddr>,
    airdrop_snip20: Option<Contract>,
    prefered_validator: Option<HumanAddr>,
    start_date: Option<u64>,
    end_date: Option<u64>,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;
    // Check if admin
    if env.message.sender != config.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Save new info
    let mut config = config_w(&mut deps.storage);
    config.update(|mut state| {
        if let Some(admin) = admin {
            state.owner = admin;
        }
        if let Some(airdrop_snip20) = airdrop_snip20 {
            state.airdrop_snip20 = airdrop_snip20;
        }
        if let Some(prefered_validator) = prefered_validator {
            state.prefered_validator = prefered_validator;
        }
        if let Some(start_date) = start_date {
            state.start_date;
        }
        if let Some(end_date) = end_date {
            state.end_date;
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
    env: Env,
) -> StdResult<HandleResponse> {
    // Check if walled in airdrop

    // Redeem and then cancel


    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::Redeem {
            status: ResponseStatus::Success } )? )
    })
}