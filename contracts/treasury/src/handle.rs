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
        Snip20Asset, fetch_snip20,
        token_config_query,
    },
    asset::Contract,
    generic_response::ResponseStatus,
};

use crate::state::{
    config_w, config_r, 
    assets_r, assets_w,
    viewing_key_r,
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

    let mut messages = vec![];

    for app in allocations {

        let allocation = allocate_amount(amount, app.allocation);

        debug_print!("Allocating {}/{} u{} to {}", allocation, amount, asset.token_info.symbol, app.contract.address);

        messages.push(send_msg(app.contract.address,
                               allocation,
                               None,
                               None,
                               1,
                               asset.contract.code_hash.clone(),
                               asset.contract.address.clone())?);
    }

    Ok(HandleResponse {
        messages: messages,
        log: vec![],
        data: Some( to_binary( &HandleAnswer::Receive {
            status: ResponseStatus::Success,
        })?),
    })
}

/* Verifies the set of allocations is < 100%
 */
/*
pub fn validate_allocations(
    apps: Vec<Application>,
    reserves: Option<Decimal>,
) -> bool {

    let allocated = Decimal::zero();
    for app in apps {
        allocated = allocated + app.allocation;
    }

    allocated < Decimal::one()
}
*/

pub fn allocate_amount(
    amount: Uint128, 
    allocation: Decimal
) -> Uint128 {

    amount * allocation
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
            status: ResponseStatus::Success } 
        )?)
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

    assets_w(&mut deps.storage).save(contract.address.to_string().as_bytes(), &fetch_snip20(&contract, &deps.querier)?)?;
    allocations_w(&mut deps.storage).save(contract.address.to_string().as_bytes(), &vec![])?;

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
        )?)
    })
}

pub fn register_app<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    contract: Contract,
    asset: HumanAddr,
    //token: Option<Contract>,
    allocation: Decimal,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    /* ADMIN ONLY */
    if env.message.sender != config.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    if (assets_r(&deps.storage).may_load(asset.to_string().as_bytes())?).is_none() {
        return Err(StdError::GenericErr {
            msg: "Unregistered asset".to_string(),
            backtrace: None,
        });
    }

    allocations_w(&mut deps.storage).update(asset.to_string().as_bytes(), |allocations| {

        // initialize list if it doesn't exist
        let mut app_list = match allocations {
            None => { vec![] }
            Some(a) => { a }
        };

        // Remove old instance of this contract
        if let Some(pos) = app_list.iter().position(|a| a.contract.address == contract.address.clone()) {
            app_list.remove(pos);
        }
        app_list.push(
            Application {
                contract,
                allocation,
            }
        );

        // Validate total allocation
        let mut total = Decimal::zero();
        for app in &app_list {
            total = total + app.allocation;
        }

        if total >= Decimal::one() {
            return Err(StdError::GenericErr {
                msg: "Allocated total exceeds 100%".to_string(),
                backtrace: None,
            });
        }

        Ok(app_list)
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

/*
pub fn rebalance<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: HumanAddr,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    if env.message.sender != config.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let mut messages = vec![];

    let total = Decimal.one();

    if let Some(asset) = assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
            
        for app in allocations_r(&deps.storage).load(asset.contract.address.to_string().as_bytes())? {
            let allocation = allocate_amount(amount, app.allocation);

            debug_print!("Allocating {} u{} to {}", allocation, asset.token_info.symbol, app.contract.address);
            messages.push(send_msg(app.contract.address,
                                   allocation,
                                   None,
                                   None,
                                   1,
                                   asset.contract.code_hash.clone(),
                                   asset.contract.address.clone())?);
        }
    }


    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some( to_binary( 
            &HandleAnswer::Receive {
                status: ResponseStatus::Success } 
            )? 
        )
    })
}
*/
