use cosmwasm_std;
use cosmwasm_std::{
    to_binary, Api, Binary,
    Env, Extern, HandleResponse,
    Querier, StdError, StdResult, Storage, 
    Uint128, HumanAddr,
    from_binary,
};
use secret_toolkit::snip20::{
    register_receive_msg, set_viewing_key_msg, token_info_query,
    send_msg,
};

use shade_protocol::{
    treasury::{
        HandleAnswer, 
        QueryAnswer,
        Allocation,
        Config,
        Flag,
    },
    snip20::{
        Snip20Asset, fetch_snip20,
        token_config_query,
    },
    asset::Contract,
    generic_response::ResponseStatus,
    //math::Uint128,
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
    msg: Option<Binary>,
) -> StdResult<HandleResponse> {

    let asset = assets_r(&deps.storage).load(env.message.sender.to_string().as_bytes())?;
    //debug_print!("Treasured {} u{}", amount, asset.token_info.symbol);
    // skip the rest if the send the "unallocated" flag
    if let Some(f) = msg {
        let flag: Flag = from_binary(&f)?;
        if flag.flag == "unallocated" {
            return Ok(HandleResponse {
                messages: vec![],
                log: vec![],
                data: Some( to_binary(
                    &HandleAnswer::Receive {
                        status: ResponseStatus::Success,
                    }
                )?),
            })
        }
    };

    let mut messages = vec![];

    allocations_w(&mut deps.storage).update(asset.contract.address.to_string().as_bytes(), |allocs| {

        let mut alloc_list = match allocs {
            None => { vec![] }
            Some(a) => { a }
        };

        for alloc in &mut alloc_list {

            match alloc {
                Allocation::Reserves { allocation } => { },
                Allocation::Rewards { allocation, contract } => {
                    messages.push(
                        send_msg(
                                contract.address.clone(),
                                amount.multiply_ratio(*allocation, 10u128.pow(18)),
                                None,
                                None,
                                None,
                                1,
                                asset.contract.code_hash.clone(),
                                asset.contract.address.clone(),
                        )?
                    );
                },
                Allocation::Staking { allocation, contract } => {

                    //debug_print!("Staking {}/{} u{} to {}", allocation, amount, asset.token_info.symbol, contract.address);

                    messages.push(
                        send_msg(
                                contract.address.clone(),
                                amount.multiply_ratio(*allocation, 10u128.pow(18)),
                                None,
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

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: Config,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    if env.message.sender != config.admin {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    config_w(&mut deps.storage).save(&config)?;

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
    reserves: Option<Uint128>,
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
            None => { Uint128::zero() }
            Some(r) => { r }
        },
    )?;

    // Register contract in asset
    messages.push(
        register_receive_msg(
            env.contract_code_hash.clone(),
            None,
            256,
            contract.code_hash.clone(),
            contract.address.clone(),
        )?
    );

    // Set viewing key
    messages.push(
        set_viewing_key_msg(
            viewing_key_r(&deps.storage).load()?,
            None,
            1,
            contract.code_hash.clone(),
            contract.address.clone()
        )?
    );

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
    alloc: Allocation,
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

    let liquid_balance: Uint128 = match query::balance(&deps, &asset)? {
        QueryAnswer::Balance { amount } => amount,
        _ => {
            return Err(StdError::GenericErr {
                msg: "Unexpected response for balance".to_string(),
                backtrace: None,
            });
        }
    };

    let alloc_portion = *match &alloc {
        Allocation::Reserves { allocation } => allocation,
        Allocation::Rewards { contract, allocation } => allocation,
        Allocation::Staking { contract, allocation } => allocation,
        Allocation::Application { contract, allocation, token } => allocation,
        Allocation::Pool { contract, allocation, secondary_asset, token } => allocation,
    };

    let alloc_address = match &alloc {
        Allocation::Staking { contract, allocation } => Some(contract.address.clone()),
        Allocation::Application { contract, allocation, token } => Some(contract.address.clone()),
        Allocation::Pool { contract, allocation, secondary_asset, token } => Some(contract.address.clone()),
        _ => None,
    };

    let mut allocated_portion = Uint128::zero();

    allocations_w(&mut deps.storage).update(asset.to_string().as_bytes(), |apps| {

        // initialize list if it doesn't exist
        let mut app_list = match apps {
            None => { vec![] }
            Some(a) => { a }
        };

        // Remove old instance of this contract
        // TODO: need type comparison or something? gonna worry about it later
        let mut existing_index = None;
        for (i, app) in app_list.iter_mut().enumerate() {

            if let Some(address) = match app {
                Allocation::Reserves { allocation } => None,
                Allocation::Rewards { contract, allocation } => Some(contract.address.clone()),
                Allocation::Staking { contract, allocation } => Some(contract.address.clone()),
                Allocation::Application { contract, allocation, token } => Some(contract.address.clone()),
                Allocation::Pool { contract, allocation, secondary_asset, token } => Some(contract.address.clone()),
            } {
                match &alloc_address {
                    Some(a) => {
                        if address == *a {
                            existing_index = Option::from(i);
                            break;
                        }
                    }
                    None => { }
                }
            }
            else {
                match alloc_address {
                    Some(_) => { }
                    None => { 
                        existing_index = Option::from(i);
                        break;
                    }
                }
            }
        }

        match existing_index {
            Some(i) => {
                app_list.remove(i);
            }
            _ => {}
        }

        // Validate addition does not exceed 100%
        for app in &app_list {

            allocated_portion = allocated_portion + match app {
                Allocation::Reserves { allocation } => Uint128::zero(),
                Allocation::Rewards { contract, allocation } => *allocation,
                Allocation::Staking { contract, allocation } => *allocation,
                Allocation::Application { contract, allocation, token } => *allocation,
                Allocation::Pool { contract, allocation, secondary_asset, token } => *allocation,
            };
        }

        
        if (allocated_portion + alloc_portion) >= Uint128(10u128.pow(18)) {
            return Err(StdError::GenericErr {
                msg: "Invalid allocation total exceeding 100%".to_string(),
                backtrace: None,
            });
        }

        app_list.push(alloc);

        Ok(app_list)
    })?;

    //TODO: get Uint128 math functions to do these things (untested)
    //TODO: re-add send_msg below
    /*
    let liquid_portion = (allocated_portion * liquid_balance) / allocated_portion;

    // Determine how much of current balance is to be allocated
    let to_allocate = liquid_balance - (alloc_portion / liquid_portion);
    */

    Ok(HandleResponse {
        messages: vec![
            /*
            send_msg(
                    full_asset.contract.address.clone(),
                    to_allocate,
                    None,
                    None,
                    1,
                    full_asset.contract.code_hash.clone(),
                    full_asset.contract.address.clone(),
            )?
            */
        ],
        log: vec![],
        data: Some( to_binary( 
            &HandleAnswer::RegisterApp {
                status: ResponseStatus::Success } 
            )? 
        )
    })
}
