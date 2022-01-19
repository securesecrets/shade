use cosmwasm_std;
use cosmwasm_std::{
    from_binary, to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr, Querier, StdError,
    StdResult, Storage, Uint128,
};
use secret_toolkit::snip20::{register_receive_msg, send_msg, set_viewing_key_msg};

use shade_protocol::{
    asset::Contract,
    generic_response::ResponseStatus,
    //math::Uint128,
    snip20::fetch_snip20,
    treasury::{Allocation, Config, Flag, HandleAnswer, QueryAnswer},
};

use crate::{
    query,
    state::{allocations_w, assets_r, assets_w, config_r, config_w, reserves_w, viewing_key_r},
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

    assets_w(&mut deps.storage).save(
        contract.address.to_string().as_bytes(),
        &fetch_snip20(&contract, &deps.querier)?,
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

    reserves_w(&mut deps.storage).save(
        contract.address.to_string().as_bytes(),
        &match reserves {
            None => Uint128::zero(),
            Some(r) => r,
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

    let alloc_portion = *match &alloc {
        Allocation::Reserves { allocation } => allocation,
        Allocation::Rewards {
            contract: _,
            allocation,
        } => allocation,
        Allocation::Staking {
            contract: _,
            allocation,
        } => allocation,
        Allocation::Application {
            contract: _,
            allocation,
            token: _,
        } => allocation,
        Allocation::Pool {
            contract: _,
            allocation,
            secondary_asset: _,
            token: _,
        } => allocation,
    };

    let alloc_address = match &alloc {
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
        // TODO: need type comparison or something? gonna worry about it later
        let mut existing_index = None;
        for (i, app) in app_list.iter_mut().enumerate() {
            if let Some(address) = match app {
                Allocation::Reserves { allocation: _ } => None,
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
            } {
                match &alloc_address {
                    Some(a) => {
                        if address == *a {
                            existing_index = Option::from(i);
                            break;
                        }
                    }
                    None => {}
                }
            } else {
                match alloc_address {
                    Some(_) => {}
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
            allocated_portion = allocated_portion
                + match app {
                    Allocation::Reserves { allocation: _ } => Uint128::zero(),
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
                };
        }

        if (allocated_portion + alloc_portion) >= Uint128(10u128.pow(18)) {
            return Err(StdError::generic_err("Invalid allocation total exceeding 100%"));
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
        data: Some(to_binary(&HandleAnswer::RegisterApp {
            status: ResponseStatus::Success,
        })?),
    })
}
