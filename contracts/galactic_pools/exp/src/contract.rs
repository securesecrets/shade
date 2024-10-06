use std::ops::{Add, AddAssign, Sub};

use rand::distributions::Uniform;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use s_toolkit::permit::{validate, Permit, TokenPermissions};
use s_toolkit::utils::types::Contract;
use s_toolkit::viewing_key::{ViewingKey, ViewingKeyStore};
use shade_protocol::c_std;

use crate::error::{self, ContractError};
use crate::msg::{
    AddContract, ExecuteAnswer, ExecuteMsg, InstantiateMsg, QueryAnswer, QueryMsg, QueryWithPermit,
    ResponseStatus::Success, VerifiedContractRes, WeightUpdate,
};
use crate::msg::{ConfigRes, Entropy, MintingSchedule};
use crate::state::{
    sort_schedule, Config, ContractStored, Schedule, ScheduleUnit, SupplyPool, VerifiedContract,
    XpSlot, CONFIG, EXP_ACCOUNTS, PREFIX_REVOKED_PERMITS, SUPPLY_POOL, VERIFIED_CONTRACTS,
    XP_APPEND_STORE, XP_NONCE,
};
use sha2::{Digest, Sha256};

use shade_protocol::{
    c_std::{
        entry_point, to_binary, Addr, Attribute, Binary, CosmosMsg, Deps, DepsMut, Env,
        MessageInfo, Response, StdError, Storage, Uint128, WasmMsg,
    },
    s_toolkit,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Initialize admins
    let mut admins = Vec::new();
    if let Some(ad) = msg.admin {
        for admin in ad {
            admins.push(admin);
        }
    } else {
        admins.push(info.sender)
    }

    // Process minting schedules and calculate total XP
    let mut total_xp: u128 = 0u128;
    let mut schedules_vec: Vec<ScheduleUnit> = Vec::new();
    //we use schedule's max duration and turn it into season_duration
    let mut max_duration: u64 = 0;
    for schedule in &msg.schedules {
        total_xp += schedule.duration as u128 * schedule.mint_per_block.u128();

        schedules_vec.push(ScheduleUnit {
            end_block: schedule.duration.add(env.block.height),
            mint_per_block: schedule.mint_per_block,
            duration: schedule.duration,
            start_block: schedule
                .start_after
                .unwrap_or_default()
                .add(env.block.height),
            start_after: schedule.start_after,
        });

        if schedule.duration > max_duration {
            max_duration = schedule.duration;
        }
    }

    // Create the contract configuration
    let config = Config {
        admins,
        contract_address: env.contract.address,
        total_weight: 0,
        minting_schedule: schedules_vec,
        season_counter: 1,
        verified_contracts: Vec::new(),
        season_duration: max_duration,
        season_ending_block: env.block.height.add(max_duration),
        season_starting_block: env.block.height,
        grand_prize_contract: None,
    };

    //Save data to storage
    CONFIG.save(deps.storage, &config)?;

    SUPPLY_POOL.save(
        deps.storage,
        config.season_counter,
        &SupplyPool {
            season_total_xp_cap: Uint128::from(total_xp),
            xp_claimed_by_contracts: Uint128::default(),
            xp_claimed_by_users: Uint128::default(),
        },
    )?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Instantiate { status: Success })?))
}

//-------------------------------------------- HANDLES ---------------------------------
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        //Admins
        ExecuteMsg::AddAdmin { address } => try_add_admin(deps, env, info, address),
        ExecuteMsg::RemoveAdmin { address } => try_remove_admin(deps, env, info, address),
        ExecuteMsg::AddContract { contracts, .. } => try_add_contract(deps, env, info, contracts),
        ExecuteMsg::RemoveContract { contracts } => try_remove_contract(deps, env, info, contracts),
        ExecuteMsg::SetGrandPrizeContract { address } => {
            try_set_grand_prize_contract(deps, env, info, address)
        }
        ExecuteMsg::SetSchedule { schedule } => try_set_schedule(deps, env, info, schedule),
        ExecuteMsg::ResetSeason {} => try_reset_season(deps, env, info),
        ExecuteMsg::UpdateWeights { weights } => try_update_weights(deps, env, info, weights),

        //Verified Contracts
        ExecuteMsg::AddExp { address, exp } => try_add_exp(deps, env, info, address, exp),
        ExecuteMsg::UpdateLastClaimed {} => try_update_last_claimed(deps, env, info),

        //Verified Contracts + Users + Grand prize contract
        ExecuteMsg::CreateViewingKey { entropy } => try_create_key(deps, env, info, entropy),
        ExecuteMsg::SetViewingKey { key, .. } => try_set_key(deps, info, &key),

        //Grand prize contract + Admin
        ExecuteMsg::GetWinners { no_of_winners } => {
            try_get_winners(deps.as_ref(), env, no_of_winners)
        }
    }
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::ContractInfo {} => query_contract_info(deps, env),
        QueryMsg::VerifiedContracts {
            start_page,
            page_size,
        } => query_verified_contracts(deps, env, start_page, page_size),

        QueryMsg::WithPermit { permit, query } => permit_queries(deps, permit, query),

        _ => viewing_keys_queries(deps, env, msg),
    }
}

