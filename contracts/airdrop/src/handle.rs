use cosmwasm_std::{to_binary, Api, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage, HumanAddr, Uint128};
use crate::state::{config_r, config_w, airdrop_address_r, claim_status_w, claim_status_r,
                   account_total_claimed_w, total_claimed_w, total_claimed_r, account_r,
                   address_in_account_w, account_w, validate_permit, revoke_permit};
use shade_protocol::{airdrop::{HandleAnswer, Config, claim_info::RequiredTask,
                               account::{Account, AddressProofPermit}},
                     generic_response::ResponseStatus};
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

pub fn try_create_account<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    addresses: Vec<AddressProofPermit>,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    // Check that airdrop hasnt ended
    if !available(&config, env) {
        return Err(StdError::unauthorized())
    }

    // Check that account doesnt exist
    let sender = env.message.sender.to_string();
    if account_r(&deps.storage).may_load(sender.as_bytes())?.is_some() {
        return Err(StdError::generic_err("Account already made"))
    }

    let mut account = Account {
        addresses: vec![],
        total_claimable: Uint128::zero()
    };

    // Try to add sender account
    match  airdrop_address_r(&deps.storage).may_load(sender.as_bytes())? {
        None => {}
        Some(airdrop) => {
            address_in_account_w(&mut deps.storage).update(sender.as_bytes(), |state| {
                let in_account = state.unwrap();

                // Check that address has not been added to an account
                if in_account {
                    return Err(StdError::generic_err(
                        format!("{:?} already in an account", sender.clone())))
                }

                Ok(true)
            })?;
            account.addresses.push(airdrop.address);
            account.total_claimable += airdrop.amount;
        }
    }

    // Validate permits
    validate_address_permits(&mut deps.storage, &mut account, addresses)?;

    // Save account
    account_w(&mut deps.storage).save(sender.as_bytes(), &account)?;

    // Add default claim at index 0
    account_total_claimed_w(&mut deps.storage).save(sender.as_bytes(), &Uint128::zero())?;
    claim_status_w(&mut deps.storage, 0).save(sender.as_bytes(), &false)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::CreateAccount {
            status: ResponseStatus::Success } )? )
    })
}

pub fn try_update_account<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    addresses: Vec<AddressProofPermit>,
) -> StdResult<HandleResponse> {

    // Check if airdrop active
    let config = config_r(&deps.storage).load()?;

    // Check that airdrop hasnt ended
    if !available(&config, env) {
        return Err(StdError::unauthorized())
    }

    // Get account
    let sender = env.message.sender.clone().to_string();
    let mut account = account_r(&deps.storage).load(sender.as_bytes())?;

    // Run the claim function
    let old_claim_amount = account.total_claimable;
    let (redeem_amount, completed_percentage) = claim_tokens(&mut deps.storage, env, &config, &account)?;

    // Setup the new addresses
    validate_address_permits(&mut deps.storage, &mut account, addresses)?;

    // Calculate the total new address amount
    let added_address_total = (account.total_claimable - old_claim_amount)?;
    let mut new_redeem = Uint128::zero();
    account_total_claimed_w(&mut deps.storage).update(sender.as_bytes(), |claimed| {
        if completed_percentage == Uint128(100) {
            new_redeem += added_address_total;
            Ok(account.total_claimable)
        }
        else {
            new_redeem += completed_percentage.multiply_ratio(
                added_address_total, Uint128(100));
            Ok(claimed.unwrap() + new_redeem)
        }
    })?;

    total_claimed_w(&mut deps.storage).update(|claimed| {
        Ok(claimed + new_redeem)
    })?;

    account_w(&mut deps.storage).save(sender.as_bytes(), &account)?;

    Ok(HandleResponse {
        messages: vec![send_msg(env.message.sender.clone(), redeem_amount+new_redeem,
                                None, None, 0,
                                config.airdrop_snip20.code_hash,
                                config.airdrop_snip20.address)?],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::UpdateAccount {
            status: ResponseStatus::Success } )? )
    })
}

