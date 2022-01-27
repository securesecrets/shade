use cosmwasm_std;
use cosmwasm_std::{
    CosmosMsg,
    from_binary, to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, Querier, StdError,
    StdResult, Storage, Uint128,
};
use secret_toolkit;
use secret_toolkit::utils::Query;
use secret_toolkit::snip20::{register_receive_msg, send_msg, set_viewing_key_msg, increase_allowance_msg, decrease_allowance_msg, allowance_query};

use shade_protocol::{
    asset::Contract,
    generic_response::ResponseStatus,
    //math::Uint128,
    snip20,
    treasury::{Allocation, Config, Flag, HandleAnswer, QueryAnswer},
};

use crate::{
    query,
    state::{
        allocations_r, allocations_w, 
        assets_r, assets_w, 
        asset_list_r, asset_list_w,
        config_r, config_w, viewing_key_r,
        last_allowance_refresh_r,
        last_allowance_refresh_w,
    },
};
use chrono::prelude::*;

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
                data: Some(to_binary(&HandleAnswer::Receive {
                    status: ResponseStatus::Success,
                })?),
            });
        }
    };

    let mut messages = vec![];

    allocations_w(&mut deps.storage).update(
        asset.contract.address.to_string().as_bytes(),
        |allocs| {
            let mut alloc_list = allocs.unwrap_or(vec![]);

            for alloc in &mut alloc_list {
                match alloc {
                    Allocation::Reserves { allocation: _ } => {}
                    Allocation::Allowance { address: _, amount: _ } => {}

                    Allocation::Rewards {
                        allocation,
                        contract,
                    } => {
                        messages.push(send_msg(
                            contract.address.clone(),
                            amount.multiply_ratio(*allocation, 10u128.pow(18)),
                            None,
                            None,
                            None,
                            1,
                            asset.contract.code_hash.clone(),
                            asset.contract.address.clone(),
                        )?);
                    }
                    Allocation::Staking {
                        allocation,
                        contract,
                    } => {
                        //debug_print!("Staking {}/{} u{} to {}", allocation, amount, asset.token_info.symbol, contract.address);

                        messages.push(send_msg(
                            contract.address.clone(),
                            amount.multiply_ratio(*allocation, 10u128.pow(18)),
                            None,
                            None,
                            None,
                            1,
                            asset.contract.code_hash.clone(),
                            asset.contract.address.clone(),
                        )?);
                    }

                    Allocation::Application {
                        contract: _,
                        allocation: _,
                        token: _,
                    } => {
                        //debug_print!("Applications Unsupported {}/{} u{} to {}", allocation, amount, asset.token_info.symbol, contract.address);
                    }
                    Allocation::Pool {
                        contract: _,
                        allocation: _,
                        secondary_asset: _,
                        token: _,
                    } => {
                        //debug_print!("Pools Unsupported {}/{} u{} to {}", allocation, amount, asset.token_info.symbol, contract.address);
                    }
                };
            }

            Ok(alloc_list)
        },
    )?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Receive {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: Config,
) -> StdResult<HandleResponse> {
    
    let cur_config = config_r(&deps.storage).load()?;

    if env.message.sender != cur_config.admin {
        return Err(StdError::unauthorized());
    }

    config_w(&mut deps.storage).save(&config)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn refresh_allowance<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
) -> StdResult<HandleResponse> {

    let naive = NaiveDateTime::from_timestamp(env.block.time as i64, 0);
    let now: DateTime<Utc> = DateTime::from_utc(naive, Utc);

    let last_refresh_str = last_allowance_refresh_r(&mut deps.storage).load()?;

    let last_refresh_offset: Option<DateTime<FixedOffset>> = match DateTime::parse_from_rfc3339(&last_refresh_str) {
        Ok(r) => Some(r),
        Err(e) => None,
    };

    match last_refresh_offset {
        Some(parsed) => {

            let last_refresh: DateTime<Utc> = parsed.with_timezone(&Utc);//.utc(); //DateTime<Utc>::from(&last_refresh_offset)?;

            if now.year() <= last_refresh.year() && now.month() <= last_refresh.month() {
                return Err(StdError::generic_err(format!("Last refresh too recent: {}", last_refresh.to_rfc3339())));
            }
            
        }
        None => {
            return Err(StdError::generic_err("Failed to parse from memory"))
        }
    };

    last_allowance_refresh_w(&mut deps.storage).save(&now.to_rfc3339())?;

    Ok(HandleResponse {
        messages: do_allowance_refresh(&deps, &env)?,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RefreshAllowance {
            status: ResponseStatus::Success,
        })?),
    })
}