/// Returns Result<Response, ContractError>
///
/// Adds admin address
///
/// # Arguments
///
/// * `deps`    - DepsMut containing all the contract's external dependencies
/// * `env`     - Env of contract's environment
/// * `info`    - Carries the info of who sent the message and how much native funds were sent along
fn try_add_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: Addr,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    enforce_admin(deps.as_ref(), &config, &info.sender)?;

    config.admins.push(address);

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::AddAdmin { status: Success })?))
}

/// Returns Result<Response, ContractError>
///
/// Removes admin address
///
/// # Arguments
///
/// * `deps`    - DepsMut containing all the contract's external dependencies
/// * `env`     - Env of contract's environment
/// * `info`    - Carries the info of who sent the message and how much native funds were sent along
fn try_remove_admin(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: Addr,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    enforce_admin(deps.as_ref(), &config, &info.sender)?;

    if !config.admins.contains(&address) {
        return Err(ContractError::CustomError {
            val: format!("Address not found in admins: {}", address),
        });
    }

    config.admins.retain(|addr| addr != &address);

    if config.admins.is_empty() {
        return Err(ContractError::CustomError {
            val: "Cannot remove the last admin".to_string(),
        });
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::RemoveAdmin { status: Success })?))
}

/// Returns Result<Response, ContractError>
///
/// Adds verified contracts
///
/// # Arguments
///
/// * `deps`    - DepsMut containing all the contract's external dependencies
/// * `env`     - Env of contract's environment
/// * `info`    - Carries the info of who sent the message and how much native funds were sent along
fn try_add_contract(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contracts: Vec<AddContract>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    enforce_admin(deps.as_ref(), &config, &info.sender)?;
    // Collect existing contracts and their rewards
    let mut new_weight = 0u64;
    let mut supply_pool = SUPPLY_POOL.load(deps.storage, config.season_counter)?;

    for contract in &config.verified_contracts {
        let mut verf_contract = VERIFIED_CONTRACTS.load(deps.storage, contract)?;
        let rewards = get_exp(
            env.block.height,
            config.total_weight,
            &config.minting_schedule,
            verf_contract.clone(),
        );
        verf_contract.available_exp += Uint128::from(rewards);
        supply_pool.xp_claimed_by_contracts += Uint128::from(rewards);

        verf_contract.last_claimed = env.block.height;
        VERIFIED_CONTRACTS.save(deps.storage, &contract, &verf_contract)?;
    }

    for contract in &contracts {
        if config.verified_contracts.contains(&contract.address) {
            continue;
        }

        config.verified_contracts.push(contract.address.clone());

        VERIFIED_CONTRACTS.save(
            deps.storage,
            &contract.address,
            &VerifiedContract {
                code_hash: contract.code_hash.clone(),
                available_exp: Uint128::zero(),
                weight: contract.weight,
                last_claimed: env.block.height,
                total_xp: Uint128::default(),
                xp_claimed: Uint128::default(),
            },
        )?;

        if let Some(final_value) = new_weight.checked_add(contract.weight) {
            new_weight = final_value;
        } else {
            return Err(ContractError::CustomError {
                val: "Overflow while adding weights".to_string(),
            });
        }
    }

    if let Some(final_weight) = config.total_weight.checked_add(new_weight) {
        config.total_weight = final_weight;
        let supply_pool = SUPPLY_POOL.load(deps.storage, config.season_counter)?;
        let total_xp_remaining =
            supply_pool.season_total_xp_cap - supply_pool.xp_claimed_by_contracts;

        for contract in &config.verified_contracts {
            let mut verf_contract = VERIFIED_CONTRACTS.load(deps.storage, contract)?;
            verf_contract.total_xp = verf_contract.xp_claimed;

            if config.total_weight > 0 {
                verf_contract.total_xp += total_xp_remaining
                    * Uint128::from((verf_contract.weight / config.total_weight) as u128);
            }

            VERIFIED_CONTRACTS.save(deps.storage, &contract, &verf_contract)?;
        }
    } else {
        return Err(ContractError::CustomError {
            val: "Overflow while adding weights".to_string(),
        });
    }

    CONFIG.save(deps.storage, &config)?;
    SUPPLY_POOL.save(deps.storage, config.season_counter, &supply_pool)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::AddContract { status: Success })?))
}

