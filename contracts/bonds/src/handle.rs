use chrono::prelude::*;
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{
    debug_print, from_binary, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse,
    HumanAddr, Querier, StdError, StdResult, Storage, Uint128 as prevUint128, WasmMsg,
};

use query_authentication::viewing_keys::ViewingKey;

use secret_toolkit::{
    snip20::{
        allowance_query, mint_msg, register_receive_msg, send_msg, token_info_query,
        transfer_from_msg, transfer_msg, Allowance,
    },
    utils::{HandleCallback, InitCallback, Query},
};

use shade_protocol::contract_interfaces::{
    airdrop::HandleMsg::CompleteTask,
    oracles::band::ReferenceData,
    oracles::oracle::QueryMsg::Price,
    snip20::{token_config_query, HandleMsg, Snip20Asset, TokenConfig},
};
use shade_protocol::contract_interfaces::{
    bonds::{
        errors::*,
        BondOpportunity, SlipMsg, {Account, AccountKey, Config, HandleAnswer, PendingBond},
    },
    snip20::fetch_snip20,
};
use shade_protocol::utils::asset::Contract;
use shade_protocol::utils::generic_response::ResponseStatus;

use std::{cmp::Ordering, convert::TryFrom, ops::Add};

use crate::state::{
    account_r, account_viewkey_w, account_w, allocated_allowance_r, allocated_allowance_w,
    allowance_key_r, allowance_key_w, bond_opportunity_r, bond_opportunity_w, collateral_assets_r,
    collateral_assets_w, config_r, config_w, global_total_claimed_r, global_total_claimed_w,
    global_total_issued_r, global_total_issued_w, issued_asset_r,
};

pub fn try_update_limit_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    limit_admin: Option<HumanAddr>,
    global_issuance_limit: Option<Uint128>,
    global_minimum_bonding_period: Option<u64>,
    global_maximum_discount: Option<Uint128>,
    reset_total_issued: Option<bool>,
    reset_total_claimed: Option<bool>,
) -> StdResult<HandleResponse> {
    let cur_config = config_r(&deps.storage).load()?;

    // Limit admin only
    if env.message.sender != cur_config.limit_admin {
        return Err(not_limit_admin());
    }

    let mut config = config_w(&mut deps.storage);
    config.update(|mut state| {
        if let Some(limit_admin) = limit_admin {
            state.limit_admin = limit_admin;
        }
        if let Some(global_issuance_limit) = global_issuance_limit {
            state.global_issuance_limit = global_issuance_limit;
        }
        if let Some(global_minimum_bonding_period) = global_minimum_bonding_period {
            state.global_minimum_bonding_period = global_minimum_bonding_period;
        }
        if let Some(global_maximum_discount) = global_maximum_discount {
            state.global_maximum_discount = global_maximum_discount;
        }
        Ok(state)
    })?;

    if let Some(reset_total_issued) = reset_total_issued {
        if reset_total_issued {
            global_total_issued_w(&mut deps.storage).save(&Uint128::zero())?;
        }
    }

    if let Some(reset_total_claimed) = reset_total_claimed {
        if reset_total_claimed {
            global_total_claimed_w(&mut deps.storage).save(&Uint128::zero())?;
        }
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateLimitConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    oracle: Option<Contract>,
    treasury: Option<HumanAddr>,
    activated: Option<bool>,
    issuance_asset: Option<Contract>,
    bond_issuance_limit: Option<Uint128>,
    bonding_period: Option<u64>,
    discount: Option<Uint128>,
    global_min_accepted_issued_price: Option<Uint128>,
    global_err_issued_price: Option<Uint128>,
    allowance_key: Option<String>,
) -> StdResult<HandleResponse> {
    let cur_config = config_r(&deps.storage).load()?;

    // Admin-only
    if !cur_config.admin.contains(&env.message.sender) {
        return Err(StdError::unauthorized());
    }

    if let Some(allowance_key) = allowance_key {
        allowance_key_w(&mut deps.storage).save(&allowance_key)?;
    };

    let mut config = config_w(&mut deps.storage);
    config.update(|mut state| {
        if let Some(oracle) = oracle {
            state.oracle = oracle;
        }
        if let Some(treasury) = treasury {
            state.treasury = treasury;
        }
        if let Some(activated) = activated {
            state.activated = activated;
        }
        if let Some(issuance_asset) = issuance_asset {
            state.issued_asset = issuance_asset;
        }
        if let Some(bond_issuance_limit) = bond_issuance_limit {
            state.bond_issuance_limit = bond_issuance_limit;
        }
        if let Some(bonding_period) = bonding_period {
            state.bonding_period = bonding_period;
        }
        if let Some(discount) = discount {
            state.discount = discount;
        }
        if let Some(global_min_accepted_issued_price) = global_min_accepted_issued_price {
            state.global_min_accepted_issued_price = global_min_accepted_issued_price;
        }
        if let Some(global_err_issued_price) = global_err_issued_price {
            state.global_err_issued_price = global_err_issued_price;
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

pub fn try_remove_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    admin_to_remove: HumanAddr
) -> StdResult<HandleResponse> {
    let mut config = config_r(&deps.storage).load()?;

    if env.message.sender != config.limit_admin {
        return Err(not_limit_admin())
    }

    // Retain only admin addresses that don't match the one to remove
    config.admin.retain(
    |admin| admin != &admin_to_remove,
    );

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RemoveAdmin { 
            status: ResponseStatus::Success, 
        })?),
    })
}

