use shade_protocol::c_std::{
    self,
    from_binary,
    to_binary,
    Api,
    Binary,
    CosmosMsg,
    Env,
    DepsMut,
    Response,
    Addr,
    Querier,
    StdError,
    StdResult,
    Storage,
    Uint128,
    WasmMsg,
};
use shade_protocol::{
    snip20::helpers::{
        allowance_query,
        batch::SendFromAction,
        balance_query,
        batch_send_from_msg,
        batch_send_msg,
        decrease_allowance_msg,
        increase_allowance_msg,
        register_receive,
        send_msg,
        set_viewing_key_msg,
    },
};

use shade_protocol::{
    contract_interfaces::{
        dao::treasury_manager::{
            Allocation,
            AllocationMeta,
            AllocationType,
            Config,
            HandleAnswer,
            QueryAnswer,
            Holder,
            Balance,
            Status,
        },
        snip20,
    },
    utils::{asset::Contract, generic_response::ResponseStatus},
};

use crate::{
    query,
    state::{
        allocations_r,
        allocations_w,
        asset_list_r,
        asset_list_w,
        assets_r,
        assets_w,
        config_r,
        config_w,
        viewing_key_r,
        holder_r, holder_w,
        holders_r, holders_w,
        self_address_r,
    },
};
use chrono::prelude::*;
use shade_protocol::contract_interfaces::dao::adapter;
use std::convert::TryFrom;

