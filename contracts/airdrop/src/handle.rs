use crate::state::{
    account_r,
    account_total_claimed_r,
    account_total_claimed_w,
    account_viewkey_w,
    account_w,
    address_in_account_w,
    claim_status_r,
    claim_status_w,
    config_r,
    config_w,
    decay_claimed_w,
    revoke_permit,
    total_claimed_r,
    total_claimed_w,
    validate_address_permit,
};
use rs_merkle::{algorithms::Sha256, Hasher, MerkleProof};
use shade_protocol::{
    c_std::{
        from_binary,
        to_binary,
        Addr,
        Api,
        Binary,
        Decimal,
        DepsMut,
        Env,
        MessageInfo,
        Querier,
        Response,
        StdError,
        StdResult,
        Storage,
        SubMsg,
        Uint128,
    },
    contract_interfaces::airdrop::{
        account::{Account, AccountKey, AddressProofMsg, AddressProofPermit},
        claim_info::RequiredTask,
        errors::{
            account_already_created,
            account_does_not_exist,
            address_already_in_account,
            airdrop_ended,
            airdrop_not_started,
            claim_too_high,
            decay_claimed,
            decay_not_set,
            expected_memo,
            invalid_dates,
            invalid_partial_tree,
            invalid_task_percentage,
            not_admin,
            nothing_to_claim,
            permit_rejected,
            unexpected_error,
        },
        Config,
        ExecuteAnswer,
    },
    query_authentication::viewing_keys::ViewingKey,
    snip20::helpers::send_msg,
    utils::generic_response::{ResponseStatus, ResponseStatus::Success},
};

#[allow(clippy::too_many_arguments)]
pub fn try_update_config(
    deps: DepsMut,
    _env: Env,
    info: &MessageInfo,
    admin: Option<Addr>,
    dump_address: Option<Addr>,
    query_rounding: Option<Uint128>,
    start_date: Option<u64>,
    end_date: Option<u64>,
    decay_start: Option<u64>,
) -> StdResult<Response> {
    let config = config_r(deps.storage).load()?;
    // Check if admin
    if info.sender != config.admin {
        return Err(not_admin(config.admin.as_str()));
    }

    // Save new info
    let mut config = config_w(deps.storage);
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
                    return Err(invalid_dates(
                        "EndDate",
                        end_date.to_string().as_str(),
                        "before",
                        "StartDate",
                        start_date.to_string().as_str(),
                    ));
                }
            } else if let Some(end_date) = state.end_date {
                if start_date > end_date {
                    return Err(invalid_dates(
                        "EndDate",
                        end_date.to_string().as_str(),
                        "before",
                        "StartDate",
                        start_date.to_string().as_str(),
                    ));
                }
            }
            if let Some(start_decay) = decay_start {
                if start_date > start_decay {
                    return Err(invalid_dates(
                        "Decay",
                        start_decay.to_string().as_str(),
                        "before",
                        "StartDate",
                        start_date.to_string().as_str(),
                    ));
                }
            } else if let Some(start_decay) = state.decay_start {
                if start_date > start_decay {
                    return Err(invalid_dates(
                        "Decay",
                        start_decay.to_string().as_str(),
                        "before",
                        "StartDate",
                        start_date.to_string().as_str(),
                    ));
                }
            }

            state.start_date = start_date;
        }
        if let Some(end_date) = end_date {
            // Avoid date collisions
            if let Some(decay_start) = decay_start {
                if decay_start > end_date {
                    return Err(invalid_dates(
                        "EndDate",
                        end_date.to_string().as_str(),
                        "before",
                        "Decay",
                        decay_start.to_string().as_str(),
                    ));
                }
            } else if let Some(decay_start) = state.decay_start {
                if decay_start > end_date {
                    return Err(invalid_dates(
                        "EndDate",
                        end_date.to_string().as_str(),
                        "before",
                        "Decay",
                        decay_start.to_string().as_str(),
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
    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::UpdateConfig { status: Success })?))
}

pub fn try_add_tasks(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    tasks: Vec<RequiredTask>,
) -> StdResult<Response> {
    let config = config_r(deps.storage).load()?;
    // Check if admin
    if info.sender != config.admin {
        return Err(not_admin(config.admin.as_str()));
    }

    config_w(deps.storage).update(|mut config| {
        let mut task_list = tasks;
        config.task_claim.append(&mut task_list);

        //Validate that they do not exceed 100
        let mut count = Uint128::zero();
        for task in config.task_claim.iter() {
            count += task.percent;
        }

        if count > Uint128::new(100u128) {
            return Err(invalid_task_percentage(count.to_string().as_str()));
        }

        Ok(config)
    })?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::AddTask { status: Success })?))
}

