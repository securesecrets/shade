use crate::state::{
    account_r, account_total_claimed_w, account_w, address_in_account_w, claim_status_r,
    claim_status_w, config_r, config_w, decay_claimed_w, revoke_permit, total_claimed_r,
    total_claimed_w, validate_address_permit,
};
use cosmwasm_std::{to_binary, Api, Binary, Decimal, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, Uint128, from_binary};
use rs_merkle::{algorithms::Sha256, Hasher, MerkleProof};
use secret_toolkit::snip20::send_msg;
use shade_protocol::airdrop::{
    account::{Account, AddressProofPermit},
    claim_info::RequiredTask,
    Config, HandleAnswer,
};
use shade_protocol::airdrop::account::AddressProofMsg;
use shade_protocol::utils::generic_response::ResponseStatus;

#[allow(clippy::too_many_arguments)]
pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    admin: Option<HumanAddr>,
    dump_address: Option<HumanAddr>,
    query_rounding: Option<Uint128>,
    start_date: Option<u64>,
    end_date: Option<u64>,
    decay_start: Option<u64>,
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
        if let Some(dump_address) = dump_address {
            state.dump_address = Some(dump_address);
        }
        if let Some(query_rounding) = query_rounding {
            state.query_rounding = query_rounding;
        }
        if let Some(start_date) = start_date {
            // Avoid date collisions
            if let Some(end_date) = end_date {
                if start_date > end_date {
                    return Err(StdError::generic_err(
                        "New start date is greater than end date",
                    ));
                }
            } else if let Some(end_date) = state.end_date {
                if start_date > end_date {
                    return Err(StdError::generic_err(
                        "New start date is greater than the current end date",
                    ));
                }
            }
            if let Some(start_decay) = decay_start {
                if start_date > start_decay {
                    return Err(StdError::generic_err(
                        "New start date is greater than start of decay",
                    ));
                }
            } else if let Some(start_decay) = state.decay_start {
                if start_date > start_decay {
                    return Err(StdError::generic_err(
                        "New start date is greater than the current start of decay",
                    ));
                }
            }

            state.start_date = start_date;
        }
        if let Some(end_date) = end_date {
            // Avoid date collisions
            if let Some(decay_start) = decay_start {
                if decay_start > end_date {
                    return Err(StdError::generic_err(
                        "New end date is before start of decay",
                    ));
                }
            } else if let Some(decay_start) = state.decay_start {
                if decay_start > end_date {
                    return Err(StdError::generic_err(
                        "New end date is before current start of decay",
                    ));
                }
            }

            state.end_date = Some(end_date);
        }
        if decay_start.is_some() {
            state.decay_start = decay_start
        }

        Ok(state)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_add_tasks<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    tasks: Vec<RequiredTask>,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;
    // Check if admin
    if env.message.sender != config.admin {
        return Err(StdError::unauthorized());
    }

    config_w(&mut deps.storage).update(|mut config| {
        let mut task_list = tasks;
        config.task_claim.append(&mut task_list);

        //Validate that they do not exceed 100
        let mut count = Uint128::zero();
        for task in config.task_claim.iter() {
            count += task.percent;
        }

        if count > Uint128(100) {
            return Err(StdError::generic_err("tasks above 100%"));
        }

        Ok(config)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddTask {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_create_account<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    addresses: Vec<AddressProofPermit>,
    partial_tree: Vec<Binary>,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    // Check that airdrop hasn't ended
    available(&config, env)?;

    // Check that account doesnt exist
    let sender = env.message.sender.to_string();
    if account_r(&deps.storage)
        .may_load(sender.as_bytes())?
        .is_some()
    {
        return Err(StdError::generic_err("Account already made"));
    }

    let mut account = Account {
        addresses: vec![],
        total_claimable: Uint128::zero(),
    };

    // Validate permits
    try_add_account_addresses(
        &mut deps.storage,
        &config,
        &env.message.sender,
        &mut account,
        addresses,
        partial_tree,
    )?;

    // Save account
    account_w(&mut deps.storage).save(sender.as_bytes(), &account)?;

    // Add default claim at index 0
    account_total_claimed_w(&mut deps.storage).save(sender.as_bytes(), &Uint128::zero())?;
    claim_status_w(&mut deps.storage, 0).save(sender.as_bytes(), &false)?;

    // Claim the airdrop after account creation
    let (completed_percentage, unclaimed_percentage) =
        update_tasks(&mut deps.storage, &config, sender)?;
    let mut messages = vec![];
    // Avoid calculating if theres nothing to claim
    if unclaimed_percentage > Uint128::zero() {
        let redeem_amount = claim_tokens(
            &mut deps.storage,
            env,
            &config,
            &account,
            completed_percentage,
            unclaimed_percentage,
        )?;

        total_claimed_w(&mut deps.storage).update(|claimed| Ok(claimed + redeem_amount))?;

        messages.push(send_msg(
            env.message.sender.clone(),
            redeem_amount,
            None,
            None,
            None,
            0,
            config.airdrop_snip20.code_hash,
            config.airdrop_snip20.address,
        )?);
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CreateAccount {
            status: ResponseStatus::Success,
        })?),
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
    available(&config, env)?;

    // Get account
    let sender = env.message.sender.clone().to_string();
    let mut account = account_r(&deps.storage).load(sender.as_bytes())?;

    // Run the claim function if theres something to claim
    let old_claim_amount = account.total_claimable;
    let (completed_percentage, unclaimed_percentage) =
        update_tasks(&mut deps.storage, &config, sender.to_string())?;

    let mut redeem_amount = Uint128::zero();

    if unclaimed_percentage > Uint128::zero() {
        redeem_amount = claim_tokens(
            &mut deps.storage,
            env,
            &config,
            &account,
            completed_percentage,
            unclaimed_percentage,
        )?;
    }

    // Setup the new addresses
    try_add_account_addresses(
        &mut deps.storage,
        &config,
        &env.message.sender,
        &mut account,
        addresses,
        partial_tree,
    )?;

    let mut messages = vec![];
    if completed_percentage > Uint128::zero() {
        // Calculate the total new address amount
        let added_address_total = (account.total_claimable - old_claim_amount)?;
        account_total_claimed_w(&mut deps.storage).update(sender.as_bytes(), |claimed| {
            if let Some(claimed) = claimed {
                let new_redeem: Uint128;
                if completed_percentage == Uint128(100) {
                    new_redeem = added_address_total * decay_factor(env.block.time, &config);
                } else {
                    new_redeem = completed_percentage
                        .multiply_ratio(added_address_total, Uint128(100))
                        * decay_factor(env.block.time, &config);
                }

                redeem_amount += new_redeem;
                Ok(claimed + new_redeem)
            } else {
                Err(StdError::generic_err("Account total claimed not set"))
            }
        })?;

        total_claimed_w(&mut deps.storage).update(|claimed| Ok(claimed + redeem_amount))?;

        messages.push(send_msg(
            env.message.sender.clone(),
            redeem_amount,
            None,
            None,
            None,
            0,
            config.airdrop_snip20.code_hash,
            config.airdrop_snip20.address,
        )?);
    }

    account_w(&mut deps.storage).save(sender.as_bytes(), &account)?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateAccount {
            status: ResponseStatus::Success,
        })?),
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
        data: Some(to_binary(&HandleAnswer::DisablePermitKey {
            status: ResponseStatus::Success,
        })?),
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
                account.to_string().as_bytes(),
                |status| {
                    // If there was a state then ignore
                    if let Some(status) = status {
                        Ok(status)
                    } else {
                        Ok(false)
                    }
                },
            )?;
        }
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Claim {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    // Check that airdrop hasn't ended
    available(&config, env)?;

    // Get account
    let sender = env.message.sender.clone();
    let account = account_r(&deps.storage).load(sender.to_string().as_bytes())?;

    // Calculate airdrop
    let (completed_percentage, unclaimed_percentage) =
        update_tasks(&mut deps.storage, &config, sender.to_string())?;

    if unclaimed_percentage == Uint128::zero() {
        return Err(StdError::generic_err("No claimable amount available"));
    }

    let redeem_amount = claim_tokens(
        &mut deps.storage,
        env,
        &config,
        &account,
        completed_percentage,
        unclaimed_percentage,
    )?;

    total_claimed_w(&mut deps.storage).update(|claimed| Ok(claimed + redeem_amount))?;

    Ok(HandleResponse {
        messages: vec![send_msg(
            sender,
            redeem_amount,
            None,
            None,
            None,
            0,
            config.airdrop_snip20.code_hash,
            config.airdrop_snip20.address,
        )?],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Claim {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_claim_decay<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    // Check if airdrop ended
    if let Some(end_date) = config.end_date {
        if let Some(dump_address) = config.dump_address {
            if env.block.time > end_date {
                decay_claimed_w(&mut deps.storage).update(|claimed| {
                    if claimed {
                        Err(StdError::generic_err("Decay already claimed"))
                    } else {
                        Ok(true)
                    }
                })?;

                let send_total =
                    (config.airdrop_amount - total_claimed_r(&deps.storage).load()?)?;
                let messages = vec![send_msg(
                    dump_address,
                    send_total,
                    None,
                    None,
                    None,
                    1,
                    config.airdrop_snip20.code_hash,
                    config.airdrop_snip20.address,
                )?];

                return Ok(HandleResponse {
                    messages,
                    log: vec![],
                    data: Some(to_binary(&HandleAnswer::ClaimDecay {
                        status: ResponseStatus::Success,
                    })?),
                });
            }
        }
    }

    Err(StdError::unauthorized())
}

/// Gets task information and sets them
pub fn update_tasks<S: Storage>(
    storage: &mut S,
    config: &Config,
    sender: String,
) -> StdResult<(Uint128, Uint128)> {
    // Calculate eligible tasks
    let mut completed_percentage = Uint128::zero();
    let mut unclaimed_percentage = Uint128::zero();
    for (index, task) in config.task_claim.iter().enumerate() {
        // Check if task has been completed
        let state = claim_status_r(storage, index).may_load(sender.as_bytes())?;

        match state {
            // Ignore if none
            None => {}
            Some(claimed) => {
                completed_percentage += task.percent;
                if !claimed {
                    // Set claim status to true since we're going to claim it now
                    claim_status_w(storage, index).save(sender.as_bytes(), &true)?;

                    unclaimed_percentage += task.percent;
                }
            }
        }
    }

    Ok((completed_percentage, unclaimed_percentage))
}

pub fn claim_tokens<S: Storage>(
    storage: &mut S,
    env: &Env,
    config: &Config,
    account: &Account,
    completed_percentage: Uint128,
    unclaimed_percentage: Uint128,
) -> StdResult<Uint128> {
    // send_amount
    let sender = env.message.sender.to_string();

    // Amount to be redeemed
    let mut redeem_amount = Uint128::zero();

    // Update total claimed and calculate claimable
    account_total_claimed_w(storage).update(sender.as_bytes(), |claimed| {
        if let Some(claimed) = claimed {
            // This solves possible uToken inaccuracies
            if completed_percentage == Uint128(100) {
                redeem_amount = (account.total_claimable - claimed)?;
            } else {
                redeem_amount =
                    unclaimed_percentage.multiply_ratio(account.total_claimable, Uint128(100));
            }

            // Update redeem amount with the decay multiplier
            redeem_amount = redeem_amount * decay_factor(env.block.time, config);

            Ok(claimed + redeem_amount)
        } else {
            Err(StdError::generic_err("Account total claimed not set"))
        }
    })?;

    Ok(redeem_amount)
}

/// Validates all of the information and updates relevant states
pub fn try_add_account_addresses<S: Storage>(
    storage: &mut S,
    config: &Config,
    sender: &HumanAddr,
    account: &mut Account,
    addresses: Vec<AddressProofPermit>,
    partial_tree: Vec<Binary>,
) -> StdResult<()> {
    // Setup the items to validate
    let mut leaves_to_validate: Vec<(usize, [u8; 32])> = vec![];

    // Iterate addresses
    for permit in addresses.iter() {
        if let Some(memo) = permit.memo.clone() {
            let params: AddressProofMsg = from_binary(&Binary::from_base64(&memo)?)?;

            let address: HumanAddr;
            // Avoid verifying sender
            if &params.address != sender {
                // Check permit legitimacy
                address = validate_address_permit(storage, permit, config.contract.clone())?;
                if address != params.address {
                    return Err(StdError::generic_err(
                        "Signer address is not the same as the permit address",
                    ));
                }
            } else {
                address = sender.clone();
            }

            // Check that airdrop amount does not exceed maximum
            if params.amount > config.max_amount {
                return Err(StdError::generic_err("Amount exceeds maximum amount"));
            }

            // Update address if its not in an account
            address_in_account_w(storage).update(address.to_string().as_bytes(), |state| {
                if state.is_some() {
                    return Err(StdError::generic_err(format!(
                        "{:?} already in an account",
                        address.to_string()
                    )));
                }

                Ok(true)
            })?;

            // Add account as a leaf
            let leaf_hash =
                Sha256::hash((address.to_string() + &params.amount.to_string()).as_bytes());
            leaves_to_validate.push((params.index as usize, leaf_hash));

            // If valid then add to account array and sum total amount
            account.addresses.push(address);
            account.total_claimable += params.amount;
        }
        else {
            return Err(StdError::generic_err(format!("Expected a memo")))
        }
    }

    // Need to sort by index in order for the proof to work
    leaves_to_validate.sort_by_key(|item| item.0);

    let mut indices: Vec<usize> = vec![];
    let mut leaves: Vec<[u8; 32]> = vec![];

    for leaf in leaves_to_validate.iter() {
        indices.push(leaf.0);
        leaves.push(leaf.1);
    }

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
        return Err(StdError::generic_err("Invalid proof"));
    }

    Ok(())
}

pub fn available(config: &Config, env: &Env) -> StdResult<()> {
    let current_time = env.block.time;

    // Check if airdrop started
    if current_time < config.start_date {
        return Err(StdError::generic_err(format!(
            "Airdrop starts on {}",
            config.start_date
        )));
    }
    if let Some(end_date) = config.end_date {
        if current_time > end_date {
            return Err(StdError::generic_err(format!(
                "Airdrop ended on {}",
                end_date
            )));
        }
    }

    Ok(())
}

/// Get the multiplier for decay, will return 1 when decay isnt in effect.
pub fn decay_factor(current_time: u64, config: &Config) -> Decimal {
    // Calculate redeem amount after applying decay
    if let Some(decay_start) = config.decay_start {
        if current_time >= decay_start {
            return inverse_normalizer(decay_start, current_time, config.end_date.unwrap());
        }
    }
    Decimal::one()
}

/// Get the inverse normalized value [0,1] of x between [min, max]
pub fn inverse_normalizer(min: u64, x: u64, max: u64) -> Decimal {
    Decimal::from_ratio(max - x, max - min)
}
