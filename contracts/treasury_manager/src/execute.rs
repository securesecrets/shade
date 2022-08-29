use shade_protocol::{
    admin::helpers::{validate_admin, AdminPermissions},
    c_std::{
        to_binary,
        Addr,
        Binary,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
        Uint128,
    },
    dao::{
        adapter,
        treasury_manager::{
            Action,
            Allocation,
            AllocationMeta,
            AllocationType,
            Balance,
            Config,
            Context,
            ExecuteAnswer,
            Holding,
            Metric,
            Status,
        },
    },
    snip20,
    snip20::{
        batch::{SendAction, SendFromAction},
        helpers::{
            allowance_query,
            balance_query,
            batch_send_from_msg,
            batch_send_msg,
            register_receive,
            send_msg,
            set_viewing_key_msg,
        },
    },
    utils::{asset::Contract, generic_response::ResponseStatus},
};

use crate::storage::*;

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _sender: Addr,
    from: Addr,
    amount: Uint128,
    _msg: Option<Binary>,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    let asset = ASSETS.load(deps.storage, info.sender.clone())?;

    // Do nothing if its an adapter (claimed funds)
    if let Some(_) = ALLOCATIONS
        .load(deps.storage, info.sender.clone())?
        .iter()
        .find(|a| a.contract.address == from)
    {
        return Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Receive {
            status: ResponseStatus::Success,
        })?));
    }

    // Default to treasury if not sent by a holder
    let holder = match HOLDERS.load(deps.storage)?.contains(&from) {
        true => from.clone(),
        false => config.treasury,
    };

    // Update holdings
    HOLDING.update(deps.storage, holder, |h| -> StdResult<Holding> {
        let mut holding = h.unwrap();
        if let Some(i) = holding
            .balances
            .iter()
            .position(|b| b.token == asset.contract.address)
        {
            holding.balances[i].amount += amount;
        } else {
            holding.balances.push(Balance {
                token: asset.contract.address,
                amount,
            });
        }
        Ok(holding)
    })?;

    METRICS.pushf(deps.storage, env.block.time, Metric {
        action: Action::FundsReceived,
        context: Context::Receive,
        timestamp: env.block.time.seconds(),
        token: info.sender,
        amount,
        user: from,
    })?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Receive {
        status: ResponseStatus::Success,
    })?))
}

pub fn try_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: Config,
) -> StdResult<Response> {
    let cur_config = CONFIG.load(deps.storage)?;

    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryManager,
        &info.sender,
        &cur_config.admin_auth,
    )?;

    CONFIG.save(deps.storage, &config)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn try_register_asset(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    contract: &Contract,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryManager,
        &info.sender,
        &config.admin_auth,
    )?;

    ASSET_LIST.update(deps.storage, |mut list| -> StdResult<Vec<Addr>> {
        list.push(contract.address.clone());
        Ok(list)
    })?;

    ASSETS.save(
        deps.storage,
        contract.address.clone(),
        &snip20::helpers::fetch_snip20(&contract, &deps.querier)?,
    )?;

    ALLOCATIONS.save(deps.storage, contract.address.clone(), &Vec::new())?;

    Ok(Response::new()
        .add_messages(vec![
            // Register contract in asset
            register_receive(env.contract.code_hash.clone(), None, &contract)?,
            // Set viewing key
            set_viewing_key_msg(VIEWING_KEY.load(deps.storage)?, None, &contract)?,
        ])
        .set_data(to_binary(&ExecuteAnswer::RegisterAsset {
            status: ResponseStatus::Success,
        })?))
}