pub fn receive<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    _sender: Addr,
    from: Addr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<Response> {

    /* TODO
     * All assets received from a "holder" will be credited to their account
     * All other assets from all other addresses will be credited to the treasury (default account)
     */

    let config = config_r(&deps.storage).load()?;
    let asset = assets_r(&deps.storage).load(info.sender.to_string().as_bytes())?;

    // Is Valid Holder
    if holders_r(&deps.storage).load()?.contains(&from) {
        // Update holdings
        holder_w(deps.storage).update(from.as_str().as_bytes(), |h| {
            let mut holder = h.unwrap();
            if let Some(i) = holder.balances.iter().position(|b| b.token == asset.contract.address) {
                holder.balances[i].amount += amount;
            }
            else {
                holder.balances.push(
                    Balance {
                        token: asset.contract.address,
                        amount: amount,
                    }
                );
            }
            Ok(holder)
        })?;
    } else {
        // Default to treasury
        // TODO: treasury balances need to update on allowance pull, as well as revenue 
        // rev-share design pending, something like 1% to rewards 
        holder_w(deps.storage).update(config.treasury.as_str().as_bytes(), |h| {
            let mut holder = h.unwrap();
            if let Some(i) = holder.balances.iter_mut().position(|b| b.token == asset.contract.address) {
                holder.balances[i].amount += amount;
            }
            else {
                holder.balances.push(
                    Balance {
                        token: asset.contract.address,
                        amount: amount,
                    }
                );
            }
            Ok(holder)
        })?;
    }

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Receive {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    config: Config,
) -> StdResult<Response> {
    let cur_config = config_r(&deps.storage).load()?;

    if info.sender != cur_config.admin {
        return Err(StdError::unauthorized());
    }

    config_w(deps.storage).save(&config)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_register_asset<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: &Env,
    contract: &Contract,
) -> StdResult<Response> {
    let config = config_r(&deps.storage).load()?;

    if info.sender != config.admin {
        return Err(StdError::unauthorized());
    }

    asset_list_w(deps.storage).update(|mut list| {
        list.push(contract.address.clone());
        Ok(list)
    })?;

    assets_w(deps.storage).save(
        contract.address.to_string().as_bytes(),
        &snip20::helpers::fetch_snip20(contract, &deps.querier)?,
    )?;

    allocations_w(deps.storage).save(contract.address.as_str().as_bytes(), &Vec::new())?;

    Ok(Response {
        messages: vec![
            // Register contract in asset
            register_receive(
                env.contract_code_hash.clone(),
                None,
                contract
            )?,
            // Set viewing key
            set_viewing_key_msg(
                viewing_key_r(&deps.storage).load()?,
                None,
                256,
                contract.code_hash.clone(),
                contract.address.clone(),
            )?,
        ],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RegisterAsset {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn allocate<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: &Env,
    asset: Addr,
    allocation: Allocation,
) -> StdResult<Response> {
    static ONE_HUNDRED_PERCENT: u128 = 10u128.pow(18);

    let config = config_r(&deps.storage).load()?;

    /* ADMIN ONLY */
    if info.sender != config.admin {
        return Err(StdError::unauthorized());
    }

    let key = asset.as_str().as_bytes();

    let mut apps = allocations_r(&deps.storage)
        .may_load(key)?
        .unwrap_or_default();

    let stale_alloc = apps
        .iter()
        .position(|a| a.contract.address == allocation.contract.address);

    match stale_alloc {
        Some(i) => {
            apps.remove(i);
        }
        None => {}
    };

    apps.push(AllocationMeta {
        nick: allocation.nick,
        contract: allocation.contract,
        amount: allocation.amount,
        alloc_type: allocation.alloc_type,
        balance: Uint128::zero(),
        tolerance: allocation.tolerance,
    });

    if (apps
        .iter()
        .map(|a| {
            if a.alloc_type == AllocationType::Portion {
                a.amount.u128()
            } else {
                0
            }
        })
        .sum::<u128>())
        > ONE_HUNDRED_PERCENT
    {
        return Err(StdError::generic_err(
            "Invalid allocation total exceeding 100%",
        ));
    }

    allocations_w(deps.storage).save(key, &apps)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Allocate {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn claim<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: &Env,
    asset: Addr,
) -> StdResult<Response> {

    if !asset_list_r(&deps.storage).load()?.contains(&asset) {
        return Err(StdError::generic_err("Unrecognized asset"));
    }
    let full_asset = assets_r(&deps.storage).load(asset.to_string().as_bytes())?;

    let config = config_r(&deps.storage).load()?;
    let mut claimer = info.sender.clone();

    if claimer == config.admin {
        claimer = config.treasury;
    }
    let holders = holders_r(&deps.storage).load()?;

    if !holders.contains(&claimer) {
        return Err(StdError::unauthorized());
    }

    let holder = holder_r(&deps.storage).load(&claimer.as_str().as_bytes())?;
    let mut unbonding = holder.unbondings.iter().find(|u| u.token == asset).unwrap();

    let mut reserves = balance_query(
        &deps.querier,
        self_address_r(&deps.storage).load()?,
        viewing_key_r(&deps.storage).load()?,
        1,
        full_asset.contract.code_hash.clone(),
        full_asset.contract.address.clone(),
    )?.amount;

    let mut messages = vec![];
    let mut total_claimed = Uint128::zero();

    // Claim if more funds are needed
    if unbonding.amount > reserves {
        let mut claim_amount = (unbonding.amount - reserves)?;

        for alloc in allocations_r(&deps.storage).load(asset.to_string().as_bytes())? {
            if claim_amount == Uint128::zero() {
                break;
            }

            let claim = adapter::claimable_query(deps, &asset.clone(), alloc.contract.clone())?;

            if claim > Uint128::zero() {
                messages.push(adapter::claim_msg(asset.clone(), alloc.contract)?);
                claim_amount = (claim_amount - claim)?;
                total_claimed += claim;
            }
        }
    }

    // Send claimed funds
    messages.push(
        send_msg(
            claimer.clone(),
            reserves + total_claimed,
            None,
            None,
            None,
            1,
            full_asset.contract.code_hash.clone(),
            full_asset.contract.address.clone(),
        )?
    );

    Ok(Response {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Claim {
            status: ResponseStatus::Success,
            amount: reserves + total_claimed,
        })?),
    })
}

pub fn update<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: &Env,
    asset: Addr,
) -> StdResult<Response> {
    let config = config_r(&deps.storage).load()?;

    let full_asset = assets_r(&deps.storage).load(asset.to_string().as_bytes())?;

    let mut allocations = allocations_r(deps.storage).load(asset.to_string().as_bytes())?;

    // Build metadata
    let mut amount_total = Uint128::zero();
    let mut portion_total = Uint128::zero();

    for i in 0..allocations.len() {
        match allocations[i].alloc_type {
            AllocationType::Amount => amount_total += allocations[i].balance,
            AllocationType::Portion => {
                allocations[i].balance = adapter::balance_query(
                    deps,
                    &full_asset.contract.address,
                    allocations[i].contract.clone(),
                )?;
                portion_total += allocations[i].balance;
            }
        };
    }

    let mut unbonding = Uint128::zero();

    // Withold pending unbondings
    for h in holders_r(&deps.storage).load()? {
        let holder = holder_r(&deps.storage).load(&h.as_str().as_bytes())?;
        if let Some(u) = holder.unbondings.iter().find(|u| u.token == asset) {
            unbonding += u.amount;
        }
    }

    // Batch send_from actions
    let mut send_actions = vec![];
    let mut messages = vec![];

    let key = viewing_key_r(&deps.storage).load()?;

    let mut allowance = allowance_query(
        &deps.querier,
        config.treasury.clone(),
        env.contract.address.clone(),
        key.clone(),
        1,
        full_asset.contract.code_hash.clone(),
        full_asset.contract.address.clone(),
    )?
    .allowance;

    let balance = balance_query(
        &deps.querier,
        self_address_r(&deps.storage).load()?,
        key.clone(),
        1,
        full_asset.contract.code_hash.clone(),
        full_asset.contract.address.clone(),
    )?
    .amount;

    let total = ((portion_total + allowance + balance) - unbonding)?;

    let mut total_unbond = Uint128::zero();
    let mut total_input = Uint128::zero();

    for adapter in allocations.clone() {
        match adapter.alloc_type {
            AllocationType::Amount => {
                //TODO Implement
            }
            AllocationType::Portion => {
                let desired_amount = adapter.amount.multiply_ratio(total, 10u128.pow(18));
                let threshold = desired_amount.multiply_ratio(adapter.tolerance, 10u128.pow(18));

                if adapter.balance < desired_amount {
                    // Need to add more from allowance
                    let input_amount = (desired_amount - adapter.balance)?;

                    if input_amount <= allowance {
                        total_input += input_amount;
                        send_actions.push(SendFromAction {
                            owner: config.treasury.clone(),
                            recipient: adapter.contract.address,
                            recipient_code_hash: Some(adapter.contract.code_hash),
                            amount: input_amount,
                            msg: None,
                            memo: None,
                        });
                        allowance = (allowance - input_amount)?;
                    } else {
                        total_input += allowance;
                        // Send all allowance
                        send_actions.push(SendFromAction {
                            owner: config.treasury.clone(),
                            recipient: adapter.contract.address,
                            recipient_code_hash: Some(adapter.contract.code_hash),
                            amount: allowance,
                            msg: None,
                            memo: None,
                        });

                        allowance = Uint128::zero();
                        break;
                    }
                }
            }
        };
    }

    if !send_actions.is_empty() {
        messages.push(batch_send_from_msg(
            send_actions,
            None,
            1,
            full_asset.contract.code_hash.clone(),
            full_asset.contract.address.clone(),
        )?);
    }

    Ok(Response {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Update {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn unbond<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: &Env,
    asset: Addr,
    amount: Uint128,
) -> StdResult<Response> {

    let config = config_r(&deps.storage).load()?;

    let mut unbonder = info.sender.clone();

    // admin unbonds on behalf of treasury
    if unbonder == config.admin {
        unbonder = config.treasury.clone();
    }
    let full_asset = assets_r(&deps.storage).load(asset.to_string().as_bytes())?;

    let holders = holders_r(&deps.storage).load()?;

    // Adjust holder balance
    if holders.contains(&unbonder) {
        let mut holder = holder_r(&deps.storage).load(unbonder.as_str().as_bytes())?;

        if holder.status != Status::Active {
            return Err(StdError::generic_err("Inactive Holder"));
        }

        if let Some(b) = holder.balances.iter().position(|h| h.token == asset) {

            // Check balance exceeds unbond amount
            if holder.balances[b].amount < amount {
                return Err(StdError::generic_err("Not enough funds to unbond"));
            }
            // Reduce balance
            else {
                holder.balances[b].amount = (holder.balances[b].amount - amount)?;
            }

            // Add unbonding
            if let Some(u) = holder.unbondings.iter().position(|h| h.token == asset) {
                holder.unbondings[u].amount += amount;
            }
            else {
                holder.unbondings.push(
                    Balance {
                        token: asset.clone(),
                        amount,
                    }
                );
            }
        }
        holder_w(deps.storage).save(&unbonder.as_str().as_bytes(), &holder)?;
    }
    else {
        return Err(StdError::unauthorized());
    }

    let mut unbond_amount = amount;

    // get other holders unbonding amount to hold
    let mut other_unbondings = Uint128::zero();

    for h in holders {
        if h == unbonder {
            continue;
        }
        let holder = holder_r(&deps.storage).load(&h.as_str().as_bytes())?;
        if let Some(u) = holder.unbondings.iter().find(|u| u.token == asset.clone()) {
            other_unbondings += u.amount;
        }
    }

    // Reserves to be sent immediately
    let mut reserves = balance_query(
        &deps.querier,
        self_address_r(&deps.storage).load()?,
        viewing_key_r(&deps.storage).load()?,
        1,
        full_asset.contract.code_hash.clone(),
        full_asset.contract.address.clone(),
    )?.amount;

    // Remove pending unbondings from reserves
    if reserves > other_unbondings {
        reserves = (reserves - other_unbondings)?;
    }
    else {
        reserves = Uint128::zero();
    }

    let mut messages = vec![];

    // Send available reserves to unbonder
    if reserves > Uint128::zero() {

        if reserves < unbond_amount {
            messages.push(
                send_msg(
                    unbonder.clone(),
                    reserves,
                    None,
                    None,
                    None,
                    1,
                    full_asset.contract.code_hash.clone(),
                    full_asset.contract.address.clone(),
                )?
            );
            unbond_amount = (unbond_amount - reserves)?;

            // Reflect sent funds in unbondings
            holder_w(deps.storage).update(&unbonder.as_str().as_bytes(), |mut h| {
                let mut holder = h.unwrap();
                if let Some(i) = holder.unbondings.iter().position(|u| u.token == asset) {
                    holder.unbondings[i].amount = (holder.unbondings[i].amount - reserves)?;
                }
                else {
                    return Err(StdError::generic_err("Failed to get unbonding, shouldn't happen"));
                }
                Ok(holder)
            })?;
        }
        else {
            messages.push(
                send_msg(
                    unbonder.clone(),
                    amount,
                    None,
                    None,
                    None,
                    1,
                    full_asset.contract.code_hash.clone(),
                    full_asset.contract.address.clone(),
                )?
            );
            unbond_amount = (unbond_amount - amount)?;

            // Reflect sent funds in unbondings
            holder_w(deps.storage).update(&unbonder.as_str().as_bytes(), |mut h| {
                let mut holder = h.unwrap();
                if let Some(i) = holder.unbondings.iter().position(|u| u.token == asset) {
                    holder.unbondings[i].amount = (holder.unbondings[i].amount - amount)?;
                }
                else {
                    return Err(StdError::generic_err("Failed to get unbonding, shouldn't happen"));
                }
                Ok(holder)
            })?;
        }
    }

    if unbond_amount >= Uint128::zero() {

        let full_asset = assets_r(&deps.storage).load(asset.to_string().as_bytes())?;

        let mut allocations = allocations_r(deps.storage).load(asset.to_string().as_bytes())?;

        // Build metadata
        let mut amount_total = Uint128::zero();
        let mut portion_total = Uint128::zero();

        // Gather adapter outstanding amounts
        for i in 0..allocations.len() {

            allocations[i].balance = adapter::balance_query(
                deps,
                &full_asset.contract.address,
                allocations[i].contract.clone(),
            )?;

            match allocations[i].alloc_type {
                AllocationType::Amount => amount_total += allocations[i].balance,
                AllocationType::Portion => portion_total += allocations[i].balance,
            };
        }

        let mut allowance = allowance_query(
            &deps.querier,
            config.treasury.clone(),
            env.contract.address.clone(),
            viewing_key_r(&deps.storage).load()?,
            1,
            full_asset.contract.code_hash.clone(),
            full_asset.contract.address.clone(),
        )?.allowance;

        let total = portion_total + allowance;

        allocations.sort_by(|a, b| a.balance.cmp(&b.balance));

        // Unbond from adapters
        for i in 0..allocations.len() {

            if unbond_amount == Uint128::zero() {
                break;
            }

            match allocations[i].alloc_type {
                AllocationType::Amount => {
                    //TODO: unbond back to desired amount
                }
                AllocationType::Portion => {
                    let desired_amount = total.multiply_ratio(
                        allocations[i].amount, 10u128.pow(18)
                    );

                    let unbondable = adapter::unbondable_query(&deps,
                                          &asset,
                                          allocations[i].contract.clone())?;

                    if unbond_amount > unbondable {
                        messages.push(
                            adapter::unbond_msg(
                                asset.clone(),
                                unbondable,
                                allocations[i].contract.clone()
                            )?
                        );
                        unbond_amount = (unbond_amount - unbondable)?;
                    }
                    else {
                        messages.push(
                            adapter::unbond_msg(
                                asset.clone(),
                                unbond_amount, 
                                allocations[i].contract.clone()
                            )?
                        );
                        unbond_amount = Uint128::zero()
                    }
                },
            };
        }
    }

    Ok(Response {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Unbond {
            status: ResponseStatus::Success,
            amount: unbond_amount,
        })?),
    })
}

pub fn add_holder<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: &Env,
    holder: Addr,
) -> StdResult<Response> {

    if info.sender != config_r(&deps.storage).load()?.admin {
        return Err(StdError::unauthorized());
    }

    let key = holder.as_str().as_bytes();

    holders_w(deps.storage).update(|mut h| {
        if h.contains(&holder.clone()) {
            return Err(StdError::generic_err("Holder already exists"));
        }
        h.push(holder.clone());
        Ok(h)
    })?;

    holder_w(deps.storage).save(key, &Holder {
        balances: Vec::new(),
        unbondings: Vec::new(),
        status: Status::Active,
    })?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddHolder {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn remove_holder<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: &Env,
    holder: Addr,
) -> StdResult<Response> {
    if info.sender != config_r(&deps.storage).load()?.admin {
        return Err(StdError::unauthorized());
    }

    let key = holder.as_str().as_bytes();

    if let Some(mut holder) = holder_r(&deps.storage).may_load(key)? {
        holder.status = Status::Closed;
        holder_w(deps.storage).save(key, &holder)?;
    } else {
        return Err(StdError::generic_err("Not an authorized holder"));
    }

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RemoveHolder {
            status: ResponseStatus::Success,
        })?),
    })
}