pub fn try_account(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    addresses: Vec<AddressProofPermit>,
    partial_tree: Vec<Binary>,
) -> StdResult<Response> {
    // Check if airdrop active
    let config = config_r(deps.storage).load()?;

    // Check that airdrop hasn't ended
    available(&config, env)?;

    // Setup account
    let sender = info.sender.to_string();

    // These variables are setup to facilitate updating
    let updating_account: bool;
    let old_claim_amount: Uint128;

    let mut account = match account_r(deps.storage).may_load(sender.as_bytes())? {
        None => {
            updating_account = false;
            old_claim_amount = Uint128::zero();
            let mut account = Account::default();

            // Validate permits
            try_add_account_addresses(
                deps.storage,
                deps.api,
                &config,
                &info.sender,
                &mut account,
                addresses.clone(),
                partial_tree.clone(),
            )?;

            // Add default claim at index 0
            account_total_claimed_w(deps.storage).save(sender.as_bytes(), &Uint128::zero())?;
            claim_status_w(deps.storage, 0).save(sender.as_bytes(), &false)?;

            account
        }
        Some(acc) => {
            updating_account = true;
            old_claim_amount = acc.total_claimable;
            acc
        }
    };

    // Claim airdrop
    let mut messages = vec![];

    let (completed_percentage, unclaimed_percentage) =
        update_tasks(deps.storage, &config, sender.clone())?;

    let mut redeem_amount = Uint128::zero();

    if unclaimed_percentage > Uint128::zero() {
        redeem_amount = claim_tokens(
            deps.storage,
            env,
            info,
            &config,
            &account,
            completed_percentage,
            unclaimed_percentage,
        )?;
    }

    // Update account after claim to calculate difference
    if updating_account {
        // Validate permits
        try_add_account_addresses(
            deps.storage,
            deps.api,
            &config,
            &info.sender,
            &mut account,
            addresses.clone(),
            partial_tree.clone(),
        )?;
    }

    if updating_account && completed_percentage > Uint128::zero() {
        // Calculate the total new address amount
        let added_address_total = account.total_claimable.checked_sub(old_claim_amount)?;
        account_total_claimed_w(deps.storage).update(sender.as_bytes(), |claimed| {
            if let Some(claimed) = claimed {
                let new_redeem: Uint128;
                if completed_percentage == Uint128::new(100u128) {
                    new_redeem =
                        added_address_total * decay_factor(env.block.time.seconds(), &config);
                } else {
                    new_redeem = completed_percentage
                        .multiply_ratio(added_address_total, Uint128::new(100u128))
                        * decay_factor(env.block.time.seconds(), &config);
                }

                redeem_amount += new_redeem;
                Ok(claimed + new_redeem)
            } else {
                Err(unexpected_error())
            }
        })?;
    }

    if redeem_amount > Uint128::zero() {
        total_claimed_w(deps.storage)
            .update(|claimed| -> StdResult<Uint128> { Ok(claimed + redeem_amount) })?;

        messages.push(send_msg(
            info.sender.to_string(),
            redeem_amount.into(),
            None,
            None,
            None,
            &config.airdrop_snip20,
        )?);
    }

    // Save account
    account_w(deps.storage).save(sender.as_bytes(), &account)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Account {
        status: ResponseStatus::Success,
        total: account.total_claimable,
        claimed: account_total_claimed_r(deps.storage).load(sender.to_string().as_bytes())?,
        // Will always be 0 since rewards are automatically claimed here
        finished_tasks: finished_tasks(deps.storage, sender.clone())?,
        addresses: account.addresses,
    })?))
}

pub fn try_disable_permit_key(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    key: String,
) -> StdResult<Response> {
    revoke_permit(deps.storage, info.sender.to_string(), key);

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::DisablePermitKey {
            status: Success,
        })?),
    )
}

pub fn try_set_viewing_key(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    key: String,
) -> StdResult<Response> {
    account_viewkey_w(deps.storage)
        .save(&info.sender.to_string().as_bytes(), &AccountKey(key).hash())?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::SetViewingKey {
            status: Success,
        })?),
    )
}