pub fn try_add_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    admin_to_add: HumanAddr
) -> StdResult<HandleResponse> {
    let mut config = config_r(&deps.storage).load()?;

    if env.message.sender != config.limit_admin {
        return Err(not_limit_admin())
    }

    // Add the new admin address
    config.admin.push(admin_to_add);
    
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RemoveAdmin { 
            status: ResponseStatus::Success, 
        })?),
    })
}

pub fn try_deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    sender: HumanAddr,
    _from: HumanAddr,
    deposit_amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    // Check that sender isn't the treasury
    if config.treasury == sender {
        return Err(blacklisted(config.treasury));
    }

    if config.contract == sender {
        return Err(blacklisted(config.contract));
    }

    // Check that sender isn't an admin
    if config.admin.contains(&sender) {
        return Err(blacklisted(sender));
    }

    // Check that sender isn't the minted asset
    if config.issued_asset.address == env.message.sender {
        return Err(issued_asset_deposit());
    }

    // Check that sender asset has an active bond opportunity
    let bond_opportunity = match bond_opportunity_r(&deps.storage)
        .may_load(env.message.sender.to_string().as_bytes())?
    {
        Some(prev_opp) => {
            bond_active(&env, &prev_opp)?;
            prev_opp
        }
        None => {
            return Err(no_bond_found(env.message.sender.as_str()));
        }
    };

    let available = bond_opportunity
        .issuance_limit
        .checked_sub(bond_opportunity.amount_issued)
        .unwrap();

    // Load mint asset information
    let issuance_asset = issued_asset_r(&deps.storage).load()?;

    // Calculate conversion of collateral to SHD
    let (amount_to_issue, deposit_price, claim_price, discount_price) = amount_to_issue(
        &deps,
        deposit_amount,
        available,
        bond_opportunity.deposit_denom.clone(),
        issuance_asset,
        bond_opportunity.discount,
        bond_opportunity.max_accepted_collateral_price,
        bond_opportunity.err_collateral_price,
        config.global_min_accepted_issued_price,
        config.global_err_issued_price,
    )?;

    if let Some(message) = msg {
        let msg: SlipMsg = from_binary(&message)?;

        // Check Slippage
        if amount_to_issue.clone() < msg.minimum_expected_amount.clone() {
            return Err(slippage_tolerance_exceeded(
                amount_to_issue,
                msg.minimum_expected_amount,
            ));
        }
    };

    bond_opportunity_w(&mut deps.storage).update(env.message.sender.to_string().as_bytes(), |opp| {
        let o = opp.unwrap();
        o.amount_issued.checked_add(amount_to_issue)?;
        Ok(o)
    })?;

    let mut messages = vec![];

    // Collateral to treasury
    messages.push(send_msg(
        config.treasury.clone(),
        prevUint128(deposit_amount.u128()),
        None,
        None,
        None,
        1,
        bond_opportunity.deposit_denom.contract.code_hash.clone(),
        bond_opportunity.deposit_denom.contract.address.clone(),
    )?);

    // Format end date as String
    let end: u64 = calculate_claim_date(env.block.time, bond_opportunity.bonding_period);

    // Begin PendingBond
    let new_bond = PendingBond {
        claim_amount: amount_to_issue.clone(),
        end_time: end,
        deposit_denom: bond_opportunity.deposit_denom,
        deposit_amount,
        deposit_price,
        claim_price,
        discount: bond_opportunity.discount,
        discount_price,
    };

    // Find user account, create if it doesn't exist
    let mut account = match account_r(&deps.storage).may_load(sender.as_str().as_bytes())? {
        None => {
            // Airdrop task
           if let Some(airdrop) = config.airdrop {
                    let msg = CompleteTask {
                        address: sender.clone(),
                        padding: None,
                    };
                    messages.push(msg.to_cosmos_msg(airdrop.code_hash, airdrop.address, None)?);
                }
            }

            Account {
                address: sender,
                pending_bonds: vec![],
            }
        }
        Some(acc) => acc,
    };

    // Add new_bond to user's pending_bonds Vec
    account.pending_bonds.push(new_bond.clone());

    // Save account
    account_w(&mut deps.storage).save(account.address.as_str().as_bytes(), &account)?;

    if !bond_opportunity.minting_bond {
        // Decrease AllocatedAllowance since user is claiming
        allocated_allowance_w(&mut deps.storage)
            .update(|allocated| Ok(allocated.checked_sub(amount_to_issue.clone())?))?;

        // Transfer funds using allowance to bonds
        messages.push(transfer_from_msg(
            config.treasury.clone(),
            env.contract.address.clone(),
            prevUint128(amount_to_issue.u128()),
            None,
            None,
            256,
            config.issued_asset.code_hash.clone(),
            config.issued_asset.address,
        )?);
    } else {
        messages.push(mint_msg(
            config.contract,
            prevUint128(amount_to_issue.u128()),
            None,
            None,
            256,
            config.issued_asset.code_hash,
            config.issued_asset.address,
        )?);
    }

    // Return Success response
    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Deposit {
            status: ResponseStatus::Success,
            deposit_amount: new_bond.deposit_amount,
            pending_claim_amount: new_bond.claim_amount,
            end_date: new_bond.end_time,
        })?),
    })
}