pub fn try_disable_permit_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    key: String,
) -> StdResult<HandleResponse> {

    revoke_permit(&mut deps.storage, env.message.sender.to_string(), key);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::DisablePermitKey {
            status: ResponseStatus::Success } )? )
    })
}


pub fn try_complete_task<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    account: HumanAddr,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    for (i, task) in config.task_claim.iter().enumerate() {
        if task.address == env.message.sender {
            claim_status_w(&mut deps.storage, i).update(
                account.to_string().as_bytes(), |status| {
                    // If there was a state then ignore
                    if let Some(status) = status {
                        Ok(status)
                    }
                    else {
                        Ok(false)
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

    // Check that airdrop hasnt ended
    if !available(&config, &env) {
        return Err(StdError::unauthorized())
    }

    // Get account
    let sender = env.message.sender.clone();
    let account = account_r(&deps.storage).load(sender.to_string().as_bytes())?;

    // Calculate airdrop
    let (redeem_amount, _) = claim_tokens(&mut deps.storage, env, &config, &account)?;

    Ok(HandleResponse {
        messages: vec![send_msg(sender, redeem_amount,
                                None, None, 0,
                                config.airdrop_snip20.code_hash,
                                config.airdrop_snip20.address)?],
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

pub fn claim_tokens<S: Storage>(
    storage: &mut S,
    env: &Env,
    config: &Config,
    account: &Account,
) -> StdResult<(Uint128, Uint128)> { // (send_amount, completed_percentage)
    let sender = env.message.sender.to_string();

    // Calculate eligible tasks
    let mut completed_percentage = Uint128::zero();
    let mut unclaimed_percentage = Uint128::zero();
    for (index, task) in config.task_claim.iter().enumerate() {
        // Check if task has been completed
        let state = claim_status_r(storage, index).may_load(
            sender.as_bytes())?;

        match state {
            // Ignore if none
            None => {}
            Some(claimed) => {
                completed_percentage += task.percent;
                if !claimed {
                    // Set claim status to true since we're going to claim it now
                    claim_status_w(storage, index).save(
                        sender.as_bytes(), &true)?;

                    unclaimed_percentage += task.percent;
                }
            }
        }
    }

    if unclaimed_percentage == Uint128::zero() {
        return Err(StdError::generic_err("No claimable amount available"))
    }

    // Amount to be redeemed
    let mut redeem_amount = Uint128::zero();

    // Update total claimed and calculate claimable
    account_total_claimed_w(storage).update(sender.as_bytes(), |claimed| {
        // This solves possible uToken inaccuracies
        if completed_percentage == Uint128(100) {
            redeem_amount = (account.total_claimable - claimed.unwrap())?;
            Ok(account.total_claimable)
        }
        else {
            redeem_amount = unclaimed_percentage.multiply_ratio(account.total_claimable, Uint128(100));
            Ok(claimed.unwrap() + redeem_amount)
        }
    })?;

    total_claimed_w(storage).update(|claimed| {
        Ok(claimed + redeem_amount)
    })?;

    Ok((redeem_amount, completed_percentage))
}

pub fn validate_address_permits<S: Storage>(
    storage: &mut S,
    account: &mut Account,
    addresses: Vec<AddressProofPermit>
) -> StdResult<()> {
    // Iterate addresses
    for permit in addresses.iter() {
        // Check that permit is available
        let address = validate_permit(storage, permit)?;

        address_in_account_w(storage).update(address.to_string().as_bytes(), |state| {
            let in_account = state.unwrap();

            // Check that address has not been added to an account
            if in_account {
                return Err(StdError::generic_err(
                    format!("{:?} already in an account", address.to_string())))
            }

            Ok(true)
        })?;

        // If valid then add to account array and sum total amount
        let airdrop = airdrop_address_r(storage).load(address.to_string().as_bytes())?;
        account.addresses.push(address);
        account.total_claimable += airdrop.amount;
    }

    Ok(())
}

pub fn available( config: &Config, env: &Env ) -> bool {
    let current_time = env.block.time;

    // Check if airdrop started
    if current_time < config.start_date {
        return false
    }
    if let Some(end_date) = config.end_date {
        if current_time > end_date {
            return false
        }
    }

    true
}