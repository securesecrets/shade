use chrono::prelude::*;
use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse,
    HumanAddr, Querier, StdError, StdResult, Storage, Uint128,
};

use query_authentication::viewing_keys::ViewingKey;

use secret_toolkit::{
    snip20::{token_info_query, register_receive_msg, send_msg, mint_msg, transfer_from_msg, allowance_query, Allowance},
    utils::Query,
};

use shade_protocol::bonds::{
    errors::*,
    {Config, HandleAnswer, PendingBond, Account, AccountKey}, BondOpportunity};
use shade_protocol::utils::generic_response::ResponseStatus;
use shade_protocol::utils::asset::Contract;
use shade_protocol::{
    snip20::{token_config_query, Snip20Asset, TokenConfig, QueryMsg::Allowance as QueryAllowance},
    oracle::QueryMsg::Price,
    band::ReferenceData,
};

use std::{cmp::Ordering, convert::TryFrom, ops::Add};

use crate::state::{config_r, config_w, collateral_assets_r, collateral_assets_w, 
    issued_asset_r, global_issuance_limit_r, global_total_issued_r, global_total_issued_w,
    bond_total_issued_r, bond_total_issued_w, account_r, account_w,
    bond_opportunity_r, bond_opportunity_w, account_viewkey_w, global_issuance_limit_w,
    global_total_claimed_r, global_total_claimed_w, allocated_allowance_r, allocated_allowance_w};