pub fn try_claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    // Check if bonding period has elapsed and allow user to claim
    // however much of the issuance asset they paid for with their deposit
    let config = config_r(&deps.storage).load()?;

    // Find user account, error out if DNE
    let mut account =
        match account_r(&deps.storage).may_load(env.message.sender.as_str().as_bytes())? {
            None => {
                return Err(StdError::NotFound {
                    kind: env.message.sender.to_string(),
                    backtrace: None,
                });
            }
            Some(acc) => acc,
        };

    // Bring up pending bonds structure for user if account is found
    let mut pending_bonds = account.pending_bonds;
    if pending_bonds.is_empty() {
        return Err(no_pending_bonds(account.address.as_str()));
    }

    // Set up loop comparison values.
    let now = env.block.time; // Current time in seconds
    let mut total = Uint128::zero();

    // Iterate through pending bonds and compare one's end to current time
    for bond in pending_bonds.iter() {
        if bond.end_time <= now {
            // Add claim amount to total
            total = total.checked_add(bond.claim_amount);
        }
    }

    // Add case for if total is 0, error out
    if total.is_zero() {
        return Err(no_bonds_claimable());
    }

    // Remove claimed bonds from vector and save back to the account
    pending_bonds.retain(
        |bond| bond.end_time > now, // Retain only the bonds that end at a time greater than now
    );

    account.pending_bonds = pending_bonds;
    account_w(&mut deps.storage).save(env.message.sender.as_str().as_bytes(), &account)?;

    global_total_claimed_w(&mut deps.storage)
        .update(|global_total_claimed| Ok(global_total_claimed.checked_add(total.clone())?))?;

    //Set up empty message vec
    let mut messages = vec![];

    // // Decide via config boolean whether or not the contract is a minting bond
    // if config.minting_bond {
    //     // Mint out the total using snip20 to the user
    //     messages.push(mint_msg(
    //         env.message.sender,
    //         total,
    //         None,
    //         None,
    //         256,
    //         config.issued_asset.code_hash.clone(),
    //         config.issued_asset.address,
    //     )?);
    // } else {
    messages.push(send_msg(
        env.message.sender,
        prevUint128(total.u128()),
        None,
        None,
        None,
        256,
        config.issued_asset.code_hash.clone(),
        config.issued_asset.address,
    )?);

    // Return Success response
    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Claim {
            status: ResponseStatus::Success,
            amount: total,
        })?),
    })
}