/// Returns Result<Response, ContractError>
///
/// Removes verified contracts
///
/// # Arguments
///
/// * `deps`    - DepsMut containing all the contract's external dependencies
/// * `env`     - Env of contract's environment
/// * `info`    - Carries the info of who sent the message and how much native funds were sent along
fn try_remove_contract(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contracts: Vec<Addr>,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    enforce_admin(deps.as_ref(), &config, &info.sender)?;
    // iterate through all the contracts and make their rewards available
    // Check if contracts vector is empty
    if contracts.is_empty() {
        return Err(ContractError::CustomError {
            val: "No contracts provided for removal".to_string(),
        });
    }

    if config.verified_contracts.is_empty() {
        return Err(ContractError::CustomError {
            val: "Total weight of contracts is zero".to_string(),
        });
    }

    // Iterate through all the contracts and make their rewards available
    let mut supply_pool = SUPPLY_POOL.load(deps.storage, config.season_counter)?;

    for contract in &config.verified_contracts {
        let mut verf_contract = VERIFIED_CONTRACTS.load(deps.storage, contract)?;
        let rewards = get_exp(
            env.block.height,
            config.total_weight,
            &config.minting_schedule,
            verf_contract.clone(),
        );
        if let Ok(xp) = verf_contract.available_exp.checked_add(rewards.into()) {
            verf_contract.available_exp = xp;
        }
        if let Ok(xp) = supply_pool
            .xp_claimed_by_contracts
            .checked_add(rewards.into())
        {
            supply_pool.xp_claimed_by_contracts = xp;
        }
        // supply_pool.exp_claimed_by_contracts = supply_pool
        //     .exp_claimed_by_contracts
        //     .checked_add(rewards.into())?;
        // // supply_pool.exp_claimed_by_contracts += Uint128::from(rewards);
        verf_contract.last_claimed = env.block.height;
        VERIFIED_CONTRACTS.save(deps.storage, &contract, &verf_contract)?;
    }

    let mut missing_contracts: Vec<Attribute> = vec![];
    let mut new_weight = 0u64;

    for contract in contracts {
        if !VERIFIED_CONTRACTS.has(deps.storage, &contract) {
            let fail = Attribute {
                key: "Missing: ".to_string(),
                value: contract.to_string(),
                encrypted: false,
            };
            missing_contracts.push(fail);
            continue;
        }

        let mut con = VERIFIED_CONTRACTS.load(deps.storage, &contract)?;
        new_weight += con.weight;
        con.weight = 0u64;
        VERIFIED_CONTRACTS.save(deps.storage, &contract, &con)?;
    }

    // config.total_weight.sub_assign(new_weight);

    if let Some(w) = config.total_weight.checked_sub(new_weight) {
        config.total_weight = w;
    } else {
        return Err(ContractError::Overflow {});
    }
    let supply_pool = SUPPLY_POOL.load(deps.storage, config.season_counter)?;
    let total_xp_remaining = supply_pool.season_total_xp_cap - supply_pool.xp_claimed_by_contracts;

    for contract in &config.verified_contracts {
        let mut verf_contract = VERIFIED_CONTRACTS.load(deps.storage, contract)?;
        verf_contract.total_xp = verf_contract.xp_claimed
            + total_xp_remaining
                * Uint128::from((verf_contract.weight / config.total_weight) as u128);

        VERIFIED_CONTRACTS.save(deps.storage, &contract, &verf_contract)?;
    }
    CONFIG.save(deps.storage, &config)?;
    SUPPLY_POOL.save(deps.storage, config.season_counter, &supply_pool)?;

    Ok(Response::new()
        .add_attributes(missing_contracts)
        .set_data(to_binary(&ExecuteAnswer::RemoveContract {
            status: Success,
        })?))
}

/// Returns Result<Response, ContractError>
///
/// Set grand prize contract
///
/// # Arguments
///
/// * `deps`    - DepsMut containing all the contract's external dependencies
/// * `env`     - Env of contract's environment
/// * `info`    - Carries the info of who sent the message and how much native funds were sent along
fn try_set_grand_prize_contract(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    address: Addr,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    enforce_admin(deps.as_ref(), &config, &info.sender)?;

    config.grand_prize_contract = Some(address);

    CONFIG.save(deps.storage, &config)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::SetGrandPrizeContract {
            status: Success,
        })?),
    )
}