pub fn try_update_limit_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    limit_admin: Option<HumanAddr>,
    global_issuance_limit: Option<Uint128>,
    global_minimum_bonding_period: Option<u64>,
    global_maximum_discount: Option<Uint128>,
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

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })

}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    admin: Option<HumanAddr>,
    oracle: Option<Contract>,
    treasury: Option<HumanAddr>,
    activated: Option<bool>,
    issuance_asset: Option<Contract>,
    minting_bond: Option<bool>,
    bond_issuance_limit: Option<Uint128>,
    bonding_period: Option<u64>,
    discount: Option<Uint128>,
) -> StdResult<HandleResponse> {
    let cur_config = config_r(&deps.storage).load()?;

    // Admin-only
    if env.message.sender != cur_config.admin {
        return Err(StdError::unauthorized());
    }

    let mut config = config_w(&mut deps.storage);
    config.update(|mut state| {
        if let Some(admin) = admin {
            state.admin = admin;
        }
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
        if let Some(minting_bond) = minting_bond {
            state.minting_bond = minting_bond;
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
/*
// Register an asset before receiving it as user deposit
pub fn try_register_bond_opportunity<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    contract: &Contract,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;
    // Check if admin
    if env.message.sender != config.admin {
        return Err(StdError::Unauthorized {backtrace: None });
    }
    
    // Check if contract is activated
    if !config.activated {
        return Err(StdError::Unauthorized {backtrace: None });
    }

    // Storing Snip20 contract as key for bucket
    let contract_str = contract.address.to_string();

    // Adding the Snip20Asset to the contract's storage
    // First acquiring TokenInfo
    let asset_info = token_info_query(
        &deps.querier,
        1,
        contract.code_hash.clone(),
        contract.address.clone(),
    )?;

    // Acquiring TokenConfig
    let asset_config: Option<TokenConfig> = 
        match token_config_query(&deps.querier, contract.clone()) {
            Ok(c) => Option::from(c),
            Err(_) => None,
        };

    // Saving Snip20Asset with contract, TokenInfo, and TokenConfig copies
    debug_print!("Registering {}", asset_info.symbol);
    collateral_assets_w(&mut deps.storage).save(
        contract_str.as_bytes(),
        &Snip20Asset {
            contract: contract.clone(),
            token_info: asset_info,
            token_config: asset_config,
        },
    )?;

    // Enact register receive so funds sent to Bonds will call Receive
    let messages = vec![register_receive(env, contract)?];

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RegisterCollateralAsset {
            status: ResponseStatus::Success,
        })?),
    })
}
*/

/*
pub fn try_remove_bond_opportunity<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    address: HumanAddr,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;
    // Check if admin
    if env.message.sender != config.admin{
        return Err(StdError::Unauthorized {backtrace: None})
    }

    let address_str = address.to_string();

    // Remove asset from the collateral assets list
    collateral_assets_w(&mut deps.storage).remove(address_str.as_bytes());

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RemoveCollateralAsset {
            status: ResponseStatus::Success,
        })?),
    })
}
*/

pub fn try_deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    sender: HumanAddr,
    _from: HumanAddr,
    deposit_amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<HandleResponse>{
    // Check if limit hasn't been reached and that contract is activated
    let config = config_r(&deps.storage).load()?;
    let global_total_issued = global_total_issued_r(&deps.storage).may_load()?;
    active(&config.activated, &config.global_issuance_limit, &global_total_issued.unwrap())?;

    // Check that sender isn't the treasury
    if config.treasury == sender {
        return Err(StdError::generic_err(
            "Sender cannot be the treasury.",
        ));
    }

    // Check that sender isn't the minted asset
    if config.issued_asset.address == env.message.sender {
        return Err(StdError::generic_err(
            "Collateral asset cannot be the same as the minted asset."
        ));
    }

    // Check that sender asset has an active bond opportunity
    let bond_opportunity = 
        match bond_opportunity_r(&deps.storage).may_load(env.message.sender.to_string().as_bytes())?{
            Some(prev_opp) => {
                debug_print!(
                    "Found Previous Bond Opportuntiy: {} {}",
                    &prev_opp.deposit_denom.token_info.symbol,
                    prev_opp.deposit_denom.contract.address.to_string()
                );
                bond_active(&env, &prev_opp)?;
                prev_opp
            }
            None => {
                return Err(no_bond_found(env.message.sender.as_str()));
            }
        };
    /*
    let collateral_asset = 
        match collateral_assets_r(&deps.storage).may_load(env.message.sender.to_string().as_bytes())?{
           Some(supported_asset) => {
                debug_print!(
                    "Found Collateral Asset: {} {}",
                    &supported_asset.token_info.symbol,
                    env.message.sender.to_string()
                );
                supported_asset
            }
            None => {
                return Err(StdError::NotFound {
                    kind: env.message.sender.to_string(),
                    backtrace: None,
                });
            }
        };
    */

    let available = (bond_opportunity.issuance_limit - bond_opportunity.amount_issued).unwrap();
    
    // Load mint asset information
    let issuance_asset = issued_asset_r(&deps.storage).load()?;
    // Calculate conversion of collateral to SHD
    let amount_to_issue = amount_to_issue(&deps, deposit_amount, available, bond_opportunity.deposit_denom.clone(), issuance_asset, bond_opportunity.discount).unwrap();
    
    // Add to total issued, globally and bond opportunity-specific
    global_total_issued_w(&mut deps.storage).update(|global_total_issued| {
        Ok(global_total_issued + amount_to_issue)
    })?;

    bond_opportunity_w(&mut deps.storage).update(env.message.sender.to_string().as_bytes(), |prev_bond_opportunity| match prev_bond_opportunity {
        Some(mut prev_bond_opportunity) => {
            prev_bond_opportunity.amount_issued = prev_bond_opportunity.amount_issued + amount_to_issue;
            Ok(prev_bond_opportunity)
        }
        None => {
            return Err(no_bond_found(env.message.sender.as_str()));
        }
    })?;

    let mut messages = vec![];

    // Collateral to treasury
    messages.push(send_msg(
        config.treasury,
        deposit_amount,
        None,
        None,
        None,
        1,
        bond_opportunity.deposit_denom.contract.code_hash.clone(),
        bond_opportunity.deposit_denom.contract.address.clone(),
    )?);

    // Format end date (7 days from now) as String
    let end: u64 = calculate_claim_date(&env, bond_opportunity.bonding_period);
    
    // Begin PendingBond
    let new_bond = PendingBond{
        claim_amount: amount_to_issue,
        end: end,
        deposit_denom: bond_opportunity.deposit_denom,
        deposit_amount,
    };

    // Find user account, create if it doesn't exist
    let mut account = match account_r(&deps.storage).may_load(sender.as_str().as_bytes())? {
        None => {
            let account = Account {
                address: sender,
                pending_bonds: vec![],
            };
            account
        }
        Some(acc) => {
            acc
        }
    };

    // Add new_bond to user's pending_bonds Vec
    account.pending_bonds.push(new_bond.clone());

    // Save account
    account_w(&mut deps.storage).save(account.address.as_str().as_bytes(), &account)?;

    // Return Success response
    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Deposit {
            status: ResponseStatus::Success,
            deposit_amount: new_bond.deposit_amount,
            pending_claim_amount: new_bond.claim_amount,
            end_date: new_bond.end, 
        })?),
    })
}

