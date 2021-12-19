use cosmwasm_std::{to_binary, Api, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage, HumanAddr, Uint128, Binary, Decimal};
use rs_merkle::{Hasher, MerkleProof, algorithms::Sha256};
use crate::state::{
    config_r, config_w, claim_status_w, claim_status_r, account_total_claimed_w,
    total_claimed_w, total_claimed_r, account_r, address_in_account_w, account_w, validate_permit,
    revoke_permit
};
use shade_protocol::{airdrop::{HandleAnswer, Config, claim_info::{RequiredTask, Reward},
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
    decay_start: Option<u64>
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
            // Avoid date collisions
            if let Some(end_date) = end_date {
                if start_date > end_date {
                    return Err(StdError::generic_err("New start date is greater than end date"))
                }
            }
            else if let Some(end_date) = state.end_date {
                if start_date > end_date {
                    return Err(StdError::generic_err("New start date is greater than the current end date"))
                }
            }

            state.start_date = start_date;
        }
        if let Some(end_date) = end_date {
            // Avoid date collisions
            if state.start_date > end_date {
                return Err(StdError::generic_err("New end date is before start date"))
            }
            if let Some(decay_start) = decay_start {
                if decay_start > end_date {
                    return Err(StdError::generic_err("New end date is before start of decay"))
                }
            }
            else if let Some(decay_start) = state.decay_start {
                if decay_start > end_date {
                    return Err(StdError::generic_err("New end date is before current start of decay"))
                }
            }

            state.end_date = Some(end_date);
        }
        if let Some(decay_start) = decay_start {
            // Avoid date collisions
            if let Some(end_date) = state.end_date {
                if decay_start > end_date {
                    return Err(StdError::generic_err("New start of decay is after end date"))
                }
            }
            else {
                return Err(StdError::generic_err("No end date set"))
            }

            state.decay_start = Some(decay_start)
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
    partial_tree: Vec<Binary>,
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

    // Validate permits
    validate_address_permits(&mut deps.storage, &config, &env.message.sender, &mut account, addresses, partial_tree)?;

    // Save account
    account_w(&mut deps.storage).save(sender.as_bytes(), &account)?;

    // Add default claim at index 0
    account_total_claimed_w(&mut deps.storage).save(sender.as_bytes(), &Uint128::zero())?;
    claim_status_w(&mut deps.storage, 0).save(sender.as_bytes(), &false)?;

    // Claim the airdrop after account creation
    //TODO: replace with claim function to avoid double checking airdrop config
    try_claim(deps, env);

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
    partial_tree: Vec<Binary>,
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
    validate_address_permits(&mut deps.storage, &config, &env.message.sender, &mut account, addresses, partial_tree)?;

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
                let send_total = (config.airdrop_amount - total_claimed_r(&deps.storage).load()?)?;
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

    // Calculate redeem amount after applying decay
    if let Some(decay_start) = config.decay_start {
        if env.block.time >= decay_start {
            redeem_amount = redeem_amount * calculate_decay_factor(decay_start,
                                                                   env.block.time,
                                                                   config.end_date.unwrap());
        }
    }

    total_claimed_w(storage).update(|claimed| {
        Ok(claimed + redeem_amount)
    })?;

    Ok((redeem_amount, completed_percentage))
}

/// Validates all of the information and updates relevant states
pub fn validate_address_permits<S: Storage>(
    storage: &mut S,
    config: &Config,
    sender: &HumanAddr,
    account: &mut Account,
    addresses: Vec<AddressProofPermit>,
    partial_tree: Vec<Binary>,
) -> StdResult<()> {
    // Setup the items to validate
    let mut leafs_to_validate: Vec<(usize, [u8; 32])> = vec![];

    // Iterate addresses
    for permit in addresses.iter() {
        // Check permit legitimacy / skip if permit is sender
        let address: HumanAddr;
        if &permit.params.address != sender {
            address = validate_permit(storage, permit, config.contract.clone())?;
            if address != permit.params.address {
                return Err(StdError::generic_err("Signer address is not the same as the permit address"))
            }
        }
        else {
            address = sender.clone();
        }

        // Check that airdrop amount does not exceed maximum
        if permit.params.amount > config.max_amount {
            return Err(StdError::generic_err("Amount exceeds maximum amount"))
        }

        // Check that address has not been added to an account
        address_in_account_w(storage).update(address.to_string().as_bytes(), |state| {
            if state.is_some() {
                return Err(StdError::generic_err(
                    format!("{:?} already in an account", address.to_string())))
            }

            Ok(true)
        })?;

        // Add account as a leaf
        let leaf_hash = Sha256::hash((address.to_string() + &permit.params.amount.to_string()).as_bytes());
        leafs_to_validate.push((permit.params.index as usize, leaf_hash));

        // If valid then add to account array and sum total amount
        account.addresses.push(address);
        account.total_claimable += permit.params.amount;
    }

    // Need to sort by index in order for the proof to work
    leafs_to_validate.sort_by_key(|item| item.0);

    let mut indices: Vec<usize> = vec![];
    let mut leaves: Vec<[u8; 32]> = vec![];

    for leaf in leafs_to_validate.iter() {
        indices.push(leaf.0);
        leaves.push(leaf.1);
    };

    // Convert partial tree from base64 to binary
    let mut partial_tree_binary: Vec<[u8; 32]> = vec![];
    for node in partial_tree.iter() {
        let mut arr: [u8; 32] = Default::default();
        arr.clone_from_slice(node.as_slice());
        partial_tree_binary.push(arr);
    }

    // Prove that user is in airdrop
    let proof = MerkleProof::<Sha256>::new(partial_tree_binary);
    // Convert to a fixed length array without messing up the contract
    let mut root: [u8; 32] = Default::default();
    root.clone_from_slice(config.merkle_root.as_slice());
    if !proof.verify(root, &indices, &leaves, config.total_accounts as usize) {
        return Err(StdError::generic_err("Invalid proof"))
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

pub fn calculate_decay_factor(min: u64, x: u64, max: u64) -> Decimal {
    // Get the inverse normalized value [0,1] of x between [min, max]
    Decimal::from_ratio(max - x, max - min)
}