pub fn try_open_bond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    collateral_asset: Contract,
    start_time: u64,
    end_time: u64,
    bond_issuance_limit: Option<Uint128>,
    bonding_period: Option<u64>,
    discount: Option<Uint128>,
    max_accepted_collateral_price: Uint128,
    err_collateral_price: Uint128,
    minting_bond: bool,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    // Admin-only
    if !config.admin.contains(&env.message.sender) {
        return Err(StdError::unauthorized());
    };

    let mut messages = vec![];

    // Check whether previous bond for this asset exists
    match bond_opportunity_r(&deps.storage)
        .may_load(collateral_asset.address.as_str().as_bytes())?
    {
        Some(prev_opp) => {
            let unspent = prev_opp
                .issuance_limit
                .checked_sub(prev_opp.amount_issued)?;
            global_total_issued_w(&mut deps.storage)
                .update(|issued| Ok(issued.checked_sub(unspent.clone())?))?;

            if !prev_opp.minting_bond {
                // Unallocate allowance that wasn't issued

                allocated_allowance_w(&mut deps.storage)
                    .update(|allocated| Ok(allocated.checked_sub(unspent)?))?;
            }
        }
        None => {
            // Save to list of current collateral addresses
            match collateral_assets_r(&deps.storage).may_load()? {
                None => {
                    let assets = vec![collateral_asset.address.clone()];
                    collateral_assets_w(&mut deps.storage).save(&assets)?;
                },
                Some(_assets) => {
                    collateral_assets_w(&mut deps.storage).update(|mut assets| {
                        assets.push(collateral_asset.address.clone());
                        Ok(assets)
                    })?;
                }
            };

            // Prepare register_receive message for new asset
            messages.push(register_receive(&env, &collateral_asset)?);
        }
    };

    // Check optional fields, setting to config defaults if None
    let limit = bond_issuance_limit.unwrap_or(config.bond_issuance_limit);
    let period = bonding_period.unwrap_or(config.bonding_period);
    let discount = discount.unwrap_or(config.discount);

    check_against_limits(&deps, limit, period, discount)?;

    if !minting_bond {
        // Check bond issuance amount against snip20 allowance and allocated_allowance
        let snip20_allowance = allowance_query(
            &deps.querier,
            config.treasury,
            env.contract.address.clone(),
            allowance_key_r(&deps.storage).load()?.to_string(),
            1,
            config.issued_asset.code_hash,
            config.issued_asset.address,
        )?;

        let allocated_allowance = allocated_allowance_r(&deps.storage).load()?;
        // Declaring again so 1.0 Uint128 works
        let snip_allowance = Uint128::from(snip20_allowance.allowance.u128());

        // Error out if allowance doesn't allow bond opportunity
        if snip_allowance.checked_sub(allocated_allowance)? < limit {
            return Err(bond_issuance_exceeds_allowance(
                snip_allowance,
                allocated_allowance,
                limit,
            ));
        };

        // Increase stored allocated_allowance by the opportunity's issuance limit
        allocated_allowance_w(&mut deps.storage).update(|allocated| Ok(allocated.checked_add(limit)?))?;
    }

    let deposit_denom = fetch_snip20(&collateral_asset.clone(), &deps.querier)?;

    // Generate bond opportunity
    let bond_opportunity = BondOpportunity {
        issuance_limit: limit,
        deposit_denom,
        start_time,
        end_time,
        discount,
        bonding_period: period,
        amount_issued: Uint128::zero(),
        max_accepted_collateral_price,
        err_collateral_price,
        minting_bond,
    };

    // Save bond opportunity
    bond_opportunity_w(&mut deps.storage).save(
        collateral_asset.address.as_str().as_bytes(),
        &bond_opportunity,
    )?;

    // Increase global total issued by bond opportunity's issuance limit
    global_total_issued_w(&mut deps.storage)
        .update(|global_total_issued| Ok(global_total_issued.checked_add(bond_opportunity.issuance_limit)?))?;

    // Return Success response
    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::OpenBond {
            status: ResponseStatus::Success,
            deposit_contract: bond_opportunity.deposit_denom.contract,
            start_time: bond_opportunity.start_time,
            end_time: bond_opportunity.end_time,
            bond_issuance_limit: bond_opportunity.issuance_limit,
            bonding_period: bond_opportunity.bonding_period,
            discount: bond_opportunity.discount,
            max_accepted_collateral_price: bond_opportunity.max_accepted_collateral_price,
            err_collateral_price: bond_opportunity.err_collateral_price,
            minting_bond: bond_opportunity.minting_bond,
        })?),
    })
}

