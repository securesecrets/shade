use cosmwasm_std::{
    to_binary, Api, Binary,
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
        QueryAnswer,
        Allocation,
        Config,
    },
    snip20::{
        Snip20Asset, fetch_snip20,
        token_config_query,
    },
    asset::Contract,
    generic_response::ResponseStatus,
};

use crate::{
    query,
    state::{
        config_w, config_r, 
        assets_r, assets_w,
        viewing_key_r,
        allocations_r, allocations_w,
        reserves_r, reserves_w,
    },
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
    //debug_print!("Treasured {} u{}", amount, asset.token_info.symbol);

    let mut messages = vec![];

    allocations_w(&mut deps.storage).update(asset.contract.address.to_string().as_bytes(), |allocs| {

        let mut alloc_list = match allocs {
            None => { vec![] }
            Some(a) => { a }
        };

        for mut alloc in &mut alloc_list {

            match alloc {
                Allocation::Reserves { allocation } => {
                },
                Allocation::Staking { allocation, contract } => {

                    let allocation = amount * *allocation;
                    //a.amount_allocated += allocation;

                    //debug_print!("Staking {}/{} u{} to {}", allocation, amount, asset.token_info.symbol, contract.address);

                    messages.push(
                        send_msg(
                                contract.address.clone(),
                                allocation,
                                None,
                                None,
                                1,
                                asset.contract.code_hash.clone(),
                                asset.contract.address.clone(),
                        )?
                    );
                },

                Allocation::Application { contract, allocation, token } => {
                    //debug_print!("Applications Unsupported {}/{} u{} to {}", allocation, amount, asset.token_info.symbol, contract.address);
                },
                Allocation::Pool { contract, allocation, secondary_asset, token } => {
                    //debug_print!("Pools Unsupported {}/{} u{} to {}", allocation, amount, asset.token_info.symbol, contract.address);
                },
            };
        }

        Ok(alloc_list)
    })?;

    Ok(HandleResponse {
        messages: messages,
        log: vec![],
        data: Some( to_binary(
            &HandleAnswer::Receive {
                status: ResponseStatus::Success,
            }
        )?),
    })
}

/* Verifies the set of apps is < 100%
 */
/*
pub fn validate_apps(
    apps: Vec<Application>,
    reserves: Option<Decimal>,
) -> bool {

    let allocated = Decimal::zero();
    for app in apps {
        allocated = allocated + app.allocation;
    }

    allocated < Decimal::one()
}

pub fn allocate_amount(
    amount: Uint128, 
    allocation: Decimal
) -> Uint128 {

    amount * allocation
}
*/

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: Config,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    if env.message.sender != config.admin {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    config_w(&mut deps.storage).save(&config);

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
    reserves: Option<Decimal>,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    if env.message.sender != config.admin {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let mut messages = vec![];

    assets_w(&mut deps.storage).save(
        contract.address.to_string().as_bytes(), 
        &fetch_snip20(&contract, &deps.querier)?
    )?;

    let allocs = match reserves {
        Some(r) => { vec![Allocation::Reserves { allocation: r } ] }
        None => { vec![] }
    };

    allocations_w(&mut deps.storage).save(
        contract.address.to_string().as_bytes(), 
        &vec![]
    )?;

    reserves_w(&mut deps.storage).save(
        contract.address.to_string().as_bytes(), 
        &match reserves {
            None => { Decimal::zero() }
            Some(r) => { r }
        },
    )?;

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

pub fn register_allocation<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: HumanAddr,
    allocation: Allocation,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    /* ADMIN ONLY */
    if env.message.sender != config.admin {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let full_asset = match assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        Some(a) => { a }
        None => {
            return Err(StdError::GenericErr {
                msg: "Unregistered asset".to_string(),
                backtrace: None,
            });
        }
    };

    let alloc_portion = match allocation {
        Allocation::Reserves { allocation } => allocation,
        Allocation::Staking { contract, allocation } => allocation,
        Allocation::Application { contract, allocation, token } => allocation,
        Allocation::Pool { contract, allocation, secondary_asset, token } => allocation,
    };

    let allocated_amount = Decimal::zero();

    allocations_w(&mut deps.storage).update(asset.to_string().as_bytes(), |apps| {

        // initialize list if it doesn't exist
        let mut app_list = match apps {
            None => { vec![] }
            Some(a) => { a }
        };

        // Remove old instance of this contract
        // TODO: need type comparison or something? gonna worry about it later
        /*
        for app in app_list.iter_mut() {

            match app {
                Allocation::Reserves(a) {
                    alloc_amount
                }
            }
            app_list.remove(pos);
        }
        */

        // Validate addition does not exceed 100%
        for app in &app_list {

            allocated_amount = allocated_amount + match app {
                Allocation::Reserves { allocation } => Decimal::zero(),
                Allocation::Staking { contract, mut allocation } => allocation,
                Allocation::Application { contract, mut allocation, token } => allocation,
                Allocation::Pool { contract, mut allocation, secondary_asset, token }=> allocation,
            };
        }

        if (allocated_amount + alloc_portion) >= Decimal::one() {
            return Err(StdError::GenericErr {
                msg: "Invalid allocation total exceeding 100%".to_string(),
                backtrace: None,
            });
        }

        app_list.push(allocation);

        Ok(app_list)
    })?;

    let liquid_portion = Decimal::one() - allocated_amount;

    let liquid_balance = match query::balance(&deps, &asset)? {
        QueryAnswer::Balance { amount } => amount,
        _ => {
            return Err(StdError::GenericErr {
                msg: "Unexpected balance response".to_string(),
                backtrace: None,
            });
        }
    };

    // Determine how much of current balance is to be allocated
    let to_allocate = liquid_balance * (alloc_portion / liquid_portion);

    Ok(HandleResponse {
        messages: vec![
            send_msg(
                    full_asset.contract.address.clone(),
                    to_allocate,
                    None,
                    None,
                    1,
                    full_asset.contract.code_hash.clone(),
                    full_asset.contract.address.clone(),
            )?
        ],
        log: vec![],
        data: Some( to_binary( 
            &HandleAnswer::RegisterApp {
                status: ResponseStatus::Success } 
            )? 
        )
    })
}

/*
pub fn rebalance<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: HumanAddr,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    if env.message.sender != config.admin {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let mut messages = vec![];

    let total = Decimal.one();

    if let Some(asset) = assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
            
        for app in allocations_r(&deps.storage).load(asset.contract.address.to_string().as_bytes())? {
            let allocation = amount * app.allocation;

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
