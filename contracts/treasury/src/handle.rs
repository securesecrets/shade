use cosmwasm_std::{
    debug_print, to_binary, Api, Binary,
    Env, Extern, HandleResponse,
    Querier, StdError, StdResult, Storage, 
    CosmosMsg, HumanAddr,
    Uint128, Decimal,
};
use secret_toolkit::{
    snip20::{
        token_info_query,
        register_receive_msg, 
        set_viewing_key_msg,
        send_msg,
    },
};

use shade_protocol::{
    treasury::{
        HandleAnswer, 
        Application,
    },
    snip20::{
        Snip20Asset,
        token_config_query,
    },
    asset::Contract,
    generic_response::ResponseStatus,
};

use crate::state::{
    config_w, config_r, 
    assets_r, assets_w,
    viewing_key_r,
    apps_r, apps_w,
    allocations_r,
    allocations_w,
};

pub fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _sender: HumanAddr,
    _from: HumanAddr,
    amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<HandleResponse> {

    let asset = assets_r(&deps.storage).load(env.message.sender.to_string().as_bytes())?;
    debug_print!("Treasured {} u{}", amount, asset.token_info.symbol);

    let allocations = allocations_r(&deps.storage).load(env.message.sender.to_string().as_bytes())?;

    for app in allocations {
        let allocation = amount.multiply_ratio(app.allocation, Uint128(1));
        messages.push(send_msg(app.contract.address,
                               capture_amount,
                               None,
                               None,
                               1,
                               asset.contract.code_hash.clone(),
                               asset.contract.address.clone())?);
        debug_print!("Treasured {} u{}", amount, asset.token_info.symbol);
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::Receive {
            status: ResponseStatus::Success,
        })?),
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
    let token_info = 

    assets_w(&mut deps.storage).save(contract.address.to_string().as_bytes(), &Snip20Asset {
        contract: contract.clone(),
        token_info: token_info_query(&deps.querier, 1,
                                      contract.code_hash.clone(),
                                      contract.address.clone())?,
        token_config: Some(token_config_query(&deps.querier, contract.clone())?),
    })?;

    // Register contract in asset
    messages.push(register_receive_msg(
        env.contract_code_hash.clone(),
        None,
        256,
        contract.code_hash.clone(),
        contract.address.clone(),
    )?);

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

pub fn register_app<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    application: Contract,
    asset: HumanAddr,
    //token: Option<Contract>,
    allocation: Decimal,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    /* ADMIN ONLY */
    if env.message.sender != config.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    match assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        None => {
            return Err(StdError::GenericErr { msg: "Unregistered asset".to_string(), backtrace: None });
        },
        Some(asset) => {

            apps_w(&mut deps.storage).update(|mut apps| {
                if !apps.contains(&application.address) {
                    apps.push(application.address.clone());
                }
                Ok(apps)
            })?;

            //let mut reserves = Decimal.one();

            // Remove old instance and add new data 
            // to assets allocation list
            allocations_w(&mut deps.storage).update(asset.contract.address.to_string().as_bytes(), |allocations| {

                let mut allocs = match allocations {
                    None => { vec![] }
                    Some(allocs) => { allocs }
                };

                // remove old instance of app
                allocs.remove(allocs.iter().position(|a| a.contract.address == application.address.clone()).unwrap());
                allocs.push(Application {
                    contract: application,
                    allocation,
                });

                /*
                for a in allocs {
                    reserves = reserves - a.allocation;
                }
                */

                Ok(allocs)
            })?;

            return Ok(HandleResponse {
                messages: vec![],
                log: vec![],
                data: Some( to_binary( 
                    &HandleAnswer::RegisterApp {
                        status: ResponseStatus::Success } 
                    )? 
                )
            });
        }
    }
}

/*
pub fn rebalance<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: HumanAddr,
) -> StdResult<HandleResponse> {

    let mut messages = vec![];

    let allocations = allocations_r(&deps.storage).load(asset.to_string().as_bytes())?;

    let total = Decimal.one()

    for alloc in allocations {

    }


    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some( to_binary( 
            &HandleAnswer::Rebalance {
                status: ResponseStatus::Success } 
            )? 
        )
    })
}
*/