pub fn allocate(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    asset: Addr,
    allocation: Allocation,
) -> StdResult<Response> {
    static ONE_HUNDRED_PERCENT: u128 = 10u128.pow(18);

    let config = CONFIG.load(deps.storage)?;

    /* ADMIN ONLY */
    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryManager,
        &info.sender,
        &config.admin_auth,
    )?;

    let mut apps = ALLOCATIONS
        .may_load(deps.storage, asset.clone())?
        .unwrap_or_default();

    let stale_alloc = apps
        .iter()
        .position(|a| a.contract.address == allocation.contract.address);

    match stale_alloc {
        Some(i) => {
            apps.swap_remove(i);
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
        .sum::<u128>()
        > ONE_HUNDRED_PERCENT
    {
        return Err(StdError::generic_err(
            "Invalid allocation total exceeding 100%",
        ));
    }

    apps.sort_by(|a, b| match a.alloc_type {
        AllocationType::Amount => match b.alloc_type {
            AllocationType::Amount => std::cmp::Ordering::Equal,
            AllocationType::Portion => std::cmp::Ordering::Less,
        },
        AllocationType::Portion => match b.alloc_type {
            AllocationType::Amount => std::cmp::Ordering::Greater,
            AllocationType::Portion => std::cmp::Ordering::Equal,
        },
    });

    ALLOCATIONS.save(deps.storage, asset.clone(), &apps)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::Allocate {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn claim(deps: DepsMut, env: &Env, info: MessageInfo, asset: Addr) -> StdResult<Response> {
    let full_asset = match ASSETS.may_load(deps.storage, asset.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Unrecognized asset"));
        }
    };

    let config = CONFIG.load(deps.storage)?;
    let mut claimer = info.sender;

    if validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryManager,
        &claimer,
        &config.admin_auth,
    )
    .is_ok()
    {
        //assert!(false, "CLAIMER TREASURY");
        claimer = config.treasury;
    }

    let mut total_claimed = Uint128::zero();
    let mut messages = vec![];

    for alloc in ALLOCATIONS.load(deps.storage, asset.clone())? {
        let claim = adapter::claimable_query(deps.querier, &asset, alloc.contract.clone())?;
        if claim > Uint128::zero() {
            messages.push(adapter::claim_msg(&asset, alloc.contract.clone())?);
            METRICS.pushf(deps.storage, env.block.time, Metric {
                action: Action::Claim,
                context: Context::Claim,
                timestamp: env.block.time.seconds(),
                token: asset.clone(),
                amount: claim,
                user: claimer.clone(),
            })?;
            total_claimed += claim;
        }
    }

    let holders = HOLDERS.load(deps.storage)?;

    if !holders.contains(&claimer.clone()) {
        return Err(StdError::generic_err("Unauthorized"));
    }
    let mut holding = HOLDING.load(deps.storage, claimer.clone())?;

    let unbonding_i = match holding
        .unbondings
        .iter_mut()
        .position(|u| u.token == asset.clone())
    {
        Some(i) => i,
        None => {
            return Err(StdError::generic_err(format!(
                "{} has no unbondings for {}",
                claimer.clone(),
                asset.clone()
            )));
        }
    };

    let reserves = balance_query(
        &deps.querier,
        SELF_ADDRESS.load(deps.storage)?,
        VIEWING_KEY.load(deps.storage)?,
        &full_asset.contract.clone(),
    )?;

    let send_amount;

    if holding.unbondings[unbonding_i].amount > reserves + total_claimed {
        send_amount = reserves + total_claimed;
    } else {
        send_amount = holding.unbondings[unbonding_i].amount;
    }
    // Adjust unbonding amount
    holding.unbondings[unbonding_i].amount = holding.unbondings[unbonding_i].amount - send_amount;
    HOLDING.save(deps.storage, claimer.clone(), &holding)?;

    // Send claimed funds
    messages.push(send_msg(
        claimer.clone(),
        send_amount,
        None,
        None,
        None,
        &full_asset.contract.clone(),
    )?);

    METRICS.pushf(deps.storage, env.block.time, Metric {
        action: Action::SendFunds,
        context: Context::Claim,
        timestamp: env.block.time.seconds(),
        token: asset.clone(),
        amount: send_amount,
        user: claimer.clone(),
    })?;

    Ok(Response::new().add_messages(messages).set_data(to_binary(
        &adapter::ExecuteAnswer::Claim {
            status: ResponseStatus::Success,
            amount: reserves + total_claimed,
        },
    )?))
}

pub fn update(deps: DepsMut, env: &Env, _info: MessageInfo, asset: Addr) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    let full_asset = ASSETS.load(deps.storage, asset.clone())?;

    let mut allocations = ALLOCATIONS.load(deps.storage, asset.clone())?;

    // Build metadata
    // amount_total is the sum of balances on adapters with amount allocations
    let mut amount_total = Uint128::zero();
    // protion_total is the sum of balances on adapters with portion allocaitons
    let mut portion_total = Uint128::zero();
    let mut effective_unbonding = vec![];

    // vec to keep track of if any allocations need to be removed
    let mut stale_allocs = vec![];
    let mut messages = vec![];

    /* this loop has 2 purposes
     * - check for stale allocaitons that need to be removed
     * - fill the amount_total and portion_total vars with data
     */
    for (i, a) in allocations.clone().iter().enumerate() {
        allocations[i].balance = adapter::balance_query(
            deps.querier,
            &full_asset.contract.address,
            a.contract.clone(),
        )?;
        effective_unbonding.push(adapter::unbonding_query(
            deps.querier,
            &full_asset.contract.address,
            a.contract.clone(),
        )?);
        let unbondable = adapter::unbondable_query(
            deps.querier,
            &full_asset.contract.address,
            a.contract.clone(),
        )?;
        let claimable = adapter::claimable_query(
            deps.querier,
            &full_asset.contract.address,
            a.contract.clone(),
        )?;
        if !claimable.is_zero() {
            messages.push(adapter::claim_msg(
                &full_asset.contract.address.clone(),
                a.contract.clone(),
            )?);
            effective_unbonding[i] += claimable;
        }
        // if all these values are zero we can safely drop the alloc
        if allocations[i].balance.is_zero()
            && a.amount.is_zero()
            && effective_unbonding[i].is_zero()
            && unbondable.is_zero()
            && claimable.is_zero()
        {
            stale_allocs.push(i);
        }
        // fill totals with data
        match a.alloc_type {
            AllocationType::Amount => amount_total += allocations[i].balance,
            AllocationType::Portion => {
                portion_total += allocations[i].balance;
            }
        };
    }
    // actually drop the stale allocs
    if !stale_allocs.is_empty() {
        for index in stale_allocs.iter().rev() {
            allocations.remove(index.clone());
        }
        ALLOCATIONS.save(deps.storage, asset.clone(), &allocations)?;
    }

    // the holder is the entity that actually holds the tokens that the treasury manager can spend
    // holder_unbonding represents how much the holder has currently asked to unbond
    let mut holder_unbonding = Uint128::zero();
    // holder_principal represents how much of the asset has came form said holder
    let mut holder_principal = Uint128::zero();

    // Withold holder unbondings
    for h in HOLDERS.load(deps.storage)? {
        // for each holder, load the respective holdings
        let holding = HOLDING.load(deps.storage, h)?;
        // sum the data
        if let Some(u) = holding.unbondings.iter().find(|u| u.token == asset) {
            holder_unbonding += u.amount;
        }
        if let Some(b) = holding.balances.iter().find(|u| u.token == asset) {
            holder_principal += b.amount;
        }
    }

    // Batch send_from actions
    let mut send_from_actions = vec![];
    let mut send_actions = vec![];

    let key = VIEWING_KEY.load(deps.storage)?;

    // Available treasury allowance
    let mut allowance = allowance_query(
        &deps.querier,
        config.treasury.clone(),
        env.contract.address.clone(),
        key.clone(),
        1,
        &full_asset.contract.clone(),
    )?
    .allowance;

    // snip20 balance query to get the treasury managers current snip20 balance
    let mut balance = balance_query(
        &deps.querier,
        SELF_ADDRESS.load(deps.storage)?,
        key.clone(),
        &full_asset.contract.clone(),
    )?;

    // this var is ment to hold the total amount that the treasury has allocated to its adapters
    // plus it's current snip20 balance
    // We subtract holder_unbonding to ensure that those tokens will be claimable
    let out_total = (amount_total + portion_total + balance) - holder_unbonding;
    // This gives us our total allowance from the treasury, used and unused
    let total = out_total + allowance;

    //setting up vars
    let mut allowance_used = Uint128::zero();
    let mut balance_used = Uint128::zero();
    let mut reserved_for_amount_adapters = Uint128::zero();

    // loop through adapters with allocations
    for (i, adapter) in allocations.clone().iter().enumerate() {
        // calculate the target balance for each
        let desired_amount = match adapter.alloc_type {
            AllocationType::Amount => {
                reserved_for_amount_adapters += adapter.amount;
                // since amount adapters' allocations are static
                adapter.amount
            }
            AllocationType::Portion => {
                // Since the list of allocations is sorted, we can ensure that type::amount
                // adapters will be processed first, so we can calculate the amount available for
                // allocation with total - reserved_for_amount_adapters
                // If statement to prevent overflow
                if total > reserved_for_amount_adapters {
                    adapter
                        .amount
                        .multiply_ratio(total - reserved_for_amount_adapters, 10u128.pow(18))
                } else {
                    Uint128::zero()
                }
            }
        };
        // threshold is the desired_amount * a percentage held in adapter.tolerance,
        // the treasury manager will only attempt to rebalance if the adapter crosses the threshold
        // in either direction
        let threshold = desired_amount.multiply_ratio(adapter.tolerance, 10u128.pow(18));

        let effective_balance = {
            if adapter.balance > effective_unbonding[i] {
                adapter.balance - effective_unbonding[i]
            } else {
                adapter.balance
            }
        };

        // Under Funded -- prioritize tm snip20 balance over allowance from treasury
        if effective_balance < desired_amount {
            // target send amount to adapter
            let mut desired_input = desired_amount - effective_balance;
            // check if threshold is crossed
            if desired_input <= threshold {
                continue;
            }

            // Fully covered by balance
            if desired_input < balance {
                send_actions.push(SendAction {
                    recipient: adapter.contract.address.clone().to_string(),
                    recipient_code_hash: Some(adapter.contract.code_hash.clone()),
                    amount: desired_input,
                    msg: None,
                    memo: None,
                });
                METRICS.pushf(deps.storage, env.block.time, Metric {
                    action: Action::SendFunds,
                    context: Context::Update,
                    timestamp: env.block.time.seconds(),
                    token: asset.clone(),
                    amount: desired_input,
                    user: adapter.contract.address.clone(),
                })?;

                // reduce snip20 balance for future loops
                balance = balance - desired_input;
                balance_used += desired_input;
                // at this point we know we have fufilled what this adapter needs
                continue;
            }
            // Send all snip20 balance since the adapter needs more that the balance can fufill,
            // but balance is not 0
            else if !balance.is_zero() {
                send_actions.push(SendAction {
                    recipient: adapter.contract.address.clone().to_string(),
                    recipient_code_hash: Some(adapter.contract.code_hash.clone()),
                    amount: balance,
                    msg: None,
                    memo: None,
                });
                METRICS.pushf(deps.storage, env.block.time, Metric {
                    action: Action::SendFunds,
                    context: Context::Update,
                    timestamp: env.block.time.seconds(),
                    token: asset.clone(),
                    amount: balance,
                    user: adapter.contract.address.clone(),
                })?;

                // reduce the desired_input to reflect the balance being sent, we know this will
                // not overflow because if balance was > desired_input, we would have hit a
                // continue statement
                desired_input = desired_input - balance;
                // reset balance since we have effectively sent everything out
                balance = Uint128::zero();
            }

            if !allowance.is_zero() {
                // This will only execute after snip20 balance has been used up
                // Fully covered by allowance
                if desired_input < allowance {
                    send_from_actions.push(SendFromAction {
                        owner: config.treasury.clone().to_string(),
                        recipient: adapter.contract.address.clone().to_string(),
                        recipient_code_hash: Some(adapter.contract.code_hash.clone()),
                        amount: desired_input,
                        msg: None,
                        memo: None,
                    });
                    METRICS.pushf(deps.storage, env.block.time, Metric {
                        action: Action::SendFundsFrom,
                        context: Context::Update,
                        timestamp: env.block.time.seconds(),
                        token: asset.clone(),
                        amount: desired_input,
                        user: adapter.contract.address.clone(),
                    })?;

                    allowance_used += desired_input;
                    // this will not overflow due to check in if statement
                    allowance = allowance - desired_input;
                    // similarily, we know that we have fufilled what this adapter needs at this
                    // point but we don't want to continue since we need to account for the
                    // allowance used in the holder's information
                }
                // Send all allowance
                else if !allowance.is_zero() {
                    send_from_actions.push(SendFromAction {
                        owner: config.treasury.clone().to_string(),
                        recipient: adapter.contract.address.clone().to_string(),
                        recipient_code_hash: Some(adapter.contract.code_hash.clone()),
                        amount: allowance,
                        msg: None,
                        memo: None,
                    });
                    METRICS.pushf(deps.storage, env.block.time, Metric {
                        action: Action::SendFundsFrom,
                        context: Context::Update,
                        timestamp: env.block.time.seconds(),
                        token: asset.clone(),
                        amount: allowance,
                        user: adapter.contract.address.clone(),
                    })?;

                    // account for allowance being sent out
                    allowance_used += allowance;
                    allowance = Uint128::zero();
                }
            }
        }
        // Over funded -- unbond
        else if effective_balance > desired_amount {
            // balance - target balance will give the amount we need to unbond
            let desired_output = effective_balance - desired_amount;
            // check to see that the threshold has been crossed
            if desired_output <= threshold {
                continue;
            }
            messages.push(adapter::unbond_msg(
                &asset.clone(),
                desired_output,
                adapter.contract.clone(),
            )?);
            let unbondings = UNBONDINGS.load(deps.storage)? + desired_output;
            UNBONDINGS.save(deps.storage, &unbondings)?;
            METRICS.pushf(deps.storage, env.block.time, Metric {
                action: Action::Unbond,
                context: Context::Update,
                timestamp: env.block.time.seconds(),
                token: asset.clone(),
                amount: desired_output,
                user: adapter.contract.address.clone(),
            })?;
        }
    }

    // Credit treasury balance with allowance used by adding allowance_used to the existing balance
    // or creating a new balance struct with allowance_used as the balance
    HOLDING.update(
        deps.storage,
        config.treasury.clone(),
        |h| -> StdResult<Holding> {
            let mut holding = h.unwrap();
            if let Some(i) = holding
                .balances
                .iter()
                .position(|u| u.token == asset.clone())
            {
                holding.balances[i].amount = holding.balances[i].amount + allowance_used;
            } else {
                holding.balances.push(Balance {
                    token: asset.clone(),
                    amount: allowance_used,
                });
            }
            Ok(holding)
        },
    )?;

    // Determine Gainz & Losses & credit to treasury
    holder_principal += allowance_used;
    // this will never overflow because total is a sum of allowance
    if total - allowance > holder_principal {
        let gains = (total - allowance) - holder_principal;
        // debit gains to treasury
        let mut holding = HOLDING.load(deps.storage, config.treasury.clone())?;
        if let Some(i) = holding.balances.iter().position(|u| u.token == asset) {
            holding.balances[i].amount += gains;
        }
        HOLDING.save(deps.storage, config.treasury.clone(), &holding)?;
        METRICS.pushf(deps.storage, env.block.time, Metric {
            action: Action::RealizeGains,
            context: Context::Update,
            timestamp: env.block.time.seconds(),
            token: asset.clone(),
            amount: gains,
            user: config.treasury.clone(),
        })?;
    } else if total - allowance < holder_principal {
        let losses = holder_principal - (total - allowance);
        // credit losses to treasury
        let mut holding = HOLDING.load(deps.storage, config.treasury.clone())?;
        if let Some(i) = holding.balances.iter().position(|u| u.token == asset) {
            holding.balances[i].amount -= losses;
        }
        HOLDING.save(deps.storage, config.treasury.clone(), &holding)?;
        METRICS.pushf(deps.storage, env.block.time, Metric {
            action: Action::RealizeLosses,
            context: Context::Update,
            timestamp: env.block.time.seconds(),
            token: asset.clone(),
            amount: losses,
            user: config.treasury.clone(),
        })?;
    }

    // push batch messages
    if !send_actions.is_empty() {
        messages.push(batch_send_msg(
            send_actions,
            None,
            &full_asset.contract.clone(),
        )?);
    }

    // push batch messages
    if !send_from_actions.is_empty() {
        messages.push(batch_send_from_msg(
            send_from_actions,
            None,
            &full_asset.contract.clone(),
        )?);
    }

    Ok(Response::new().add_messages(messages).set_data(to_binary(
        &adapter::ExecuteAnswer::Update {
            status: ResponseStatus::Success,
        },
    )?))
}