/* Not exposed as a tx
 */
pub fn do_allowance_refresh<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: &Env,
) -> StdResult<Vec<CosmosMsg>> {

    let mut messages = vec![];

    // could be from env (might be faster)?
    let key = viewing_key_r(&deps.storage).load()?;

    for asset in asset_list_r(&deps.storage).load()? {
        for alloc in allocations_r(&deps.storage).load(&asset.to_string().as_bytes())? {
            match alloc {
                Allocation::Allowance { address, amount }=> {

                    let full_asset = assets_r(&deps.storage).load(asset.to_string().as_bytes())?;
                    // Determine current allowance
                    let cur_allowance = allowance_query(
                        &deps.querier,
                        env.contract.address.clone(),
                        address.clone(),
                        key.clone(),
                        1,
                        full_asset.contract.code_hash.clone(),
                        full_asset.contract.address.clone()
                    )?;

                    if amount > cur_allowance.allowance {

                        // Increase to monthly allowance amount
                        messages.push(
                            increase_allowance_msg(
                                address.clone(),
                                (amount - cur_allowance.allowance)?,
                                None,
                                None,
                                1,
                                full_asset.contract.code_hash.clone(),
                                full_asset.contract.address.clone(),
                            )?
                        );
                    }
                    else if amount < cur_allowance.allowance {

                        // Decrease to monthly allowance
                        messages.push(
                            decrease_allowance_msg(
                                address.clone(),
                                (cur_allowance.allowance - amount)?,
                                None,
                                None,
                                1,
                                full_asset.contract.code_hash.clone(),
                                full_asset.contract.address.clone(),
                            )?
                        );
                    }

                }
                _ => { }
            }
        }

    }

    Ok(messages)
}

