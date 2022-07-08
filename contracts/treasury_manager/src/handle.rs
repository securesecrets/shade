use cosmwasm_std::{
    self,
    to_binary,
    Api,
    Binary,
    Env,
    Extern,
    HandleResponse,
    HumanAddr,
    Querier,
    StdError,
    StdResult,
    Storage,
    Uint128,
};
use secret_toolkit::{
    snip20::{
        allowance_query,
        batch::{SendFromAction, SendAction},
        balance_query,
        batch_send_from_msg,
        batch_send_msg,
        register_receive_msg,
        send_msg,
        set_viewing_key_msg,
    },
};

use shade_protocol::{
    contract_interfaces::{
        dao::{
            adapter,
            treasury_manager::{
                Allocation,
                AllocationMeta,
                AllocationType,
                Config,
                HandleAnswer,
                Holding,
                Balance,
                Status,
            }
        },
        snip20,
    },
    utils::{asset::Contract, generic_response::ResponseStatus},
};

use crate::{
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
        holding_r, holding_w,
        holdings_r, holdings_w,
        self_address_r,
    },
};
use std::collections::HashMap;



pub fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _sender: HumanAddr,
    from: HumanAddr,
    amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<HandleResponse> {

    /* TODO
     * All assets received from a "holder" will be credited to their account
     * All other assets from all other addresses will be credited to the treasury (default account)
     */

    let config = config_r(&deps.storage).load()?;
    let asset = assets_r(&deps.storage).load(env.message.sender.to_string().as_bytes())?;

    // Is Valid Holding
    let holding = match holdings_r(&deps.storage).load()?.contains(&from) {
        true => from,
        false=> config.treasury,
    };

    // Update holdings
    holding_w(&mut deps.storage).update(holding.as_str().as_bytes(), |h| {
        let mut holding = h.unwrap();
        if let Some(i) = holding.balances.iter().position(|b| b.token == asset.contract.address) {
            holding.balances[i].amount += amount;
        }
        else {
            holding.balances.push(
                Balance {
                    token: asset.contract.address,
                    amount: amount,
                }
            );
        }
        Ok(holding)
    })?;

    Ok(HandleResponse {
        messages: vec![],
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
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    if env.message.sender != config.admin {
        return Err(StdError::unauthorized());
    }

    asset_list_w(&mut deps.storage).update(|mut list| {
        list.push(contract.address.clone());
        Ok(list)
    })?;

    assets_w(&mut deps.storage).save(
        contract.address.to_string().as_bytes(),
        &snip20::helpers::fetch_snip20(contract, &deps.querier)?,
    )?;

    allocations_w(&mut deps.storage).save(contract.address.as_str().as_bytes(), &Vec::new())?;

    Ok(HandleResponse {
        messages: vec![
            // Register contract in asset
            register_receive_msg(
                env.contract_code_hash.clone(),
                None,
                256,
                contract.code_hash.clone(),
                contract.address.clone(),
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
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: HumanAddr,
    allocation: Allocation,
) -> StdResult<HandleResponse> {
    static ONE_HUNDRED_PERCENT: u128 = 10u128.pow(18);

    let config = config_r(&deps.storage).load()?;

    /* ADMIN ONLY */
    if env.message.sender != config.admin {
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

    if apps
        .iter()
        .map(|a| {
            if a.alloc_type == AllocationType::Portion {
                a.amount.u128()
            } else {
                0u128
            }
        })
        .sum::<u128>() > ONE_HUNDRED_PERCENT
    {
        return Err(StdError::generic_err(
            "Invalid allocation total exceeding 100%",
        ));
    }

    allocations_w(&mut deps.storage).save(key, &apps)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Allocate {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: HumanAddr,
) -> StdResult<HandleResponse> {

    if !asset_list_r(&deps.storage).load()?.contains(&asset) {
        return Err(StdError::generic_err("Unrecognized asset"));
    }
    let full_asset = assets_r(&deps.storage).load(asset.to_string().as_bytes())?;

    let config = config_r(&deps.storage).load()?;
    let mut claimer = env.message.sender.clone();

    if claimer == config.admin {
        claimer = config.treasury;
    }

    let holders = holders_r(&deps.storage).load()?;

    if !holders.contains(&claimer) {
        return Err(StdError::unauthorized());
    }
    let holder = holding_r(&deps.storage).load(&claimer.as_str().as_bytes())?;

    let unbonding = match holder.unbondings.iter().find(|u| u.token == asset) {
        Some(u) => u,
        None => {
            return Err(StdError::generic_err(
                    format!("No unbondings for token: {}, holder: {}", 
                             asset, claimer)
            ));
        }
    };

    let reserves = balance_query(
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

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Claim {
            status: ResponseStatus::Success,
            amount: reserves + total_claimed,
        })?),
    })
}

pub fn update<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: HumanAddr,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;

    let full_asset = assets_r(&deps.storage).load(asset.to_string().as_bytes())?;

    let mut allocations = allocations_r(&mut deps.storage).load(asset.to_string().as_bytes())?;

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
        let holder = holding_r(&deps.storage).load(&h.as_str().as_bytes())?;
        if let Some(u) = holder.unbondings.iter().find(|u| u.token == asset) {
            unbonding += u.amount;
        }
    }

    // Batch send_from actions
    let mut send_from_actions = vec![];
    let mut send_actions = vec![];
    let mut messages = vec![];

    let key = viewing_key_r(&deps.storage).load()?;

    // Available treasury allowance
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

    // Available balance
    let mut balance = balance_query(
        &deps.querier,
        self_address_r(&deps.storage).load()?,
        key.clone(),
        1,
        full_asset.contract.code_hash.clone(),
        full_asset.contract.address.clone(),
    )?
    .amount;

    let total = ((portion_total + allowance + balance) - unbonding)?;

    let _total_unbond = Uint128::zero();

    let mut allowance_used = Uint128::zero();
    // TODO: implement to use pending balance as well
    let mut balance_used = Uint128::zero();

    for adapter in allocations.clone() {
        match adapter.alloc_type {
            AllocationType::Amount => {
                //TODO Implement
            }
            AllocationType::Portion => {
                let desired_amount = adapter.amount.multiply_ratio(total, 10u128.pow(18));
                let threshold = desired_amount.multiply_ratio(adapter.tolerance, 10u128.pow(18));

                // Under funded
                if adapter.balance < desired_amount {

                    let mut desired_input = (desired_amount - adapter.balance)?;

                    // Check tolerance threshold
                    if desired_input < threshold {
                        continue;
                    }

                    // Fully covered by balance
                    if desired_input <= balance {
                        send_actions.push(SendAction {
                            recipient: adapter.contract.address.clone(),
                            recipient_code_hash: Some(adapter.contract.code_hash.clone()),
                            amount: desired_input,
                            msg: None,
                            memo: None,
                        });

                        balance = (balance - desired_input)?;
                        balance_used += desired_input;
                        desired_input = Uint128::zero();
                    }
                    // Send all balance
                    else if !balance.is_zero() {
                        send_actions.push(SendAction {
                            recipient: adapter.contract.address.clone(),
                            recipient_code_hash: Some(adapter.contract.code_hash.clone()),
                            amount: balance,
                            msg: None,
                            memo: None,
                        });

                        balance = Uint128::zero();
                        balance_used += balance;
                        desired_input = (desired_input - balance)?;
                        break;
                    }

                    // Fully covered by allowance
                    if desired_input <= allowance {
                        send_from_actions.push(SendFromAction {
                            owner: config.treasury.clone(),
                            recipient: adapter.contract.address.clone(),
                            recipient_code_hash: Some(adapter.contract.code_hash.clone()),
                            amount: desired_input,
                            msg: None,
                            memo: None,
                        });

                        allowance = (allowance - desired_input)?;
                        allowance_used += desired_input;
                        desired_input = Uint128::zero();
                    } 
                    // Send all allowance
                    else if !allowance.is_zero() {
                        send_from_actions.push(SendFromAction {
                            owner: config.treasury.clone(),
                            recipient: adapter.contract.address.clone(),
                            recipient_code_hash: Some(adapter.contract.code_hash.clone()),
                            amount: allowance,
                            msg: None,
                            memo: None,
                        });

                        allowance = Uint128::zero();
                        allowance_used += allowance;
                        desired_input = (desired_input - allowance)?;
                        break;
                    }
                }
                // Over funded
                else if adapter.balance > desired_amount {

                    let mut desired_output = (adapter.balance - desired_amount)?;

                    if desired_output < threshold {
                        continue;
                    }

                    messages.push(
                        adapter::unbond_msg(
                            asset.clone(),
                            desired_output,
                            adapter.contract.clone()
                        )?
                    );
                }
            }
        };
    }

    // Credit treasury balance with allowance used
    holding_w(&mut deps.storage).update(&config.treasury.as_str().as_bytes(), |h| {
        let mut holder = h.unwrap();
        if let Some(i) = holder.balances.iter().position(|u| u.token == asset) {
            holder.balances[i].amount = holder.balances[i].amount + allowance_used;
        }
        else {
            holder.balances.push(
                Balance {
                    token: asset,
                    amount: allowance_used,
                }
            );
        }
        Ok(holder)
    })?;

    if !send_actions.is_empty() {
        messages.push(batch_send_msg(
            send_actions,
            None,
            1,
            full_asset.contract.code_hash.clone(),
            full_asset.contract.address.clone(),
        )?);
    }

    if !send_from_actions.is_empty() {
        messages.push(batch_send_from_msg(
            send_from_actions,
            None,
            1,
            full_asset.contract.code_hash.clone(),
            full_asset.contract.address.clone(),
        )?);
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Update {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn unbond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    asset: HumanAddr,
    amount: Uint128,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    let mut unbonder = env.message.sender.clone();

    // admin unbonds on behalf of treasury
    if unbonder == config.admin {
        unbonder = config.treasury.clone();
    }

    let full_asset = assets_r(&deps.storage).load(asset.to_string().as_bytes())?;

    let holders = holders_r(&deps.storage).load()?;

    // Adjust holder balance
    if holders.contains(&unbonder) {
        let mut holder = holding_r(&deps.storage).load(unbonder.as_str().as_bytes())?;

        if holder.status != Status::Active {
            return Err(StdError::generic_err("Inactive Holding"));
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
        else {
            return Err(StdError::generic_err("Cannot unbond, holder has no balance"));
        }

        holding_w(&mut deps.storage).save(&unbonder.as_str().as_bytes(), &holder)?;
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
        let holder = holding_r(&deps.storage).load(&h.as_str().as_bytes())?;
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
            holding_w(&mut deps.storage).update(&unbonder.as_str().as_bytes(), |h| {
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
            holding_w(&mut deps.storage).update(&unbonder.as_str().as_bytes(), |h| {
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

        let mut allocations = allocations_r(&mut deps.storage).load(asset.to_string().as_bytes())?;

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

        let allowance = allowance_query(
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
                    let _desired_amount = total.multiply_ratio(
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

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Unbond {
            status: ResponseStatus::Success,
            amount: unbond_amount,
        })?),
    })
}

pub fn add_holder<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    holder: HumanAddr,
) -> StdResult<HandleResponse> {

    if env.message.sender != config_r(&deps.storage).load()?.admin {
        return Err(StdError::unauthorized());
    }

    let key = holder.as_str().as_bytes();

    holders_w(&mut deps.storage).update(|mut h| {
        if h.contains(&holder.clone()) {
            return Err(StdError::generic_err("Holding already exists"));
        }
        h.push(holder.clone());
        Ok(h)
    })?;

    holding_w(&mut deps.storage).save(key, &Holding {
        balances: Vec::new(),
        unbondings: Vec::new(),
        status: Status::Active,
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddHolding {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn remove_holder<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    holder: HumanAddr,
) -> StdResult<HandleResponse> {
    if env.message.sender != config_r(&deps.storage).load()?.admin {
        return Err(StdError::unauthorized());
    }

    let key = holder.as_str().as_bytes();

    if let Some(mut holder) = holding_r(&deps.storage).may_load(key)? {
        holder.status = Status::Closed;
        holding_w(&mut deps.storage).save(key, &holder)?;
    } else {
        return Err(StdError::generic_err("Not an authorized holder"));
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RemoveHolding {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn distribute_gain(
    gain: Uint128,
    token: HumanAddr,
    holders: mut HashMap<HumanAddr, Holding>, 
) -> StdResult<HashMap<HumanAddr, Holding>> {
    let ratios = holding_ratios(&mut holders);

    for addr, holder  in holders {
        let balance = match holder.balances.iter().find(|u| u.token == asset) {
            Some(b) => b,
            None => Uint128::zero(),
        }
    }

    Ok(holders)
}

pub fn distribute_loss(
    loss: Uint128,
    token: HumanAddr,
    holders: mut HashMap<HumanAddr, Holding>, 
) -> StdResult<Vec<Holding>> {
    let ratios = holding_ratios(&mut holders);

    Ok(holders)
}

/* Builds a map of { HumanAddr: <asset_portion * 10^18> }
 */
pub fn holding_ratios(
    holdings: mut Vec<Holding>, 
) -> HashMap<HumanAddr, Uint128> {

    let ratios = HashMap<HumanAddr, Uint128>::new();
    let denominator = 10u128.pow(18);

    let sum = holdings
        .iter()
        .map(|(addr, holding)| {
            match holding.balances.iter().find(|u| u.token == asset) {
                Some(b) => b.u128(),
                None => 0u128,
            }
        }).sum();

    for (addr, holding) in holding {
        let balance = match holding.balances.iter().find(|u| u.token == asset) {
            Some(b) => b,
            None => Uint128::zero(),
        }
        ratios.insert(&addr, balance.multiply_ratio(10u128.pow(18), total));
    }

    ratios
}