/// Returns Result<Response, ContractError>
///
/// Set minting schedule for xp
///
/// # Arguments
///
/// * `deps`        - DepsMut containing all the contract's external dependencies
/// * `env`         - Env of contract's environment
/// * `info`        - Carries the info of who sent the message and how much native funds were sent along
/// * `schedule`    - Minting schedule
fn try_set_schedule(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    schedule: MintingSchedule,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    enforce_admin(deps.as_ref(), &config, &info.sender)?;
    let mut supply_pool = SUPPLY_POOL.load(deps.storage, config.season_counter)?;

    //get xp already minted
    let already_mined_xp = get_exp(
        env.block.height,
        config.total_weight,
        &config.minting_schedule,
        VerifiedContract {
            code_hash: String::new(),
            available_exp: Uint128::default(),
            weight: config.total_weight,
            last_claimed: config.season_starting_block,
            total_xp: Uint128::default(),
            xp_claimed: Uint128::default(), // TODO check this
        },
    );

    //Should have the ability to
    //1) update the duration of season
    //2) should update the minting of a schedule

    let mut total_xp: u128 = 0u128;
    let mut schedule_vec: Vec<ScheduleUnit> = Vec::new();
    //we use schedule's max duration and turn it into season_duration
    let mut max_end_block: u64 = 0;
    for sch in schedule {
        let start_block = sch.start_after.unwrap_or_default().add(env.block.height);

        let mut end_block;

        if sch.continue_with_current_season {
            let difference = (env.block.height).sub(config.season_starting_block);
            end_block = sch.duration.add(env.block.height).sub(difference);
            total_xp += (sch.duration - difference) as u128 * sch.mint_per_block.u128();
        } else {
            end_block = sch.duration.add(env.block.height);
            total_xp += sch.duration as u128 * sch.mint_per_block.u128();
        }

        end_block = end_block.add(sch.start_after.unwrap_or_default());

        schedule_vec.push(ScheduleUnit {
            end_block,
            mint_per_block: sch.mint_per_block,
            duration: sch.duration,
            start_block,
            start_after: sch.start_after,
        });

        if end_block > max_end_block {
            max_end_block = end_block;
        }
    }

    config.season_ending_block = max_end_block;
    config.season_duration = max_end_block.sub(config.season_starting_block);

    let mut s = schedule_vec;
    sort_schedule(&mut s);

    config.minting_schedule = s;

    total_xp += already_mined_xp;

    supply_pool.season_total_xp_cap = Uint128::from(total_xp);

    // config.minting_schedule = s;
    CONFIG.save(deps.storage, &config)?;
    SUPPLY_POOL.save(deps.storage, config.season_counter, &supply_pool)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::SetSchedule { status: Success })?))
}

/// Returns Result<Response, ContractError>
///
/// Resets the seasons
///
/// # Arguments
///
/// * `deps`        - DepsMut containing all the contract's external dependencies
/// * `env`         - Env of contract's environment
/// * `info`        - Carries the info of who sent the message and how much native funds were sent along
fn try_reset_season(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;
    enforce_admin(deps.as_ref(), &config, &info.sender)?;

    config.season_counter = config.season_counter.add(1u64);

    // TODO send transaction to grandprize contract to end round.
    let mut max_end_block: u64 = 0;
    let mut total_xp: u128 = 0u128;

    //RESETS the schedule using duraion.

    for schedule in &mut config.minting_schedule {
        schedule.start_block = env
            .block
            .height
            .add(schedule.start_after.unwrap_or_default());

        schedule.end_block = env
            .block
            .height
            .add(schedule.duration)
            .add(schedule.start_after.unwrap_or_default());

        total_xp += schedule.duration as u128 * schedule.mint_per_block.u128();

        if schedule.end_block > max_end_block {
            max_end_block = schedule.end_block;
        }
    }

    config.season_starting_block = env.block.height;
    config.season_ending_block = max_end_block;
    config.season_duration = max_end_block.sub(env.block.height);

    CONFIG.save(deps.storage, &config)?;

    let supply_pool = SupplyPool {
        season_total_xp_cap: Uint128::from(total_xp),
        xp_claimed_by_contracts: Uint128::zero(),
        xp_claimed_by_users: Uint128::zero(),
    };
    SUPPLY_POOL.save(deps.storage, config.season_counter, &supply_pool)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::EndRound { status: Success })?))
}