pub fn try_close_bond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    collateral_asset: Contract,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    // Admin-only
    if !config.admin.contains(&env.message.sender) {
        return Err(StdError::unauthorized());
    };

    // Check whether previous bond for this asset exists

    match bond_opportunity_r(&deps.storage)
        .may_load(collateral_asset.address.as_str().as_bytes())?
    {
        Some(prev_opp) => {
            bond_opportunity_w(&mut deps.storage)
                .remove(collateral_asset.address.as_str().as_bytes());

            // Remove asset from address list
            collateral_assets_w(&mut deps.storage).update(|mut assets| {
                assets.retain(|address| *address != collateral_asset.address);
                Ok(assets)
            })?;

            let unspent = prev_opp
                .issuance_limit
                .checked_sub(prev_opp.amount_issued)?;
            global_total_issued_w(&mut deps.storage)
                .update(|issued| Ok(issued.checked_sub(unspent.clone())?))?;

            if !prev_opp.minting_bond {
                // Unallocate allowance that wasn't issued

                allocated_allowance_w(&mut deps.storage)
                    .update(|allocated| Ok(allocated.checked_sub(unspent)?))?;
            }
        }
        None => {
            // Error out, no bond found with that deposit asset
            return Err(no_bond_found(collateral_asset.address.as_str()));
        }
    }

    let messages = vec![];

    // Return Success response
    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ClosedBond {
            status: ResponseStatus::Success,
            collateral_asset,
        })?),
    })
}

fn bond_active(env: &Env, bond_opp: &BondOpportunity) -> StdResult<()> {
    if bond_opp.amount_issued >= bond_opp.issuance_limit {
        return Err(bond_limit_reached(bond_opp.issuance_limit));
    }
    if bond_opp.start_time > env.block.time {
        return Err(bond_not_started(bond_opp.start_time, env.block.time));
    }
    if bond_opp.end_time < env.block.time {
        return Err(bond_ended(bond_opp.end_time, env.block.time));
    }
    Ok(())
}

fn check_against_limits<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    bond_limit: Uint128,
    bond_period: u64,
    bond_discount: Uint128,
) -> StdResult<bool> {
    let config = config_r(&deps.storage).load()?;
    // Check that global issuance limit won't be exceeded by this opportunity's limit
    let global_total_issued = global_total_issued_r(&deps.storage).load()?;
    let global_issuance_limit = config.global_issuance_limit;

    active(
        &config.activated,
        &config.global_issuance_limit,
        &global_total_issued,
    )?;

    if global_total_issued.checked_add(bond_limit)? > global_issuance_limit {
        return Err(bond_limit_exceeds_global_limit(
            global_issuance_limit,
            global_total_issued,
            bond_limit,
        ));
    } else if bond_period < config.global_minimum_bonding_period {
        return Err(bonding_period_below_minimum_time(
            bond_period,
            config.global_minimum_bonding_period,
        ));
    } else if bond_discount > config.global_maximum_discount {
        return Err(bond_discount_above_maximum_rate(
            bond_discount,
            config.global_maximum_discount,
        ));
    }
    Ok(true)
}

pub fn active(
    activated: &bool,
    global_issuance_limit: &Uint128,
    global_total_issued: &Uint128,
) -> StdResult<()> {
    // Error out if bond contract isn't active
    if !activated {
        return Err(contract_not_active());
    }

    // Check whether mint limit has been reached
    if global_total_issued >= global_issuance_limit {
        return Err(global_limit_reached(*global_issuance_limit));
    }

    Ok(())
}