pub fn try_claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    //TODO, should check if bonding period has elapsed and allow user to claim
    //however much of the issuance asset they paid for with their deposit
    let config = config_r(&deps.storage).load()?;

    // Find user account, error out if DNE
    let account = match account_r(&deps.storage).may_load(env.message.sender.as_str().as_bytes())? {
        None => {
            return Err(StdError::NotFound {
                kind: env.message.sender.to_string(),
                backtrace: None,
            });
        }
        Some(acc) => {
            acc
        }
    };

    // Bring up pending bonds structure for user if account is found
    let mut pending_bonds = account.pending_bonds;
    if pending_bonds.is_empty(){
        return Err(no_pending_bonds(account.address.as_str()))
    }

    // Set up loop comparison values.
    let now = env.block.time * 24u64 * 60u64 * 60u64; // Current time in seconds
    let mut total = Uint128(0);

    // Iterate through pending bonds and compare one's end to current time
    let pending_bonds_iter = pending_bonds.iter();
    for bond in pending_bonds_iter{
        if bond.end <= now {                // Add claim amount to total
            total = total.add(bond.claim_amount);
        }
    }

    // Remove claimed bonds from vector
    pending_bonds.retain(|bond|
        bond.end > now  // Retain only the bonds that end at a time greater than now
    );

    // Add total to running total of amount claimed, globally
    let mut global_total_claimed = global_total_claimed_r(&deps.storage).load().unwrap();
    global_total_claimed += total.clone();
    global_total_claimed_w(&mut deps.storage).save(&global_total_claimed)?;

    //Set up empty message vec
    let mut messages = vec![];

    // Decide via config boolean whether or not the contract is a minting bond
    if config.minting_bond {
        // Mint out the total using snip20 to the user
        messages.push(mint_msg(
            env.message.sender,
            total,
            None,
            None,
            256,
            config.issued_asset.code_hash.clone(),
            config.issued_asset.address,
        )?);
    } else {
        // Decrease AllocatedAllowance since user is claiming
        allocated_allowance_w(&mut deps.storage).update(|allocated| allocated - total)?;

        // Transfer funds using allowance to the user
        messages.push(transfer_from_msg(
            config.treasury,
            env.message.sender,
            total,
            None,
            None,
            256,
            config.issued_asset.code_hash.clone(),
            config.issued_asset.address,
        )?);
    }


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
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    // Admin-only
    if env.message.sender != config.admin {
        return Err(StdError::unauthorized());
    };

    // Check optional fields, setting to config defaults if None
    let limit = bond_issuance_limit.unwrap_or(config.bond_issuance_limit);
    let period = bonding_period.unwrap_or(config.bonding_period);
    let discount = discount.unwrap_or(config.discount);

    check_against_limits(&deps, limit, period, discount)?;

    // Check bond issuance amount against snip20 allowance and allocated_allowance
    let snip20_allowance = allowance_query(
        &deps.querier,
        config.treasury, 
        env.contract.address.clone(),
        "asdf".to_string(),
        1,
        config.issued_asset.code_hash,
        config.issued_asset.address,
    )?;

    let allocated_allowance = allocated_allowance_r(&deps.storage).load().unwrap();

    // Error out if allowance doesn't allow bond opportunity
    if (snip20_allowance.allowance - allocated_allowance).unwrap() < limit {
        return Err(bond_issuance_exceeds_allowance(snip20_allowance.allowance, allocated_allowance, limit));
    };

    // Increase stored allocated_allowance by the opportunity's issuance limit
    allocated_allowance_w(&mut deps.storage).update(|allocated| {
        Ok(allocated + limit)
    })?;

    // Acquiring TokenInfo
    let asset_info = token_info_query(
        &deps.querier,
        1,
        collateral_asset.code_hash.clone(),
        collateral_asset.address.clone(),
    )?;

    // Acquiring TokenConfig
    let asset_config: Option<TokenConfig> = 
        match token_config_query(&deps.querier, collateral_asset.clone()) {
            Ok(c) => Option::from(c),
            Err(_) => None,
        };

    let deposit_denom = Snip20Asset {
        contract: collateral_asset.clone(),
        token_info: asset_info,
        token_config: asset_config,
    };

    let mut messages = vec![];

    // Check whether previous bond for this asset exists
    match bond_opportunity_r(&deps.storage).may_load(collateral_asset.address.as_str().as_bytes())?{
        Some(prev_opp) => {
            debug_print!(
                "Found Previous Bond Opportuntiy: {} {}",
                &prev_opp.deposit_denom.token_info.symbol,
                prev_opp.deposit_denom.contract.address.to_string()
            );
        }
        None => {
            // Save to list of current collateral addresses
            collateral_assets_w(&mut deps.storage).update(|mut assets|{
                assets.push(collateral_asset.address.clone());
                Ok(assets)
            })?;

            // Prepare register_receive message for new asset
            messages.push(register_receive(&env, &collateral_asset)?);
        }
    };
    
    // Generate bond opportunity
    let bond_opportunity = BondOpportunity {
        issuance_limit: limit,
        deposit_denom: deposit_denom,
        start_time,
        end_time,
        discount: discount,
        bonding_period: period,  
        amount_issued: Uint128(0),                 
    };
    
    // Save bond opportunity
    bond_opportunity_w(&mut deps.storage).save(collateral_asset.address.as_str().as_bytes(), &bond_opportunity)?;
    
    // Increase global total issued by bond opportunity's issuance limit
    let mut global_total_issued = global_total_issued_r(&deps. storage).load().unwrap();
    global_total_issued += bond_opportunity.issuance_limit;
    global_total_issued_w(&mut deps.storage).save(&global_total_issued)?;

    // Try with update instead
    //let mut global_update_issued = global_issuance_limit_w(&mut deps.storage).update(bond_opportunity.issuance_limit);

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
    if env.message.sender != config.admin {
        return Err(StdError::unauthorized());
    };

    // Check whether previous bond for this asset exists

    match bond_opportunity_r(&deps.storage).may_load(collateral_asset.address.as_str().as_bytes())?{
        Some(prev_opp) => {
            debug_print!(
                "Found Previous Bond Opportuntiy: {} {}",
                &prev_opp.deposit_denom.token_info.symbol,
                prev_opp.deposit_denom.contract.address.to_string()
            );
            bond_opportunity_w(&mut deps.storage).remove(collateral_asset.address.as_str().as_bytes());
            
            //let collateral_addresses = collateral_assets_r(&deps.storage).load()?;
            //let index = collateral_addresses.iter().position(|&x| x == collateral_asset.address).unwrap();
            //collateral_addresses.swap_remove(index);
            //collateral_assets_w(&mut deps.storage).save(&collateral_addresses)?;
            
            // Remove asset from address list
            collateral_assets_w(&mut deps.storage).update(|mut assets|{
                assets.retain(|address| *address != collateral_asset.address);
                Ok(assets)
            })?;

            // Unallocate allowance that wasn't issued
            let unspent = (prev_opp.issuance_limit - prev_opp.amount_issued)?;
            allocated_allowance_w(&mut deps.storage).update(|allocated| {
                Ok((allocated - unspent)?)
            })?;
        }
        None => {   // Error out, no bond found with that deposit asset
            return Err(no_bond_found(collateral_asset.address.as_str()))           
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

pub fn try_set_viewing_key<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    key: String,
) -> StdResult<HandleResponse> {
    account_viewkey_w(&mut deps.storage).save(
        &env.message.sender.to_string().as_bytes(),
        &AccountKey(key).hash(),
    )?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetViewingKey {
            status: ResponseStatus::Success,
        })?),
    })
}