pub fn unbond(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    asset: Addr,
    amount: Uint128,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;
    //let asset = deps.api.addr_validate(asset.as_str())?;
    let mut unbonder = info.sender.clone();

    // admin unbonds on behalf of treasury
    if validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryManager,
        &unbonder,
        &config.admin_auth,
    )
    .is_ok()
    {
        unbonder = config.treasury.clone();
    }

    let full_asset = ASSETS.load(deps.storage, asset.clone())?;

    let holders = HOLDERS.load(deps.storage)?;

    // Adjust holder balance
    if holders.contains(&unbonder.clone()) {
        let mut holding = HOLDING.load(deps.storage, unbonder.clone())?;

        if holding.status != Status::Active {
            return Err(StdError::generic_err("Inactive Holding"));
        }

        let balance_i = match holding
            .balances
            .iter()
            .position(|h| h.token == asset.clone())
        {
            Some(i) => i,
            None => {
                return Err(StdError::generic_err(format!(
                    "Cannot unbond, holder has no holdings of {}",
                    asset.clone()
                )));
            }
        };

        // Check balance exceeds unbond amount
        if holding.balances[balance_i].amount < amount {
            return Err(StdError::generic_err("Not enough funds to unbond"));
        } else {
            holding.balances[balance_i].amount = holding.balances[balance_i].amount - amount;
        }

        // Add unbonding
        if let Some(u) = holding
            .unbondings
            .iter()
            .position(|h| h.token == asset.clone())
        {
            holding.unbondings[u].amount += amount;
        } else {
            holding.unbondings.push(Balance {
                token: asset.clone(),
                amount,
            });
        }

        HOLDING.save(deps.storage, unbonder.clone(), &holding)?;
    } else {
        return Err(StdError::generic_err("unauthorized"));
    }

    let mut unbonding_tot = Uint128::zero();
    for a in ALLOCATIONS.load(deps.storage, asset.clone())? {
        unbonding_tot +=
            adapter::unbonding_query(deps.querier, &asset.clone(), a.contract.clone())?;
    }

    let mut unbond_amount = {
        let u = UNBONDINGS.load(deps.storage)?;
        if u <= unbonding_tot {
            if u <= amount {
                UNBONDINGS.save(deps.storage, &Uint128::zero())?;
                amount - u
            } else {
                UNBONDINGS.save(deps.storage, &(u - amount))?;
                Uint128::zero()
            }
        } else {
            amount
        }
    };

    // get other holders unbonding amount to hold
    let mut other_unbondings = Uint128::zero();

    for h in holders {
        if h == unbonder.clone() {
            continue;
        }
        let other_holding = HOLDING.load(deps.storage, h)?;
        if let Some(u) = other_holding
            .unbondings
            .iter()
            .find(|u| u.token == asset.clone())
        {
            other_unbondings += u.amount;
        }
    }

    // Reserves to be sent immediately
    let mut reserves = balance_query(
        &deps.querier,
        SELF_ADDRESS.load(deps.storage)?,
        VIEWING_KEY.load(deps.storage)?,
        &full_asset.contract.clone(),
    )?;

    // Remove pending unbondings from reserves
    if reserves > other_unbondings {
        reserves = reserves - other_unbondings;
    } else {
        reserves = Uint128::zero();
    }

    let mut messages = vec![];

    // Send available reserves to unbonder
    if reserves > Uint128::zero() {
        if reserves < unbond_amount {
            // reserves can't cover unbond
            messages.push(send_msg(
                unbonder.clone(),
                reserves,
                None,
                None,
                None,
                &full_asset.contract.clone(),
            )?);
            METRICS.pushf(deps.storage, env.block.time, Metric {
                action: Action::SendFunds,
                context: Context::Unbond,
                timestamp: env.block.time.seconds(),
                token: asset.clone(),
                amount: reserves,
                user: unbonder.clone(),
            })?;
            unbond_amount = unbond_amount - reserves;

            // Reflect sent funds in unbondings
            HOLDING.update(deps.storage, unbonder, |h| -> StdResult<Holding> {
                let mut holding = h.unwrap();
                if let Some(i) = holding.unbondings.iter().position(|u| u.token == asset) {
                    holding.unbondings[i].amount = holding.unbondings[i].amount - reserves;
                } else {
                    return Err(StdError::generic_err(
                        "Failed to get unbonding, shouldn't happen",
                    ));
                }
                Ok(holding)
            })?;
        } else {
            // reserves can cover unbond
            messages.push(send_msg(
                unbonder.clone(),
                amount,
                None,
                None,
                None,
                &full_asset.contract.clone(),
            )?);
            METRICS.pushf(deps.storage, env.block.time, Metric {
                action: Action::SendFunds,
                context: Context::Unbond,
                timestamp: env.block.time.seconds(),
                token: asset.clone(),
                amount,
                user: unbonder.clone(),
            })?;

            // Reflect sent funds in unbondings
            HOLDING.update(deps.storage, unbonder, |h| {
                let mut holder = h.unwrap();
                if let Some(i) = holder.unbondings.iter().position(|u| u.token == asset) {
                    holder.unbondings[i].amount = holder.unbondings[i].amount - amount;
                } else {
                    return Err(StdError::generic_err(
                        "Failed to get unbonding, shouldn't happen",
                    ));
                }
                Ok(holder)
            })?;

            return Ok(Response::new().add_messages(messages).set_data(to_binary(
                &adapter::ExecuteAnswer::Unbond {
                    status: ResponseStatus::Success,
                    amount,
                },
            )?));
        }
    }

    let full_asset = ASSETS.load(deps.storage, asset.clone())?;

    let mut allocations = ALLOCATIONS.load(deps.storage, asset.clone())?;

    // Build metadata
    let mut amount_total = Uint128::zero();
    let mut portion_total = Uint128::zero();

    // Gather adapter outstanding amounts
    for i in 0..allocations.len() {
        allocations[i].balance = adapter::balance_query(
            deps.querier,
            &full_asset.contract.address,
            allocations[i].contract.clone(),
        )?;

        match allocations[i].alloc_type {
            AllocationType::Amount => amount_total += allocations[i].balance,
            AllocationType::Portion => portion_total += allocations[i].balance,
        };
    }

    let mut alloc_meta = vec![];
    let mut tot_unbond_available = Uint128::zero();
    let mut tot_unbonding = Uint128::zero();

    for allocation in allocations.clone() {
        let bal = adapter::unbondable_query(deps.querier, &asset, allocation.contract.clone())?;
        let unbonding =
            adapter::unbonding_query(deps.querier, &asset, allocation.contract.clone())?;

        alloc_meta.push(AllocationMeta {
            nick: allocation.nick,
            contract: allocation.contract,
            amount: allocation.amount,
            alloc_type: allocation.alloc_type,
            balance: bal,
            tolerance: Uint128::zero(),
        });
        tot_unbond_available += bal;
        tot_unbonding += unbonding;
    }

    if unbond_amount == tot_unbond_available {
        for a in alloc_meta.clone() {
            messages.push(adapter::unbond_msg(
                &full_asset.contract.address.clone(),
                a.balance.clone(),
                a.contract.clone(),
            )?);
        }
        return Ok(Response::new().add_messages(messages).set_data(to_binary(
            &adapter::ExecuteAnswer::Unbond {
                status: ResponseStatus::Success,
                amount,
            },
        )?));
    }

    let mut total_amount_unbonding = Uint128::zero();

    let mut unbond_amounts = vec![];

    let portions = alloc_meta
        .clone()
        .into_iter()
        .filter(|a| a.alloc_type == AllocationType::Portion)
        .collect::<Vec<AllocationMeta>>();
    let amounts = alloc_meta
        .clone()
        .into_iter()
        .filter(|a| a.alloc_type == AllocationType::Amount)
        .collect::<Vec<AllocationMeta>>();

    for meta in amounts.clone() {
        if meta.balance > meta.amount {
            total_amount_unbonding += meta.balance - meta.amount;
            unbond_amounts.push(meta.balance - meta.amount);
        } else {
            unbond_amounts.push(Uint128::zero())
        }
    }

    if unbond_amount == total_amount_unbonding {
        for (i, meta) in amounts.clone().iter().enumerate() {
            messages.push(adapter::unbond_msg(
                &full_asset.contract.address.clone(),
                unbond_amounts[i],
                meta.contract.clone(),
            )?);
            METRICS.pushf(deps.storage, env.block.time, Metric {
                action: Action::Unbond,
                context: Context::Unbond,
                timestamp: env.block.time.seconds(),
                token: asset.clone(),
                amount: unbond_amounts[i],
                user: meta.contract.address.clone(),
            })?;
        }
        return Ok(Response::new().add_messages(messages).set_data(to_binary(
            &adapter::ExecuteAnswer::Unbond {
                status: ResponseStatus::Success,
                amount,
            },
        )?));
    } else if unbond_amount < total_amount_unbonding {
        let mut modified_total_amount_unbonding = Uint128::zero();
        for (i, meta) in amounts.clone().iter().enumerate() {
            unbond_amounts[i] =
                unbond_amount.multiply_ratio(unbond_amounts[i], total_amount_unbonding);
            modified_total_amount_unbonding += unbond_amounts[i];
            if i == amounts.len() - 1 && modified_total_amount_unbonding < unbond_amount {
                unbond_amounts[i] += Uint128::new(1);
            }
            messages.push(adapter::unbond_msg(
                &full_asset.contract.address.clone(),
                unbond_amounts[i],
                meta.contract.clone(),
            )?);
            METRICS.pushf(deps.storage, env.block.time, Metric {
                action: Action::Unbond,
                context: Context::Unbond,
                timestamp: env.block.time.seconds(),
                token: asset.clone(),
                amount: unbond_amounts[i],
                user: meta.contract.address.clone(),
            })?;
        }
        return Ok(Response::new().add_messages(messages).set_data(to_binary(
            &adapter::ExecuteAnswer::Unbond {
                status: ResponseStatus::Success,
                amount,
            },
        )?));
    }

    // if portion total > unbond - tot, we know the portion adapters can cover the rest
    if unbond_amount - total_amount_unbonding < portion_total {
        for (i, meta) in amounts.clone().iter().enumerate() {
            if !unbond_amounts[i].is_zero() {
                messages.push(adapter::unbond_msg(
                    &full_asset.contract.address.clone(),
                    unbond_amounts[i],
                    meta.contract.clone(),
                )?);
                METRICS.pushf(deps.storage, env.block.time, Metric {
                    action: Action::Unbond,
                    context: Context::Unbond,
                    timestamp: env.block.time.seconds(),
                    token: asset.clone(),
                    amount: unbond_amounts[i],
                    user: meta.contract.address.clone(),
                })?;
            }
        }
        let amount_adapt_tot_unbonding = total_amount_unbonding;
        for (i, meta) in portions.clone().iter().enumerate() {
            let unbond_from_portion = (unbond_amount - amount_adapt_tot_unbonding)
                .multiply_ratio(meta.balance, portion_total);
            unbond_amounts.push(unbond_from_portion);
            total_amount_unbonding += unbond_from_portion;
            if i == portions.len() - 1 && total_amount_unbonding < unbond_amount {
                messages.push(adapter::unbond_msg(
                    &full_asset.contract.address.clone(),
                    unbond_from_portion + Uint128::new(1),
                    meta.contract.clone(),
                )?);
                METRICS.pushf(deps.storage, env.block.time, Metric {
                    action: Action::Unbond,
                    context: Context::Unbond,
                    timestamp: env.block.time.seconds(),
                    token: asset.clone(),
                    amount: unbond_from_portion + Uint128::new(1),
                    user: meta.contract.address.clone(),
                })?;
            } else if !unbond_from_portion.is_zero() {
                messages.push(adapter::unbond_msg(
                    &full_asset.contract.address.clone(),
                    unbond_from_portion,
                    meta.contract.clone(),
                )?);
                METRICS.pushf(deps.storage, env.block.time, Metric {
                    action: Action::Unbond,
                    context: Context::Unbond,
                    timestamp: env.block.time.seconds(),
                    token: asset.clone(),
                    amount: unbond_from_portion,
                    user: meta.contract.address.clone(),
                })?;
            }
        }
        return Ok(Response::new().add_messages(messages).set_data(to_binary(
            &adapter::ExecuteAnswer::Unbond {
                status: ResponseStatus::Success,
                amount,
            },
        )?));
    } else {
        // Otherwise we need to unbond everything from the portion adapters and go back to the
        // amount adapters
        for meta in portions {
            unbond_amounts.push(meta.balance);
            messages.push(adapter::unbond_msg(
                &full_asset.contract.address,
                meta.balance,
                meta.contract.clone(),
            )?);
            METRICS.pushf(deps.storage, env.block.time, Metric {
                action: Action::Unbond,
                context: Context::Unbond,
                timestamp: env.block.time.seconds(),
                token: asset.clone(),
                amount: meta.balance,
                user: meta.contract.address.clone(),
            })?;
            total_amount_unbonding += meta.balance;
        }
        if total_amount_unbonding == unbond_amount {
            for (i, meta) in amounts.clone().iter().enumerate() {
                if !unbond_amounts[i].is_zero() {
                    messages.push(adapter::unbond_msg(
                        &full_asset.contract.address,
                        unbond_amounts[i].clone(),
                        meta.contract.clone(),
                    )?);
                    METRICS.pushf(deps.storage, env.block.time, Metric {
                        action: Action::Unbond,
                        context: Context::Unbond,
                        timestamp: env.block.time.seconds(),
                        token: asset.clone(),
                        amount: unbond_amounts[i].clone(),
                        user: meta.contract.address.clone(),
                    })?;
                }
            }
            return Ok(Response::new().add_messages(messages).set_data(to_binary(
                &adapter::ExecuteAnswer::Unbond {
                    status: ResponseStatus::Success,
                    amount,
                },
            )?));
        } else {
            let mut amount_alloc = Uint128::zero();
            for meta in amounts.clone() {
                amount_alloc += meta.amount;
            }
            let mut modified_total_amount_unbonding = total_amount_unbonding;
            for (i, meta) in amounts.iter().enumerate() {
                unbond_amounts[i] += (unbond_amount - total_amount_unbonding)
                    .multiply_ratio(meta.amount, amount_alloc);

                modified_total_amount_unbonding += meta.balance;
                if i == amounts.len() - 1
                    && modified_total_amount_unbonding < unbond_amount
                    && unbond_amount - modified_total_amount_unbonding
                        < meta.balance - unbond_amounts[i]
                {
                    unbond_amounts[i] += unbond_amount - total_amount_unbonding;
                }
                messages.push(adapter::unbond_msg(
                    &full_asset.contract.address,
                    unbond_amounts[i],
                    meta.contract.clone(),
                )?);
                METRICS.pushf(deps.storage, env.block.time, Metric {
                    action: Action::Unbond,
                    context: Context::Unbond,
                    timestamp: env.block.time.seconds(),
                    token: asset.clone(),
                    amount: unbond_amounts[i].clone(),
                    user: meta.contract.address.clone(),
                })?;
            }
            return Ok(Response::new().add_messages(messages).set_data(to_binary(
                &adapter::ExecuteAnswer::Unbond {
                    status: ResponseStatus::Success,
                    amount,
                },
            )?));
        }
    }
}