/// Returns Result<Response, ContractError>
///
/// Update the weights of already exiting contracts
///
/// # Arguments
///
/// * `deps`        - DepsMut containing all the contract's external dependencies
/// * `env`         - Env of contract's environment
/// * `info`        - Carries the info of who sent the message and how much native funds were sent along
fn try_update_weights(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    contracts: Vec<WeightUpdate>,
) -> Result<Response, ContractError> {
    // let mut state = config_read(&deps.storage).load()?;
    let mut config = CONFIG.load(deps.storage)?;
    enforce_admin(deps.as_ref(), &config, &info.sender)?;

    let mut new_weight_counter: u64 = 0;
    let mut old_weight_counter: u64 = 0;

    // Update reward contracts one by one
    for to_update in contracts {
        let raw_contract = VERIFIED_CONTRACTS.load(deps.storage, &to_update.address);
        let mut contract;
        if raw_contract.is_ok() {
            contract = raw_contract?;
        } else {
            return Err(ContractError::CustomError {
                val: format!(
                    "Contract address {} is not a a verified contract. Add contract first",
                    to_update.address
                ),
            });
        }

        // There is no need to update a SPY twice in a block, and there is no need to update a SPY
        // that had 0 weight until now
        if contract.last_claimed < env.block.height && contract.weight > 0 {
            // Calc amount to mint for this spy contract and push to messages
            let rewards = get_exp(
                env.block.height,
                config.total_weight,
                &config.minting_schedule,
                contract.clone(),
            );
            contract.available_exp += Uint128::from(rewards);
        }
        let old_weight = contract.weight;
        let new_weight = to_update.weight;

        // Set new weight and update total counter
        contract.weight = new_weight;
        contract.last_claimed = env.block.height;
        VERIFIED_CONTRACTS.save(deps.storage, &to_update.address, &contract)?;

        // Update counters to batch update after the loop
        new_weight_counter = new_weight_counter
            .checked_add(new_weight)
            .ok_or(ContractError::Overflow {})?;
        old_weight_counter = old_weight_counter
            .checked_add(old_weight)
            .ok_or(ContractError::Overflow {})?;
    }

    config.total_weight = config
        .total_weight
        .checked_sub(old_weight_counter)
        .and_then(|intermediate| intermediate.checked_add(new_weight_counter))
        .ok_or(ContractError::Underflow {})?;

    let supply_pool = SUPPLY_POOL.load(deps.storage, config.season_counter)?;
    let total_xp_remaining = supply_pool.season_total_xp_cap - supply_pool.xp_claimed_by_contracts;

    for contract in &config.verified_contracts {
        let mut verf_contract = VERIFIED_CONTRACTS.load(deps.storage, contract)?;
        verf_contract.total_xp = verf_contract.xp_claimed;

        if config.total_weight > 0 {
            verf_contract.total_xp += total_xp_remaining
                * Uint128::from((verf_contract.weight / config.total_weight) as u128);
        }

        VERIFIED_CONTRACTS.save(deps.storage, &contract, &verf_contract)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::UpdateWeights {
            status: Success,
        })?),
    )
}

/// Returns Result<Response, ContractError>
///
/// Adds exp to user account
///
/// # Arguments
///
/// * `deps`    - DepsMut containing all the contract's external dependencies
/// * `env`     - Env of contract's environment
/// * `info`    - Carries the info of who sent the message and how much native funds were sent along
fn try_add_exp(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    user_address: Addr,
    exp: Uint128,
) -> Result<Response, ContractError> {
    if !VERIFIED_CONTRACTS.has(deps.storage, &info.sender) {
        return Err(ContractError::CustomError {
            val: "This function can only be called by a verified contract".to_string(),
        });
    }
    let config = CONFIG.load(deps.storage)?;

    let mut contract = VERIFIED_CONTRACTS.load(deps.storage, &info.sender).unwrap();

    if exp > contract.available_exp {
        return Err(ContractError::CustomError {
            val: "Cannot assign more exp then available".to_string(),
        });
    }

    contract.available_exp -= exp;
    VERIFIED_CONTRACTS.save(deps.storage, &info.sender, &contract)?;

    if !EXP_ACCOUNTS.has(deps.storage, (&user_address, config.season_counter)) {
        EXP_ACCOUNTS.save(deps.storage, (&user_address, config.season_counter), &exp)?;
    } else {
        let old_exp = EXP_ACCOUNTS
            .load(deps.storage, (&user_address, config.season_counter))
            .unwrap();
        let new_exp = old_exp + exp;
        EXP_ACCOUNTS.save(
            deps.storage,
            (&user_address, config.season_counter),
            &new_exp,
        )?;
    }
    //updating supply pool
    let mut supply_pool = SUPPLY_POOL
        .load(deps.storage, config.season_counter)
        .unwrap();
    supply_pool.xp_claimed_by_users += exp;
    SUPPLY_POOL.save(deps.storage, config.season_counter, &supply_pool)?;

    //Add XP slot
    let mut xp_nonce = XP_NONCE
        .load(deps.storage, config.season_counter)
        .unwrap_or_default();

    XP_APPEND_STORE
        .add_suffix(&format!("{}", config.season_counter))
        .push(
            deps.storage,
            &XpSlot {
                starting_slot: xp_nonce.add(Uint128::one()),
                ending_slot: xp_nonce.add(exp),
                user_address,
            },
        )?;
    xp_nonce.add_assign(exp);

    XP_NONCE.save(deps.storage, config.season_counter, &xp_nonce)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::AddExp { status: Success })?))
}