pub fn amount_to_issue<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    collateral_amount: Uint128,
    available: Uint128,
    collateral_asset: Snip20Asset,
    issuance_asset: Snip20Asset,
    discount: Uint128,
    max_accepted_collateral_price: Uint128,
    err_collateral_price: Uint128,
    min_accepted_issued_price: Uint128,
    err_issued_price: Uint128,
) -> StdResult<(Uint128, Uint128, Uint128, Uint128)> {
    let mut disc = discount;
    let mut collateral_price = oracle(&deps, collateral_asset.token_info.symbol.clone())?; // Placeholder for Oracle lookup
    if collateral_price > max_accepted_collateral_price {
        if collateral_price > err_collateral_price {
            return Err(collateral_price_exceeds_limit(
                collateral_price.clone(),
                err_collateral_price.clone(),
            ));
        }
        collateral_price = max_accepted_collateral_price;
    }
    let mut issued_price = oracle(deps, issuance_asset.token_info.symbol.clone())?; // Placeholder for minted asset price lookup
    if issued_price < err_issued_price {
        return Err(issued_price_below_minimum(
            issued_price.clone(),
            err_issued_price.clone(),
        ));
    }
    if issued_price < min_accepted_issued_price {
        disc = Uint128::zero();
        issued_price = min_accepted_issued_price;
    }
    let (issued_amount, discount_price) = calculate_issuance(
        collateral_price.clone(),
        collateral_amount,
        collateral_asset.token_info.decimals,
        issued_price,
        issuance_asset.token_info.decimals,
        disc,
        min_accepted_issued_price,
    );
    if issued_amount > available {
        return Err(mint_exceeds_limit(issued_amount, available));
    }
    Ok((
        issued_amount,
        collateral_price,
        issued_price,
        discount_price,
    ))
}

pub fn calculate_issuance(
    collateral_price: Uint128,
    collateral_amount: Uint128,
    collateral_decimals: u8,
    issued_price: Uint128,
    issued_decimals: u8,
    discount: Uint128,
    min_accepted_issued_price: Uint128,
) -> (Uint128, Uint128) {
    // Math must be done in integers
    // collateral_decimals  = x
    // issued_decimals = y
    // collateral_price     = p1 * 10^18
    // issued_price = p2 * 10^18
    // collateral_amount    = a1 * 10^x
    // issued_amount       = a2 * 10^y
    // discount            = d1 * 10^18

    // (a1 * 10^x) * (p1 * 10^18) = (a2 * 10^y) * (p2 * 10^18) * ((100 - d1) * 10^16)

    //                             (p1 * 10^18)
    // (a1 * 10^x) * ------------------------------------ = (a2 * 10^y)
    //                      (p2 * 10^18) * ((100 - d1))
    let percent_disc = Uint128::new(100_000u128).checked_sub(discount); // - discount.multiply_ratio(1000u128, 1_000_000_000_000_000_000u128).u128();
    let mut discount_price = issued_price.multiply_ratio(percent_disc, 100000u128);
    if discount_price < min_accepted_issued_price {
        discount_price = min_accepted_issued_price
    }
    let issued_amount = collateral_amount.multiply_ratio(collateral_price, discount_price);
    let difference: i32 = issued_decimals as i32 - collateral_decimals as i32;
    match difference.cmp(&0) {
        Ordering::Greater => (
            Uint128::from(issued_amount.u128() * 10u128.pow(u32::try_from(difference).unwrap())),
            discount_price,
        ),
        Ordering::Less => (
            issued_amount
                .multiply_ratio(1u128, 10u128.pow(u32::try_from(difference.abs()).unwrap())),
            discount_price,
        ),
        Ordering::Equal => (issued_amount, discount_price),
    }
}

pub fn calculate_claim_date(env_time: u64, bonding_period: u64) -> u64 {
    // Previously, translated the passed u64 as days and converted to seconds.
    // Now, however, it treats the passed value as seconds, due to that being
    // how the block environment tracks it.
    let end = env_time.checked_add(bonding_period).unwrap();

    end
}

pub fn register_receive(env: &Env, contract: &Contract) -> StdResult<CosmosMsg> {
    register_receive_msg(
        env.contract_code_hash.clone(),
        None,
        256,
        contract.code_hash.clone(),
        contract.address.clone(),
    )
}

pub fn oracle<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    symbol: String,
) -> StdResult<Uint128> {
    let config: Config = config_r(&deps.storage).load()?;
    let answer: ReferenceData = Price { symbol }.query(
        &deps.querier,
        config.oracle.code_hash,
        config.oracle.address,
    )?;
    Ok(Uint128::from(answer.rate))
}