pub fn try_complete_task(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    account: Addr,
) -> StdResult<Response> {
    let config = config_r(deps.storage).load()?;

    for (i, task) in config.task_claim.iter().enumerate() {
        if task.address == info.sender {
            claim_status_w(deps.storage, i).update(
                account.to_string().as_bytes(),
                |status| -> StdResult<bool> {
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

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::CompleteTask { status: Success })?))
}

pub fn try_claim(deps: DepsMut, env: &Env, info: &MessageInfo) -> StdResult<Response> {
    let config = config_r(deps.storage).load()?;

    // Check that airdrop hasn't ended
    available(&config, env)?;

    // Get account
    let sender = info.sender.clone();
    let account = account_r(deps.storage).load(sender.to_string().as_bytes())?;

    // Calculate airdrop
    let (completed_percentage, unclaimed_percentage) =
        update_tasks(deps.storage, &config, sender.to_string())?;

    if unclaimed_percentage == Uint128::zero() {
        return Err(nothing_to_claim());
    }

    let redeem_amount = claim_tokens(
        deps.storage,
        env,
        info,
        &config,
        &account,
        completed_percentage,
        unclaimed_percentage,
    )?;

    total_claimed_w(deps.storage)
        .update(|claimed| -> StdResult<Uint128> { Ok(claimed + redeem_amount) })?;

    Ok(Response::new()
        .set_data(to_binary(&ExecuteAnswer::Claim {
            status: ResponseStatus::Success,
            total: account.total_claimable,
            claimed: account_total_claimed_r(deps.storage).load(sender.to_string().as_bytes())?,
            finished_tasks: finished_tasks(deps.storage, sender.to_string())?,
            addresses: account.addresses,
        })?)
        .add_message(send_msg(
            sender.to_string(),
            redeem_amount.into(),
            None,
            None,
            None,
            &config.airdrop_snip20,
        )?))
}

pub fn try_claim_decay(deps: DepsMut, env: &Env, info: &MessageInfo) -> StdResult<Response> {
    let config = config_r(deps.storage).load()?;

    // Check if airdrop ended
    if let Some(end_date) = config.end_date {
        if let Some(dump_address) = config.dump_address {
            if env.block.time.seconds() > end_date {
                decay_claimed_w(deps.storage).update(|claimed| {
                    if claimed {
                        Err(decay_claimed())
                    } else {
                        Ok(true)
                    }
                })?;

                let total_claimed = total_claimed_r(deps.storage).load()?;
                let send_total = config.airdrop_amount.checked_sub(total_claimed)?;
                let messages = vec![send_msg(
                    dump_address.to_string(),
                    send_total.into(),
                    None,
                    None,
                    None,
                    &config.airdrop_snip20,
                )?];

                return Ok(Response::new()
                    .set_data(to_binary(&ExecuteAnswer::ClaimDecay { status: Success })?));
            }
        }
    }

    Err(decay_not_set())
}

pub fn finished_tasks(storage: &dyn Storage, account: String) -> StdResult<Vec<RequiredTask>> {
    let mut finished_tasks = vec![];
    let config = config_r(storage).load()?;

    for (index, task) in config.task_claim.iter().enumerate() {
        match claim_status_r(storage, index).may_load(account.as_bytes())? {
            None => {}
            Some(_) => {
                finished_tasks.push(task.clone());
            }
        }
    }

    Ok(finished_tasks)
}

/// Gets task information and sets them
pub fn update_tasks(
    storage: &mut dyn Storage,
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

pub fn claim_tokens(
    storage: &mut dyn Storage,
    env: &Env,
    info: &MessageInfo,
    config: &Config,
    account: &Account,
    completed_percentage: Uint128,
    unclaimed_percentage: Uint128,
) -> StdResult<Uint128> {
    // send_amount
    let sender = info.sender.to_string();

    // Amount to be redeemed
    let mut redeem_amount = Uint128::zero();

    // Update total claimed and calculate claimable
    account_total_claimed_w(storage).update(sender.as_bytes(), |claimed| {
        if let Some(claimed) = claimed {
            // This solves possible uToken inaccuracies
            if completed_percentage == Uint128::new(100u128) {
                redeem_amount = account.total_claimable.checked_sub(claimed)?;
            } else {
                redeem_amount = unclaimed_percentage
                    .multiply_ratio(account.total_claimable, Uint128::new(100u128));
            }

            // Update redeem amount with the decay multiplier
            redeem_amount = redeem_amount * decay_factor(env.block.time.seconds(), config);

            Ok(claimed + redeem_amount)
        } else {
            Err(account_does_not_exist())
        }
    })?;

    Ok(redeem_amount)
}

/// Validates all of the information and updates relevant states
pub fn try_add_account_addresses(
    storage: &mut dyn Storage,
    api: &dyn Api,
    config: &Config,
    sender: &Addr,
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

            // Avoid verifying sender
            if &params.address != sender {
                // Check permit legitimacy
                validate_address_permit(storage, api, permit, &params, config.contract.clone())?;
            }

            // Check that airdrop amount does not exceed maximum
            if params.amount > config.max_amount {
                return Err(claim_too_high(
                    params.amount.to_string().as_str(),
                    config.max_amount.to_string().as_str(),
                ));
            }

            // Update address if its not in an account
            address_in_account_w(storage).update(
                params.address.to_string().as_bytes(),
                |state| -> StdResult<bool> {
                    if state.is_some() {
                        return Err(address_already_in_account(params.address.as_str()));
                    }

                    Ok(true)
                },
            )?;

            // Add account as a leaf
            let leaf_hash =
                Sha256::hash((params.address.to_string() + &params.amount.to_string()).as_bytes());
            leaves_to_validate.push((params.index as usize, leaf_hash));

            // If valid then add to account array and sum total amount
            account.addresses.push(params.address);
            account.total_claimable += params.amount;
        } else {
            return Err(expected_memo());
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
        return Err(invalid_partial_tree());
    }

    Ok(())
}

pub fn available(config: &Config, env: &Env) -> StdResult<()> {
    let current_time = env.block.time.seconds();

    // Check if airdrop started
    if current_time < config.start_date {
        return Err(airdrop_not_started(
            config.start_date.to_string().as_str(),
            current_time.to_string().as_str(),
        ));
    }
    if let Some(end_date) = config.end_date {
        if current_time > end_date {
            return Err(airdrop_ended(
                end_date.to_string().as_str(),
                current_time.to_string().as_str(),
            ));
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