fn bond_active(env: &Env, bond_opp: &BondOpportunity) -> StdResult<()> {
    if bond_opp.amount_issued >= bond_opp.issuance_limit {
        return Err(bond_limit_reached(bond_opp.issuance_limit))
    }
    if bond_opp.start_time < env.block.time {
        return Err(bond_not_started(bond_opp.start_time, env.block.time))
    }
    if bond_opp.end_time < env.block.time {
        return Err(bond_ended(bond_opp.end_time, env.block.time))
    }
    Ok(())
}

fn check_against_limits<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>, 
    bond_limit: Uint128, 
    bond_period: u64, 
    bond_discount: Uint128,
) -> StdResult<bool> {
    let config = config_r(&deps.storage).load().unwrap();
    // Check that global issuance limit won't be exceeded by this opportunity's limit
    let global_total_issued = global_total_issued_r(&deps.storage).load().unwrap();
    let global_issuance_limit = global_issuance_limit_r(&deps.storage).load().unwrap();
    if global_total_issued + bond_limit > global_issuance_limit {
        return Err(bond_limit_exceeds_global_limit(global_issuance_limit, global_total_issued, bond_limit))
    }
    else if bond_period < config.global_minimum_bonding_period {
        return Err(bonding_period_below_minimum_time(bond_period, config.global_minimum_bonding_period))
    }
    else if bond_discount > config.global_maximum_discount {
        return Err(bond_discount_above_maximum_rate(bond_discount, config.global_maximum_discount))
    }
    Ok(true)
} 

