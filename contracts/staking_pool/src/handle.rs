use cosmwasm_std::{
    debug_print, to_binary, Api, Binary,
    Env, Extern, HandleResponse,
    Querier, StdError, StdResult, Storage, 
    CosmosMsg, HumanAddr, Uint128
};
use secret_toolkit::{
    snip20::{
        token_info_query,
        register_receive_msg, 
        set_viewing_key_msg,
    },
};

use shade_protocol::{
    staking_pool::HandleAnswer,
    asset::Contract,
    generic_response::ResponseStatus,
};

use crate::state::{
    config_w, config_r, 
    delegations_w, delegations_r,
};

pub fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _sender: HumanAddr,
    _from: HumanAddr,
    amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<HandleResponse> {

    let sscrt = assets_r(&deps.storage);

    /* 
     * if sscrt:
     *   Redeem <amount> sSCRT -> SCRT
     *   Choose Validator 
     *   Stake SCRT
     *
     * elif app_token:
     *   Burn tokens
     *   Begin unbonding of amount to be given to user
    */

    debug_print!("Received {}", amount);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::Receive {
            status: ResponseStatus::Success,
        } )? ),
    })
}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Option<HumanAddr>,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    if env.message.sender != config.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Save new info
    let mut config = config_w(&mut deps.storage);
    config.update(|mut state| {
        if let Some(owner) = owner {
            state.owner = owner;
        }
        Ok(state)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success } )? )
    })
}

pub fn try_register_asset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    contract: &Contract,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    if env.message.sender != config.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let mut messages = vec![];
    let token_info = token_info_query(&deps.querier, 1,
                                      contract.code_hash.clone(),
                                      contract.address.clone())?;

    assets_w(&mut deps.storage).save(contract.address.to_string().as_bytes(), &Snip20Asset {
        contract: contract.clone(),
        token_info,
    })?;

    // Register contract in asset
    messages.push(register_receive(&env, &contract)?);

    // Set viewing key
    messages.push(set_viewing_key_msg(
                    viewing_key_r(&deps.storage).load()?,
                    None,
                    1,
                    contract.code_hash.clone(),
                    contract.address.clone())?);


    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some( to_binary( 
            &HandleAnswer::RegisterAsset {
                status: ResponseStatus::Success } 
            )? 
        )
    })
}

pub fn register_receive (
    env: &Env,
    contract: &Contract,
) -> StdResult<CosmosMsg> {
    let cosmos_msg = register_receive_msg(
        env.contract_code_hash.clone(),
        None,
        256,
        contract.code_hash.clone(),
        contract.address.clone(),
    );

    cosmos_msg
}