pub fn try_register_asset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    contract: &Contract,
    reserves: Option<Uint128>,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    if env.message.sender != config.admin {
        return Err(StdError::unauthorized());
    }

    let mut messages = vec![];

    asset_list_w(&mut deps.storage).update(|mut list| {
        list.push(contract.address.clone());
        Ok(list)
    })?;
    assets_w(&mut deps.storage).save(
        contract.address.to_string().as_bytes(),
        &snip20::fetch_snip20(&contract, &deps.querier)?,
    )?;

    let allocs = match reserves {
        Some(r) => {
            vec![Allocation::Reserves { allocation: r }]
        }
        None => {
            vec![]
        }
    };

    allocations_w(&mut deps.storage).save(contract.address.to_string().as_bytes(), &allocs)?;

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
        contract.address.clone(),
    )?);

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RegisterAsset {
            status: ResponseStatus::Success,
        })?),
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
        return Err(StdError::unauthorized());
    }

    let full_asset = match assets_r(&deps.storage).may_load(asset.to_string().as_bytes())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Unregistered asset"));
        }
    };

    let liquid_balance: Uint128 = match query::balance(&deps, &asset)? {
        QueryAnswer::Balance { amount } => amount,
        _ => {
            return Err(StdError::generic_err("Unexpected response for balance"));
       }
    };

    let alloc_portion = match &alloc {
        Allocation::Reserves { allocation } => *allocation,

        // TODO: Needs to be accounted for elsewhere
        Allocation::Allowance { address: _, amount: _ } => Uint128::zero(),

        Allocation::Rewards {
            contract: _,
            allocation,
        } => *allocation,
        Allocation::Staking {
            contract: _,
            allocation,
        } => *allocation,
        Allocation::Application {
            contract: _,
            allocation,
            token: _,
        } => *allocation,
        Allocation::Pool {
            contract: _,
            allocation,
            secondary_asset: _,
            token: _,
        } => *allocation,
    };

    let alloc_address = match &alloc {
        Allocation::Allowance { address, amount: _ } => Some(address.clone()),
        Allocation::Staking {
            contract,
            allocation: _,
        } => Some(contract.address.clone()),
        Allocation::Application {
            contract,
            allocation: _,
            token: _,
        } => Some(contract.address.clone()),
        Allocation::Pool {
            contract,
            allocation: _,
            secondary_asset: _,
            token: _,
        } => Some(contract.address.clone()),
        _ => None,
    };

    let mut allocated_portion = Uint128::zero();

    allocations_w(&mut deps.storage).update(asset.to_string().as_bytes(), |apps| {
        // Initialize list if it doesn't exist
        let mut app_list = match apps {
            None => {
                vec![]
            }
            Some(a) => a,
        };

        // Remove old instance of this contract
        let mut existing_index = None;
        for (i, app) in app_list.iter_mut().enumerate() {
            if let Some(address) = match app {
                Allocation::Rewards {
                    contract,
                    allocation: _,
                } => Some(contract.address.clone()),
                Allocation::Staking {
                    contract,
                    allocation: _,
                } => Some(contract.address.clone()),
                Allocation::Application {
                    contract,
                    allocation: _,
                    token: _,
                } => Some(contract.address.clone()),
                Allocation::Pool {
                    contract,
                    allocation: _,
                    secondary_asset: _,
                    token: _,
                } => Some(contract.address.clone()),
                _ => None,
            } {
                match &alloc_address {
                    Some(a) => {
                        // Found the address, mark index and break from scan loop
                        if address == *a {
                            existing_index = Option::from(i);
                            break;
                        }
                    }
                    None => {}
                }
            } else {
                /*
                 * I think this is not needed, must have been a late night
                match alloc_address {
                    Some(_) => {}
                    None => {
                        existing_index = Option::from(i);
                        break;
                    }
                }
                */
            }
        }

        // If an element was marked, remove it from the list
        match existing_index {
            Some(i) => {
                app_list.remove(i);
            }
            _ => {}
        }

        // Validate addition does not exceed 100%
        for app in &app_list {

            allocated_portion = allocated_portion + match app {
                Allocation::Rewards {
                    contract: _,
                    allocation: _,
                } => Uint128::zero(),
                Allocation::Staking {
                    contract: _,
                    allocation,
                } => *allocation,
                Allocation::Application {
                    contract: _,
                    allocation,
                    token: _,
                } => *allocation,
                Allocation::Pool {
                    contract: _,
                    allocation,
                    secondary_asset: _,
                    token: _,
                } => *allocation,
                _ => Uint128::zero(),
            };
        }

        if (allocated_portion + alloc_portion) >= Uint128(10u128.pow(18)) {
            return Err(StdError::generic_err("Invalid allocation total exceeding 100%"));
        }

        app_list.push(alloc);

        Ok(app_list)
    })?;

    /*TODO: Need to re-allocate/re-balance funds based on the new addition
     * get Uint128 math functions to do these things (untested)
     * re-add send_msg below
     */
    
    /*
    let liquid_portion = (allocated_portion * liquid_balance) / allocated_portion;

    // Determine how much of current balance is to be allocated
    let to_allocate = liquid_balance - (alloc_portion / liquid_portion);
    */

    Ok(HandleResponse {
        messages: vec![
            /*
            send_msg(
                    alloc_address,
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
        data: Some(to_binary(&HandleAnswer::RegisterApp {
            status: ResponseStatus::Success,
        })?),
    })
}

