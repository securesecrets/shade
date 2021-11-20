use cosmwasm_std::{to_binary, Api, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage, HumanAddr, Uint128};
use crate::state::{config_r, config_w, reward_r, claim_status_w, claim_status_r, user_total_claimed_w, total_claimed_w, total_claimed_r};
use shade_protocol::airdrop::{HandleAnswer, RequiredTask};
use shade_protocol::generic_response::ResponseStatus;
use secret_toolkit::snip20::send_msg;

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    admin: Option<HumanAddr>,
    dump_address: Option<HumanAddr>,
    start_date: Option<u64>,
    end_date: Option<u64>,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;
    // Check if admin
    if env.message.sender != config.admin {
        return Err(StdError::unauthorized());
    }

    // Save new info
    let mut config = config_w(&mut deps.storage);
    config.update(|mut state| {
        if let Some(admin) = admin {
            state.admin = admin;
        }
        if let Some(dump_address)= dump_address {
            state.dump_address = Some(dump_address);
        }
        if let Some(start_date) = start_date {
            state.start_date = start_date;
        }
        if let Some(end_date) = end_date {
            state.end_date = Some(end_date);
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

pub fn try_add_tasks<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    tasks: Vec<RequiredTask>
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    // Check if admin
    if env.message.sender != config.admin {
        return Err(StdError::unauthorized());
    }

    config_w(&mut deps.storage).update(|mut config| {
        let mut task_list = tasks;
        config.task_claim.append(&mut task_list);

        //Validate that they do not excede 100
        let mut count = Uint128::zero();
        for task in config.task_claim.iter() {
            count += task.percent;
        }

        if count > Uint128(100) {
            return Err(StdError::generic_err("tasks above 100%"))
        }

        Ok(config)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::AddTask {
            status: ResponseStatus::Success } )? )
    })
}

pub fn try_complete_task<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    address: HumanAddr,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    for (i, item) in config.task_claim.iter().enumerate() {
        if item.address == env.message.sender {
            claim_status_w(&mut deps.storage, i).update(
                address.to_string().as_bytes(), |status| {
                    // If there was a state then ignore
                    if status.is_none() {
                        Ok(false)
                    }
                    else {
                        Err(StdError::unauthorized())
                    }
                })?;

            return Ok(HandleResponse {
                messages: vec![],
                log: vec![],
                data: Some( to_binary( &HandleAnswer::Claim {
                    status: ResponseStatus::Success } )? )
            })
        }
    }

    // if not found
    Err(StdError::not_found("task"))
}

pub fn try_claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    // Check if airdrop started
    if env.block.time < config.start_date {
        return Err(StdError::unauthorized())
    }
    if let Some(end_date) = config.end_date {
        if env.block.time > end_date {
            return Err(StdError::unauthorized())
        }
    }

    let user = env.message.sender.clone();
    let user_key = user.to_string();

    let eligible_amount = reward_r(&deps.storage).load(user_key.as_bytes())?.amount;

    let mut claimed_percentage = Uint128::zero();
    let mut total = Uint128::zero();
    for (i, task) in config.task_claim.iter().enumerate() {
        // Check if completed
        let state = claim_status_r(&deps.storage, i).may_load(
            user_key.as_bytes())?;
        match state {
            None => {}
            Some(claimed) => {
                claimed_percentage += task.percent;
                if !claimed {
                    claim_status_w(&mut deps.storage, i).save(
                        user_key.as_bytes(), &true)?;
                    total += task.percent.multiply_ratio(
                        eligible_amount, Uint128(100));
                }
            }
        };
    }

    // Update redeem info
    let mut redeem_amount = total;
    user_total_claimed_w(&mut deps.storage).update(
        user_key.as_bytes(), |total_claimed| {
            // Fix division issues
            if claimed_percentage == Uint128(100) {
                redeem_amount = (eligible_amount - total_claimed.unwrap())?;
            }
            Ok(total_claimed.unwrap() + redeem_amount)
        })?;

    total_claimed_w(&mut deps.storage).update(|total_claimed| {
        Ok(total_claimed + redeem_amount)
    })?;

    // Redeem
    let messages =  vec![send_msg(user, redeem_amount,
                                  None, None, 0,
                                  config.airdrop_snip20.code_hash,
                                  config.airdrop_snip20.address)?];

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some( to_binary( &HandleAnswer::Claim {
            status: ResponseStatus::Success } )? )
    })
}

pub fn try_decay<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    // Check if airdrop ended
    if let Some(end_date) = config.end_date {
        if let Some(dump_address) = config.dump_address {
            if env.block.time > end_date {
                let send_total = (config.airdrop_total - total_claimed_r(&deps.storage).load()?)?;
                let messages = vec![send_msg(
                    dump_address, send_total, None, None,
                    1, config.airdrop_snip20.code_hash,
                    config.airdrop_snip20.address)?];

                return Ok(HandleResponse {
                    messages,
                    log: vec![],
                    data: Some( to_binary( &HandleAnswer::Decay {
                        status: ResponseStatus::Success } )? )
                })
            }
        }
    }

    Err(StdError::unauthorized())
}