pub fn add_holder(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    holder: Addr,
) -> StdResult<Response> {
    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryManager,
        &info.sender,
        &CONFIG.load(deps.storage)?.admin_auth,
    )?;

    HOLDERS.update(deps.storage, |mut h| {
        if h.contains(&holder.clone()) {
            return Err(StdError::generic_err("Holding already exists"));
        }
        h.push(holder.clone());
        Ok(h)
    })?;

    HOLDING.save(deps.storage, holder.clone(), &Holding {
        balances: Vec::new(),
        unbondings: Vec::new(),
        status: Status::Active,
    })?;
    METRICS.pushf(deps.storage, env.block.time, Metric {
        action: Action::AddHolder,
        context: Context::Holders,
        timestamp: env.block.time.seconds(),
        token: Addr::unchecked(""),
        amount: Uint128::zero(),
        user: holder,
    })?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::AddHolder {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn remove_holder(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    holder: Addr,
) -> StdResult<Response> {
    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryManager,
        &info.sender,
        &CONFIG.load(deps.storage)?.admin_auth,
    )?;

    if let Some(mut holding) = HOLDING.may_load(deps.storage, holder.clone())? {
        holding.status = Status::Closed;
        HOLDING.save(deps.storage, holder.clone(), &holding)?;
    } else {
        return Err(StdError::generic_err("Not an authorized holder"));
    }

    METRICS.pushf(deps.storage, env.block.time, Metric {
        action: Action::RemoveHolder,
        context: Context::Holders,
        timestamp: env.block.time.seconds(),
        token: Addr::unchecked(""),
        amount: Uint128::zero(),
        user: holder,
    })?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::RemoveHolder {
            status: ResponseStatus::Success,
        })?),
    )
}