pub fn active(activated: &bool, global_issuance_limit: &Uint128, global_total_issued: &Uint128) -> StdResult<()> {
    // Error out if bond contract isn't active
    if !activated {
        return Err(contract_not_active());
    }

    // Check whether mint limit has been reached
    if global_total_issued >= global_issuance_limit {
        return Err(global_limit_reached(*global_issuance_limit))
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
) -> StdResult<Uint128> {
    let collateral_price = oracle(&deps, collateral_asset.token_info.symbol.clone())?;// Placeholder for Oracle lookup
    let issued_price = oracle(deps, issuance_asset.token_info.symbol.clone())?; // Placeholder for minted asset price lookup
    let issued_amount = calculate_issuance(
        collateral_price, 
        collateral_amount,
        collateral_asset.token_info.decimals,
        issued_price,
        issuance_asset.token_info.decimals,
        discount,
    );
    if issued_amount > available {
        return Err(mint_exceeds_limit(issued_amount, available))
    }
    Ok(issued_amount)
}

pub fn calculate_issuance(
    collateral_price: Uint128,
    collateral_amount: Uint128,
    collateral_decimals: u8,
    issued_price: Uint128,
    issued_decimals: u8,
    discount: Uint128,
) -> Uint128 {
    // Math must be done in integers
    // collateral_decimals  = x
    // issued_decimals = y
    // collateral_price     = p1 * 10^18
    // issued_price = p2 * 10^18
    // collateral_amount    = a1 * 10^x
    // issued_amount       = a2 * 10^y

    // (a1 * 10^x) * (p1 * 10^18) = (a2 * 10^y) * (p2 * 10^18)

    //                (p1 * 10^18)
    // (a1 * 10^x) * --------------  = (a2 * 10^y)
    //                (p2 * 10^18)
    let issued_amount = collateral_amount.multiply_ratio(collateral_price, issued_price);
    let difference: i32 = issued_decimals as i32 - collateral_decimals as i32;

    match difference.cmp(&0) {
        Ordering::Greater => {
            Uint128(issued_amount.u128() * 10u128.pow(u32::try_from(difference).unwrap()))
        }
        Ordering::Less => {
            issued_amount.multiply_ratio(1u128, 10u128.pow(u32::try_from(difference.abs()).unwrap()))
        }
        Ordering::Equal => issued_amount,
    }
}

pub fn calculate_claim_date(
    env: &Env,
    bonding_period: u64,
) -> u64 {
    //let naive = NaiveDateTime::from_timestamp(env.block.time as i64, 0);
    //let now: DateTime<Utc> = DateTime::from_utc(naive, Utc);
    // Take now, add bonding_period, save as end_time
    //let bond_duration: Duration = Duration::days(bonding_period as i64);
    //let end: DateTime<Utc> = now.add(bond_duration);

    // Attempt at a block time implementation instead
    let delay = bonding_period.checked_mul(24u64 * 60u64 * 60u64).unwrap();
    let end = env.block.time.checked_add(delay).unwrap();

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

fn oracle<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    symbol: String,
) -> StdResult<Uint128> {
    let config: Config = config_r(&deps.storage).load()?;
    let answer: ReferenceData = Price { symbol }.query(
        &deps.querier,
        config.oracle.code_hash,
        config.oracle.address,
    )?;
    Ok(answer.rate)
}