/// Returns Result<Response, ContractError>
///
/// update the contract's exp
///
/// # Arguments
///
/// * `deps`    - DepsMut containing all the contract's external dependencies
/// * `env`     - Env of contract's environment
/// * `info`    - Carries the info of who sent the message and how much native funds were sent along
fn try_update_last_claimed(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if env.block.height > config.season_ending_block {}

    if !VERIFIED_CONTRACTS.has(deps.storage, &info.sender) {
        return Err(ContractError::CustomError {
            val: "This function can only be called by a verified contract".to_string(),
        });
    }

    let mut contract = VERIFIED_CONTRACTS.load(deps.storage, &info.sender).unwrap();

    let available_exp = Uint128::from(get_exp(
        env.block.height,
        config.total_weight,
        &config.minting_schedule,
        contract.clone(),
    ));

    contract.available_exp += available_exp;

    if env.block.height <= config.season_ending_block {
        contract.last_claimed = env.block.height;
    } else {
        contract.last_claimed = config.season_ending_block;
    }

    VERIFIED_CONTRACTS.save(deps.storage, &info.sender, &contract)?;

    let mut supply_pool = SUPPLY_POOL
        .load(deps.storage, config.season_counter)
        .unwrap();
    supply_pool.xp_claimed_by_contracts += available_exp;

    SUPPLY_POOL.save(deps.storage, config.season_counter, &supply_pool)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::UpdateLastClaimed {
            status: Success,
        })?),
    )
}

/// Returns Result<Response, ContractError>
///
/// create a viewing key
///
/// # Arguments
///
/// * `deps`    - DepsMut containing all the contract's external dependencies
/// * `env`     - Env of contract's environment
/// * `info`    - Carries the info of who sent the message and how much native funds were sent along
/// * `entropy` - string to be used as an entropy source for randomization
fn try_create_key(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    entropy: String,
) -> Result<Response, ContractError> {
    let key = ViewingKey::create(
        deps.storage,
        &info,
        &env,
        info.sender.as_str(),
        entropy.as_bytes(),
    );

    Ok(Response::new()
        .add_attribute("viewing_key", key)
        .set_data(to_binary(&ExecuteAnswer::CreateViewingKey {
            status: Success,
        })?))
}

/// Returns Result<Response, ContractError>
///
/// sets the viewing key
///
/// # Arguments
///
/// * `deps` - DepsMut containing all the contract's external dependencies
/// * `info` - Carries the info of who sent the message and how much native funds were sent along
/// * `key`  - string slice to be used as the viewing key
fn try_set_key(deps: DepsMut, info: MessageInfo, key: &str) -> Result<Response, ContractError> {
    ViewingKey::set(deps.storage, info.sender.as_str(), key);

    Ok(Response::new()
        .add_attribute("viewing_key", key)
        .set_data(to_binary(&ExecuteAnswer::SetViewingKey {
            status: Success,
        })?))
}

//An helper function
fn get_exp(
    current_block: u64,
    total_weight: u64,
    schedule: &Schedule,
    contract: VerifiedContract,
) -> u128 {
    let mut multiplier = 0;
    // Going serially assuming that schedule is not a big vector
    for u in schedule.to_owned() {
        if current_block >= u.start_block && contract.last_claimed < u.end_block {
            if current_block >= u.end_block {
                multiplier +=
                    (u.end_block - contract.last_claimed) as u128 * u.mint_per_block.u128();

                if contract.last_claimed < u.start_block {
                    multiplier -=
                        (u.start_block - contract.last_claimed) as u128 * u.mint_per_block.u128();
                }

                // last_update_block = u.end_block;
            } else {
                multiplier +=
                    (current_block - contract.last_claimed) as u128 * u.mint_per_block.u128();

                if contract.last_claimed < u.start_block {
                    multiplier -=
                        (u.start_block - contract.last_claimed) as u128 * u.mint_per_block.u128();
                }

                // last_update_block = current_block;
                // break; // No need to go further up the schedule
            }
        }
    }

    if total_weight.eq(&0u64) {
        return 0u128;
    }

    let xp = (multiplier * contract.weight as u128) / total_weight as u128;

    xp
}

/// Enforces that an address is a admin address.
/// Takes in a Deps instance, a Config struct, and an Address, and checks if the provided
/// address is an admin in the given Config. If the address is not an admin, returns a
/// ContractError with a message indicating that the provided address is not an admin.
///
/// # Arguments
///
/// * `deps` - Deps containing all the contract's external dependencies
/// * `config` - The Config struct to check for admins
/// * `address` - The Address to check if it is an admin
fn enforce_admin(deps: Deps, config: &Config, address: &Addr) -> Result<(), ContractError> {
    if !config.admins.contains(address) {
        return Err(error::ContractError::CustomError {
            val: format!("Not an admin: {}", address),
        });
    }

    Ok(())
}

// ---------------------------------------- QUERIES --------------------------------------

/// Returns QueryResult from validating a permit and then using its creator's address when
/// performing the specified query
///
/// # Arguments
///
/// * `deps` - a reference to Extern containing all the contract's external dependencies
/// * `permit` - the permit used to authentic the query
/// * `query` - the query to perform
fn permit_queries(
    deps: Deps,
    permit: Permit,
    query: QueryWithPermit,
) -> Result<Binary, ContractError> {
    // Validate permit content
    let config = CONFIG.load(deps.storage)?;

    let viewer = validate(
        deps,
        PREFIX_REVOKED_PERMITS.load(deps.storage).unwrap().as_str(),
        &permit,
        config.contract_address.to_string(),
        None,
    )?;

    let account = deps.api.addr_validate(&viewer)?;

    // Permit validated! We can now execute the query.
    match query {
        QueryWithPermit::UserExp { season } => {
            if !permit.check_permission(&TokenPermissions::Balance) {
                return Err(ContractError::Unauthorized {});
            }

            query_exp(deps, account, season)
        }
    }
}

pub fn viewing_keys_queries(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    let (address, key) = msg.get_validation_params();

    if !is_key_valid(deps.storage, &address, key) {
        Err(ContractError::Unauthorized {})
    } else {
        match msg {
            // Base
            QueryMsg::UserExp {
                address, season, ..
            } => query_exp(deps, address, season),
            //Only be checked by admin and verified contracts
            QueryMsg::CheckUserExp {
                user_address,
                address,
                key: _,
                season,
            } => {
                let config = CONFIG.load(deps.storage)?;
                // let admin_bool = config.admins.to_string() == address;

                let _res = enforce_admin(deps, &config, &address);

                let is_admin = _res.is_ok();

                let is_verified_contract = VERIFIED_CONTRACTS.has(deps.storage, &address);

                if !(is_admin || is_verified_contract) {
                    return Err(error::ContractError::Std(StdError::generic_err(format!(
                        "{} is not authorized to access this query",
                        address
                    ))));
                }

                query_exp(deps, user_address, season)
            }

            QueryMsg::GetWinner { no_of_winners, .. } => {
                let config = CONFIG.load(deps.storage)?;
                // let admin_bool = config.admins.to_string() == address;

                let _res = enforce_admin(deps, &config, &deps.api.addr_validate(&address)?);

                let is_admin = _res.is_ok();

                let is_grand_prize;

                if let Some(address) = config.grand_prize_contract {
                    is_grand_prize = address == address;
                } else {
                    is_grand_prize = false;
                }

                if !(is_admin || is_grand_prize) {
                    return Err(error::ContractError::Std(StdError::generic_err(format!(
                        "{} is not authorized to access this query",
                        address
                    ))));
                }

                query_get_winners(deps, env, no_of_winners)
            }

            QueryMsg::Contract { address, .. } => query_contract(deps, env, address),

            _ => panic!("This query type does not require authentication"),
        }
    }
}

fn try_get_winners(
    deps: Deps,
    env: Env,
    no_of_winners: Option<u64>,
) -> Result<Response, ContractError> {
    let winners = get_winners(deps, env, no_of_winners.unwrap_or_default())?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::GetWinners { winners })?))
}

fn query_contract_info(deps: Deps, env: Env) -> Result<Binary, ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let supply_pool = SUPPLY_POOL.load(deps.storage, config.season_counter)?;

    let info = ConfigRes {
        admins: config.admins,
        contract_address: config.contract_address,
        total_weight: config.total_weight,
        minting_schedule: config.minting_schedule,
        verified_contracts: config.verified_contracts,
        season_count: config.season_counter,
        season_starting_block: config.season_starting_block,
        season_ending_block: config.season_ending_block,
        season_duration: config.season_duration,
        season_total_xp_cap: supply_pool.season_total_xp_cap,
        current_block: env.block.height,
    };

    Ok(to_binary(&QueryAnswer::ContractInfoResponse { info })?)
}

fn query_verified_contracts(
    deps: Deps,
    _env: Env,
    start_page: Option<u32>,
    page_size: Option<u32>,
) -> Result<Binary, ContractError> {
    // Check for defaults
    let start = start_page.unwrap_or(0);
    let size = page_size.unwrap_or(5);
    let config = CONFIG.load(deps.storage)?;

    // Prep empty List of Listing Data for response
    let mut contract_list: Vec<VerifiedContractRes> = vec![];

    for contract in config
        .verified_contracts
        .iter()
        .skip((start as usize) * (size as usize))
        .take(size as usize)
    {
        let verf_contract = VERIFIED_CONTRACTS.load(deps.storage, &contract)?;
        contract_list.push(VerifiedContractRes {
            address: contract.clone(),
            available_xp: verf_contract.available_exp,
            weight: verf_contract.weight,
            last_claimed: verf_contract.last_claimed,
            code_hash: verf_contract.code_hash,
        });
    }

    Ok(to_binary(&QueryAnswer::VerifiedContractsResponse {
        contracts: contract_list,
    })?)
}

fn query_exp(deps: Deps, address: Addr, season: Option<u64>) -> Result<Binary, ContractError> {
    let exp: Uint128;
    let config = CONFIG.load(deps.storage)?;
    if EXP_ACCOUNTS.has(deps.storage, (&address, config.season_counter)) {
        exp = EXP_ACCOUNTS
            .load(
                deps.storage,
                (&address, season.unwrap_or(config.season_counter)),
            )
            .unwrap()
    } else {
        exp = Uint128::from(0_u128)
    }
    Ok(to_binary(&QueryAnswer::UserExp { exp })?)
}

fn query_contract(deps: Deps, env: Env, address: Addr) -> Result<Binary, ContractError> {
    let raw_contract = VERIFIED_CONTRACTS.load(deps.storage, &address);

    let contract;
    if raw_contract.is_ok() {
        contract = raw_contract.unwrap();
    } else {
        contract = VerifiedContract {
            available_exp: Uint128::default(),
            weight: 0,
            last_claimed: env.block.height,
            code_hash: String::new(),
            total_xp: Uint128::default(),
            xp_claimed: Uint128::default(),
        };
    }

    let config = CONFIG.load(deps.storage)?;

    let unclaimed_exp = Uint128::from(get_exp(
        env.block.height,
        config.total_weight,
        &config.minting_schedule,
        contract.clone(),
    ));

    Ok(to_binary(&QueryAnswer::ContractResponse {
        available_exp: contract.available_exp,
        unclaimed_exp,
        weight: contract.weight,
        last_claimed: contract.last_claimed,
        total_xp: contract.total_xp,
        xp_claimed: contract.xp_claimed,
    })?)
}

//----------------------------------------- Helper functions----------------------------------

/// Returns bool result of validating an address' viewing key
///
/// # Arguments
///
/// * `storage`     - a reference to the contract's storage
/// * `account`     - a reference to the str whose key should be validated
/// * `viewing_key` - String key used for authentication
fn is_key_valid(storage: &dyn Storage, account: &str, viewing_key: String) -> bool {
    ViewingKey::check(storage, account, &viewing_key).is_ok()
}

fn query_get_winners(
    deps: Deps,
    env: Env,
    no_of_winners: Option<u64>,
) -> Result<Binary, ContractError> {
    let winners = get_winners(deps, env, no_of_winners.unwrap_or_default())?;

    Ok(to_binary(&QueryAnswer::GetWinnersResponse { winners })?)
}

fn get_winners(deps: Deps, env: Env, no_of_winners: u64) -> Result<Vec<Addr>, ContractError> {
    let mut winners: Vec<Addr> = Vec::new();
    let mut winner_xp_indexed: Vec<u128> = Vec::new();

    let config = CONFIG.load(deps.storage)?;

    let xp_nonce = XP_NONCE
        .load(deps.storage, config.season_counter)
        .unwrap_or_default();

    if xp_nonce == Uint128::zero() {
        return Err(ContractError::CustomError {
            val: String::from("Not enough users to return a winner"),
        });
    }

    //run a loop for no_of_winners
    //search user with such xp.
    let len = XP_APPEND_STORE
        .add_suffix(&format!("{}", config.season_counter))
        .get_len(deps.storage)
        .unwrap_or_default();

    let mut min: u32 = 0;
    let mut max: u32 = len;

    for _ in 0..no_of_winners {
        let drafted_xp = calculate_drafted_xp(env.clone(), xp_nonce.u128());

        winner_xp_indexed.push(drafted_xp);

        while min <= max {
            let mid = (min + max) / 2;
            let value = XP_APPEND_STORE
                .add_suffix(&format!("{}", config.season_counter))
                .get_at(deps.storage, mid)
                .unwrap();

            if drafted_xp >= value.starting_slot.u128() && drafted_xp <= value.ending_slot.u128() {
                winners.push(value.user_address);
                break;
            } else if value.ending_slot < drafted_xp.into() {
                min = mid + 1;
            } else {
                max = mid - 1;
            }
        }
    }

    Ok(winners)
}

use std::convert::TryInto;

fn binary_to_u128(binary: &Binary) -> Option<u128> {
    let bytes = binary.0.as_slice();
    // Choosing the last 16 bytes for this example
    if bytes.len() >= 16 {
        Some(u128::from_be_bytes(
            bytes[(bytes.len() - 16)..].try_into().unwrap(),
        ))
    } else {
        Some(u128::from_be_bytes(bytes.try_into().unwrap()))
    }
}

fn calculate_drafted_xp(env: Env, xp_nonce: u128) -> u128 {
    if let Some(random_binary) = env.block.random {
        let random_number = binary_to_u128(&random_binary).unwrap(); // Adjust based on the chosen method
        return random_number % xp_nonce + 1;
    }
    0 // Fallback value
}
