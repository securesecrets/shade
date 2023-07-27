use crate::storage::*;
use itertools::{Either, Itertools};
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
            AllocationTempData,
            AllocationType,
            Balance,
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
    utils::{
        asset::{Contract, RawContract},
        generic_response::ResponseStatus,
    },
};

static ONE_HUNDRED_PERCENT: Uint128 = Uint128::new(10u128.pow(18));

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
    let asset = match ASSETS.may_load(deps.storage, info.sender.clone())? {
        Some(a) => a,
        None => {
            return Err(StdError::generic_err("Not a registered asset"));
        }
    };

    METRICS.push(deps.storage, env.block.time, Metric {
        action: Action::FundsReceived,
        context: Context::Receive,
        timestamp: env.block.time.seconds(),
        token: info.sender.clone(),
        amount,
        user: from.clone(),
    })?;

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

    let mut holding = HOLDING.load(deps.storage, holder.clone())?;
    if holding.status == Status::Closed {
        return Err(StdError::generic_err(
            "Cannot add holdings when status is closed",
        ));
    }
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

    HOLDING.save(deps.storage, holder, &holding)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Receive {
        status: ResponseStatus::Success,
    })?))
}

pub fn update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin_auth: Option<RawContract>,
    treasury: Option<String>,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;

    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryManager,
        &info.sender,
        &config.admin_auth,
    )?;

    if let Some(admin_auth) = admin_auth {
        config.admin_auth = admin_auth.into_valid(deps.api)?;
    }
    if let Some(treasury) = treasury {
        config.treasury = deps.api.addr_validate(&treasury)?;
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::UpdateConfig {
            config,
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn register_asset(
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

    let mut list = ASSET_LIST.load(deps.storage)?;
    list.push(contract.address.clone());
    ASSET_LIST.save(deps.storage, &list)?;

    ASSETS.save(
        deps.storage,
        contract.address.clone(),
        &snip20::helpers::fetch_snip20(&contract, &deps.querier)?,
    )?;

    ALLOCATIONS.save(deps.storage, contract.address.clone(), &Vec::new())?;

    UNBONDINGS.save(deps.storage, contract.address.clone(), &Uint128::zero())?;

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
    _env: &Env,
    info: MessageInfo,
    asset: Addr,
    allocation: Allocation,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryManager,
        &info.sender,
        &config.admin_auth,
    )?;

    if allocation.tolerance >= ONE_HUNDRED_PERCENT {
        return Err(StdError::generic_err(format!(
            "Tolerance {} >= 100%",
            allocation.tolerance
        )));
    }

    let mut allocations = ALLOCATIONS
        .may_load(deps.storage, asset.clone())?
        .unwrap_or_default();

    // adapters can't have two allocations so remove the duplicate
    let stale_alloc = allocations
        .iter()
        .position(|a| a.contract.address == allocation.contract.address);

    match stale_alloc {
        Some(i) => {
            allocations.swap_remove(i);
        }
        None => {}
    };

    allocations.push(AllocationMeta {
        nick: allocation.nick,
        contract: allocation.contract,
        amount: allocation.amount,
        alloc_type: allocation.alloc_type,
        tolerance: allocation.tolerance,
    });

    // ensure that the portion allocations don't go above 100%
    if allocations
        .iter()
        .map(|a| {
            if a.alloc_type == AllocationType::Portion {
                a.amount
            } else {
                Uint128::zero()
            }
        })
        .sum::<Uint128>()
        > ONE_HUNDRED_PERCENT
    {
        return Err(StdError::generic_err(
            "Invalid allocation total exceeding 100%",
        ));
    }

    // Sort the allocations Amount < Portion
    allocations.sort_by(|a, b| match a.alloc_type {
        AllocationType::Amount => match b.alloc_type {
            AllocationType::Amount => std::cmp::Ordering::Equal,
            AllocationType::Portion => std::cmp::Ordering::Less,
        },
        AllocationType::Portion => match b.alloc_type {
            AllocationType::Amount => std::cmp::Ordering::Greater,
            AllocationType::Portion => std::cmp::Ordering::Equal,
        },
    });

    ALLOCATIONS.save(deps.storage, asset.clone(), &allocations)?;

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
    // if the claimer isn't a holder, it should default to the treasruy
    let claimer = match HOLDERS.load(deps.storage)?.contains(&info.sender) {
        true => info.sender,
        false => config.treasury.clone(),
    };

    let mut total_claimed = Uint128::zero();
    let mut messages = vec![];

    // claim from adapters that have claimable value
    for alloc in ALLOCATIONS.load(deps.storage, asset.clone())? {
        let claim = adapter::claimable_query(deps.querier, &asset, alloc.contract.clone())?;
        if claim > Uint128::zero() {
            messages.push(adapter::claim_msg(&asset, alloc.contract.clone())?);
            METRICS.push(deps.storage, env.block.time, Metric {
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

    let mut holding = HOLDING.load(deps.storage, claimer.clone())?;

    // get the position of the holders unbondings
    let unbonding_i = match holding
        .unbondings
        .iter_mut()
        .position(|u| u.token == asset.clone())
    {
        Some(i) => i,
        None => {
            return Ok(Response::new().add_messages(messages).set_data(to_binary(
                &adapter::ExecuteAnswer::Claim {
                    status: ResponseStatus::Success,
                    amount: Uint128::zero(),
                },
            )?));
        }
    };

    let reserves = balance_query(
        &deps.querier,
        env.contract.address.clone(),
        VIEWING_KEY.load(deps.storage)?,
        &full_asset.contract.clone(),
    )?;

    let send_amount = {
        // if reserves and total claimed is less than the unbondings of the holder, we need to send
        // all of the reserves and all that will be claimed
        if holding.unbondings[unbonding_i].amount > reserves + total_claimed {
            reserves + total_claimed
        } else {
            // otherwise just send the unbonding amount
            holding.unbondings[unbonding_i].amount
        }
    };

    // Adjust unbonding amount
    holding.unbondings[unbonding_i].amount = holding.unbondings[unbonding_i].amount - send_amount;

    if claimer != config.treasury && holding.status == Status::Closed {
        if let Some(balance_i) = holding
            .balances
            .iter_mut()
            .position(|u| u.token == asset.clone())
        {
            if holding.unbondings[unbonding_i].amount == Uint128::zero()
                && holding.balances[balance_i].amount == Uint128::zero()
            {
                holding.unbondings.swap_remove(unbonding_i);
                holding.balances.swap_remove(balance_i);
            }
        }
    }

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

    METRICS.push(deps.storage, env.block.time, Metric {
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

    // the sum of balances on 'amount' adapters
    let mut amount_total = Uint128::zero();
    // the sum of balances on 'portion' adapters
    let mut portion_total = Uint128::zero();
    // allocations marked for removal
    let mut stale_allocs = vec![];
    let mut messages = vec![];
    let mut adapter_info = vec![];

    /* this loop has 2 purposes
     * - check for stale allocaitons that need to be removed
     * - fill the amount_total and portion_total vars with data
     */
    for (i, a) in allocations.clone().iter().enumerate() {
        let bal = adapter::balance_query(
            deps.querier,
            &full_asset.contract.address,
            a.contract.clone(),
        )?;
        let mut unbonding = adapter::unbonding_query(
            deps.querier,
            &full_asset.contract.address,
            a.contract.clone(),
        )?;
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
            unbonding += claimable;
        }
        // if all these values are zero we can safely drop the alloc
        if bal.is_zero()
            && a.amount.is_zero()
            && unbonding.is_zero()
            && unbondable.is_zero()
            && claimable.is_zero()
        {
            stale_allocs.push(i);
        }

        adapter_info.push(AllocationTempData {
            contract: a.contract.clone(),
            alloc_type: a.alloc_type.clone(),
            amount: a.amount.clone(),
            tolerance: a.tolerance.clone(),
            balance: bal,
            unbondable,
            unbonding,
        });

        // fill totals with data
        match a.alloc_type {
            AllocationType::Amount => amount_total += bal,
            AllocationType::Portion => portion_total += bal,
        };
    }

    // actually drop the stale allocs
    if !stale_allocs.is_empty() {
        for index in stale_allocs.iter().rev() {
            // remove used here to preserve sorted vec
            allocations.remove(index.clone());
        }
        ALLOCATIONS.save(deps.storage, asset.clone(), &allocations)?;
    }

    // the holder is the entity that actually holds the tokens that the treasury manager can spend
    // holder_unbonding represents how much the holder has currently asked to unbond
    let mut holder_unbonding = Uint128::zero();
    // holder_principal represents how much of the asset has came form said holder
    let mut holder_principal = Uint128::zero();

    let mut holders = HOLDERS.load(deps.storage)?;
    // Withold holder unbondings
    for (i, h) in holders.clone().iter().enumerate() {
        // for each holder, load the respective holdings
        let holding = HOLDING.load(deps.storage, h.clone())?;
        // sum the data
        if let Some(u) = holding.unbondings.iter().find(|u| u.token == asset) {
            holder_unbonding += u.amount;
        }
        if let Some(b) = holding.balances.iter().find(|u| u.token == asset) {
            holder_principal += b.amount;
        }
        if holding.status == Status::Closed
            && holding.balances.len() == 0
            && holding.unbondings.len() == 0
        {
            HOLDING.remove(deps.storage, h.clone());
            holders.swap_remove(i);
            HOLDERS.save(deps.storage, &holders)?;
        }
    }

    // Batch send_from actions
    let mut send_from_actions = vec![];
    let mut send_actions = vec![];
    let mut metrics = vec![];

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
        env.contract.address.clone(),
        key.clone(),
        &full_asset.contract.clone(),
    )?;

    // total amount allocated to adapters + current snip20 balance
    // We subtract holder_unbonding to ensure that those tokens will be claimable
    let out_total = (amount_total + portion_total + balance) - holder_unbonding;
    // This gives us our total allowance from the treasury, used and unused
    let total = out_total + allowance;

    balance = {
        if balance > holder_unbonding {
            balance - holder_unbonding
        } else {
            Uint128::zero()
        }
    };

    // setting up vars
    let mut allowance_used = Uint128::zero();
    let mut balance_used = Uint128::zero();
    let mut reserved_for_amount_adapters = Uint128::zero();

    // loop through adapters with allocations
    for adapter in adapter_info {
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
                        .multiply_ratio(total - reserved_for_amount_adapters, ONE_HUNDRED_PERCENT)
                } else {
                    Uint128::zero()
                }
            }
        };
        // threshold is the desired_amount * a percentage held in adapter.tolerance,
        // the treasury manager will only attempt to rebalance if the adapter crosses the threshold
        // in either direction
        let threshold = desired_amount.multiply_ratio(adapter.tolerance, ONE_HUNDRED_PERCENT);

        // effective balance is the adapters' actual unbondable amount
        let effective_balance = {
            if adapter.balance > adapter.unbonding {
                adapter.balance - adapter.unbonding
            } else {
                // adapter balance should never be less than unbonding so if it's equal to then we
                // just set effective bal to zero
                Uint128::zero()
            }
        };

        match desired_amount.cmp(&effective_balance) {
            // Under Funded -- prioritize tm snip20 balance over allowance from treasury
            std::cmp::Ordering::Greater => {
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
                    metrics.push(Metric {
                        action: Action::SendFunds,
                        context: Context::Update,
                        timestamp: env.block.time.seconds(),
                        token: asset.clone(),
                        amount: desired_input,
                        user: adapter.contract.address.clone(),
                    });

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
                    metrics.push(Metric {
                        action: Action::SendFunds,
                        context: Context::Update,
                        timestamp: env.block.time.seconds(),
                        token: asset.clone(),
                        amount: balance,
                        user: adapter.contract.address.clone(),
                    });

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
                        metrics.push(Metric {
                            action: Action::SendFundsFrom,
                            context: Context::Update,
                            timestamp: env.block.time.seconds(),
                            token: asset.clone(),
                            amount: desired_input,
                            user: adapter.contract.address.clone(),
                        });

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
                        metrics.push(Metric {
                            action: Action::SendFundsFrom,
                            context: Context::Update,
                            timestamp: env.block.time.seconds(),
                            token: asset.clone(),
                            amount: allowance,
                            user: adapter.contract.address.clone(),
                        });

                        // account for allowance being sent out
                        allowance_used += allowance;
                        allowance = Uint128::zero();
                    }
                }
            }
            // Over funded -- unbond
            std::cmp::Ordering::Less => {
                // balance - target balance will give the amount we need to unbond
                let desired_output = effective_balance - desired_amount;

                // check to see that the threshold has been crossed
                if desired_output <= threshold {
                    continue;
                }

                if !desired_output.is_zero() {
                    messages.push(adapter::unbond_msg(
                        &asset.clone(),
                        desired_output.clone(),
                        adapter.contract.clone(),
                    )?);
                    metrics.push(Metric {
                        action: Action::Unbond,
                        context: Context::Update,
                        timestamp: env.block.time.seconds(),
                        token: asset.clone(),
                        amount: desired_output,
                        user: adapter.contract.address.clone(),
                    });
                }
                let unbondings = UNBONDINGS
                    .load(deps.storage, full_asset.contract.address.clone())?
                    + desired_output;
                UNBONDINGS.save(
                    deps.storage,
                    full_asset.contract.address.clone(),
                    &unbondings,
                )?;
            }
            _ => {}
        }
    }

    // Credit treasury balance with allowance used by adding allowance_used to the existing balance
    // or creating a new balance struct with allowance_used as the balance
    let mut holding = HOLDING.load(deps.storage, config.treasury.clone())?;
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
    HOLDING.save(deps.storage, config.treasury.clone(), &holding)?;

    // Determine Gainz & Losses & credit to treasury
    holder_principal += allowance_used;

    // this will never overflow because total is a sum of allowance
    match (total - allowance).cmp(&holder_principal) {
        std::cmp::Ordering::Greater => {
            let gains = (total - allowance) - holder_principal;
            // debit gains to treasury
            let mut holding = HOLDING.load(deps.storage, config.treasury.clone())?;
            if let Some(i) = holding.balances.iter().position(|u| u.token == asset) {
                holding.balances[i].amount += gains;
            }
            HOLDING.save(deps.storage, config.treasury.clone(), &holding)?;
            metrics.push(Metric {
                action: Action::RealizeGains,
                context: Context::Update,
                timestamp: env.block.time.seconds(),
                token: asset.clone(),
                amount: gains,
                user: config.treasury.clone(),
            });
        }
        std::cmp::Ordering::Less => {
            let losses = holder_principal - (total - allowance);
            // credit losses to treasury
            let mut holding = HOLDING.load(deps.storage, config.treasury.clone())?;
            if let Some(i) = holding.balances.iter().position(|u| u.token == asset) {
                holding.balances[i].amount -= losses;
            }
            HOLDING.save(deps.storage, config.treasury.clone(), &holding)?;
            metrics.push(Metric {
                action: Action::RealizeLosses,
                context: Context::Update,
                timestamp: env.block.time.seconds(),
                token: asset.clone(),
                amount: losses,
                user: config.treasury.clone(),
            });
        }
        _ => {}
    }

    // exec batch balance send messages
    if !send_actions.is_empty() {
        messages.push(batch_send_msg(
            send_actions,
            None,
            &full_asset.contract.clone(),
        )?);
    }

    // exec batch allowance send messages
    if !send_from_actions.is_empty() {
        messages.push(batch_send_from_msg(
            send_from_actions,
            None,
            &full_asset.contract.clone(),
        )?);
    }

    METRICS.append(deps.storage, env.block.time, &mut metrics)?;

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
    let holders = HOLDERS.load(deps.storage)?;

    // if the claimer isn't a holder, it should be an admin and default to the treasruy
    let unbonder = match holders.contains(&info.sender) {
        true => info.sender,
        false => {
            validate_admin(
                &deps.querier,
                AdminPermissions::TreasuryManager,
                &info.sender,
                &config.admin_auth,
            )?;
            config.treasury
        }
    };

    let full_asset = ASSETS.load(deps.storage, asset.clone())?;

    // Adjust holder balance
    let mut holding = HOLDING.load(deps.storage, unbonder.clone())?;

    // get the position of the balance for the asset
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

    let mut unbond_amount = amount;
    // Check balance exceeds unbond amount
    if holding.balances[balance_i].amount < amount {
        return Err(StdError::generic_err("Not enough funds to unbond"));
    } else {
        if holding.status == Status::Active {
            holding.balances[balance_i].amount = holding.balances[balance_i].amount - amount;
        } else {
            unbond_amount = holding.balances[balance_i].amount;
            holding.balances[balance_i].amount = Uint128::zero();
        }
    }

    // Add unbonding
    if let Some(u) = holding
        .unbondings
        .iter()
        .position(|h| h.token == asset.clone())
    {
        holding.unbondings[u].amount += unbond_amount;
    } else {
        holding.unbondings.push(Balance {
            token: asset.clone(),
            amount: unbond_amount,
        });
    }

    HOLDING.save(deps.storage, unbonder.clone(), &holding)?;
    let allocations = ALLOCATIONS.load(deps.storage, asset.clone())?;

    // get the total amount that the adapters are currently unbonding
    let mut unbonding_tot = Uint128::zero();
    for a in allocations.clone() {
        unbonding_tot +=
            adapter::unbonding_query(deps.querier, &asset.clone(), a.contract.clone())?;
    }

    // find the unbond_amount based off of amounts that the TM has unbonded independent of a holder
    unbond_amount = {
        let u = UNBONDINGS.load(deps.storage, full_asset.contract.address.clone())?;
        // if the independent unbondings is less than what the adapters are acutally unbonding, we
        // know another holder has asked to do some unbonding and the adapters are unbonding for
        // that holder
        if u <= unbonding_tot {
            if u <= unbond_amount {
                // if amount > independent unbonding, we reduce independent unbondings to
                // zero and return the amount we actually want to unbond from the adapters
                UNBONDINGS.save(
                    deps.storage,
                    full_asset.contract.address.clone(),
                    &Uint128::zero(),
                )?;
                unbond_amount - u
            } else {
                // independent unbondings covers the amount
                UNBONDINGS.save(
                    deps.storage,
                    full_asset.contract.address.clone(),
                    &(u - unbond_amount),
                )?;
                Uint128::zero()
            }
        } else {
            // We error out since this case is completely unexpected
            // Independent unbonding should never be greater than what the adapters are curretnly
            // unbonding
            /*return Err(StdError::generic_err(
                "Independent TM unbonding is greater than what the adapters are unbonding",
            ));*/
            // TODO figure out why we can't throw an error here
            // NOTE it has something to do with gains/losses
            unbond_amount
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
        env.contract.address.clone(),
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
    let mut metrics = vec![];

    // Send available reserves to unbonder
    if reserves > Uint128::zero() {
        if reserves < unbond_amount {
            // reserves can't cover unbond
            // Don't need batch send bc there's only one send msg
            messages.push(send_msg(
                unbonder.clone(),
                reserves,
                None,
                None,
                None,
                &full_asset.contract.clone(),
            )?);
            metrics.push(Metric {
                action: Action::SendFunds,
                context: Context::Unbond,
                timestamp: env.block.time.seconds(),
                token: asset.clone(),
                amount: reserves,
                user: unbonder.clone(),
            });
            unbond_amount = unbond_amount - reserves;

            // Reflect sent funds in unbondings
            let mut holding = HOLDING.load(deps.storage, unbonder.clone())?;
            if let Some(i) = holding.unbondings.iter().position(|u| u.token == asset) {
                holding.unbondings[i].amount = holding.unbondings[i].amount - reserves;
            }
            HOLDING.save(deps.storage, unbonder, &holding)?;
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
            metrics.push(Metric {
                action: Action::SendFunds,
                context: Context::Unbond,
                timestamp: env.block.time.seconds(),
                token: asset.clone(),
                amount,
                user: unbonder.clone(),
            });

            // Reflect sent funds in unbondings
            let mut holding = HOLDING.load(deps.storage, unbonder.clone())?;
            if let Some(i) = holding.unbondings.iter().position(|u| u.token == asset) {
                holding.unbondings[i].amount = holding.unbondings[i].amount - amount;
            }
            HOLDING.save(deps.storage, unbonder, &holding)?;

            METRICS.append(deps.storage, env.block.time, &mut metrics)?;
            return Ok(Response::new().add_messages(messages).set_data(to_binary(
                &adapter::ExecuteAnswer::Unbond {
                    status: ResponseStatus::Success,
                    amount,
                },
            )?));
        }
    }

    // let full_asset = ASSETS.load(deps.storage, asset.clone())?;

    // Build metadata
    let mut alloc_meta = vec![];
    let mut amount_total = Uint128::zero();
    let mut portion_total = Uint128::zero();
    let mut tot_unbond_available = Uint128::zero();

    // Gather adapter outstanding amounts
    for a in allocations {
        let bal = adapter::balance_query(deps.querier, &asset, a.contract.clone())?;
        let unbondable = adapter::unbondable_query(deps.querier, &asset, a.contract.clone())?;

        alloc_meta.push(AllocationTempData {
            contract: a.contract.clone(),
            alloc_type: a.alloc_type.clone(),
            amount: a.amount.clone(),
            tolerance: a.tolerance.clone(),
            balance: bal,
            unbondable,
            unbonding: Uint128::zero(),
        });

        tot_unbond_available += unbondable;

        match a.alloc_type {
            AllocationType::Amount => amount_total += bal,
            AllocationType::Portion => portion_total += bal,
        };
    }

    // if unbond_amount == tot_amount_unbonding, unbond all unbondable amounts and return
    if unbond_amount == tot_unbond_available {
        for a in alloc_meta.clone() {
            messages.push(adapter::unbond_msg(
                &full_asset.contract.address.clone(),
                a.unbondable.clone(),
                a.contract.clone(),
            )?);
            metrics.push(Metric {
                action: Action::Unbond,
                context: Context::Unbond,
                timestamp: env.block.time.seconds(),
                token: asset.clone(),
                amount: a.balance.clone(),
                user: a.contract.address.clone(),
            });
        }
        METRICS.append(deps.storage, env.block.time, &mut metrics)?;
        return Ok(Response::new().add_messages(messages).set_data(to_binary(
            &adapter::ExecuteAnswer::Unbond {
                status: ResponseStatus::Success,
                amount,
            },
        )?));
    }

    let mut total_amount_unbonding = Uint128::zero();
    let mut unbond_amounts = vec![];

    let (amounts, portions): (Vec<AllocationTempData>, Vec<AllocationTempData>) = alloc_meta
        .clone()
        .into_iter()
        .partition_map(|a| match a.alloc_type {
            AllocationType::Amount => Either::Left(a),
            AllocationType::Portion => Either::Right(a),
        });

    // unbond the extra tokens from the amount adapters
    for meta in amounts.clone() {
        if meta.unbondable > meta.amount {
            total_amount_unbonding += meta.unbondable - meta.amount;
            unbond_amounts.push(meta.unbondable - meta.amount);
        } else {
            unbond_amounts.push(Uint128::zero())
        }
    }

    // if the extra tokens from the amount adapters covers the unbond request, push the messages
    // and return
    if unbond_amount == total_amount_unbonding {
        for (i, meta) in amounts.clone().iter().enumerate() {
            messages.push(adapter::unbond_msg(
                &full_asset.contract.address.clone(),
                unbond_amounts[i],
                meta.contract.clone(),
            )?);
            metrics.push(Metric {
                action: Action::Unbond,
                context: Context::Unbond,
                timestamp: env.block.time.seconds(),
                token: asset.clone(),
                amount: unbond_amounts[i],
                user: meta.contract.address.clone(),
            });
        }
        METRICS.append(deps.storage, env.block.time, &mut metrics)?;
        return Ok(Response::new().add_messages(messages).set_data(to_binary(
            &adapter::ExecuteAnswer::Unbond {
                status: ResponseStatus::Success,
                amount,
            },
        )?));
    } else if unbond_amount < total_amount_unbonding {
        // if the extra tokens are greater than the unbond request, unbond proportionally to the
        // extra tokens available and return
        let mut modified_total_amount_unbonding = Uint128::zero();
        for (i, meta) in amounts.clone().iter().enumerate() {
            unbond_amounts[i] =
                unbond_amount.multiply_ratio(unbond_amounts[i], total_amount_unbonding);
            modified_total_amount_unbonding += unbond_amounts[i];
            // avoid off by one error
            if i == amounts.len() - 1
                && modified_total_amount_unbonding < unbond_amount
                && unbond_amounts[i] + Uint128::new(1) <= meta.unbondable
            {
                unbond_amounts[i] += Uint128::new(1);
            }
            messages.push(adapter::unbond_msg(
                &full_asset.contract.address.clone(),
                unbond_amounts[i],
                meta.contract.clone(),
            )?);
            metrics.push(Metric {
                action: Action::Unbond,
                context: Context::Unbond,
                timestamp: env.block.time.seconds(),
                token: asset.clone(),
                amount: unbond_amounts[i],
                user: meta.contract.address.clone(),
            });
        }
        METRICS.append(deps.storage, env.block.time, &mut metrics)?;
        return Ok(Response::new().add_messages(messages).set_data(to_binary(
            &adapter::ExecuteAnswer::Unbond {
                status: ResponseStatus::Success,
                amount,
            },
        )?));
    }

    // if portion total > unbond - tot, we know the portion adapters can cover the rest
    if unbond_amount - total_amount_unbonding < portion_total {
        // unbond the tokens slotted for unbonding from the amount adapters
        for (i, meta) in amounts.clone().iter().enumerate() {
            if !unbond_amounts[i].is_zero() {
                messages.push(adapter::unbond_msg(
                    &full_asset.contract.address.clone(),
                    unbond_amounts[i],
                    meta.contract.clone(),
                )?);
                metrics.push(Metric {
                    action: Action::Unbond,
                    context: Context::Unbond,
                    timestamp: env.block.time.seconds(),
                    token: asset.clone(),
                    amount: unbond_amounts[i],
                    user: meta.contract.address.clone(),
                });
            }
        }
        let amount_adapt_tot_unbonding = total_amount_unbonding;
        /* For each portion adapter, unbond the amount proportional to its portion of the total
         * balance
         */
        for (i, meta) in portions.clone().iter().enumerate() {
            let unbond_from_portion = (unbond_amount - amount_adapt_tot_unbonding)
                .multiply_ratio(meta.unbondable, portion_total);
            unbond_amounts.push(unbond_from_portion);
            total_amount_unbonding += unbond_from_portion;
            // Avoid off by 1 error
            if i == portions.len() - 1
                && total_amount_unbonding < unbond_amount
                && unbond_from_portion + Uint128::new(1) <= meta.unbondable
            {
                messages.push(adapter::unbond_msg(
                    &full_asset.contract.address.clone(),
                    unbond_from_portion + Uint128::new(1),
                    meta.contract.clone(),
                )?);
                metrics.push(Metric {
                    action: Action::Unbond,
                    context: Context::Unbond,
                    timestamp: env.block.time.seconds(),
                    token: asset.clone(),
                    amount: unbond_from_portion + Uint128::new(1),
                    user: meta.contract.address.clone(),
                });
            } else if !unbond_from_portion.is_zero() {
                messages.push(adapter::unbond_msg(
                    &full_asset.contract.address.clone(),
                    unbond_from_portion,
                    meta.contract.clone(),
                )?);
                metrics.push(Metric {
                    action: Action::Unbond,
                    context: Context::Unbond,
                    timestamp: env.block.time.seconds(),
                    token: asset.clone(),
                    amount: unbond_from_portion,
                    user: meta.contract.address.clone(),
                });
            }
        }
        METRICS.append(deps.storage, env.block.time, &mut metrics)?;
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
            unbond_amounts.push(meta.unbondable);
            if !meta.unbondable.is_zero() {
                messages.push(adapter::unbond_msg(
                    &full_asset.contract.address,
                    meta.unbondable,
                    meta.contract.clone(),
                )?);
                metrics.push(Metric {
                    action: Action::Unbond,
                    context: Context::Unbond,
                    timestamp: env.block.time.seconds(),
                    token: asset.clone(),
                    amount: meta.unbondable,
                    user: meta.contract.address.clone(),
                });
            }
            total_amount_unbonding += meta.unbondable;
        }
        // tot_amount_unbonding is equal to unbond_amount, unbonding everything from the portion
        // adapters covers our requested unbonding, so we push msgs and return
        if total_amount_unbonding == unbond_amount {
            for (i, meta) in amounts.clone().iter().enumerate() {
                if !unbond_amounts[i].is_zero() {
                    messages.push(adapter::unbond_msg(
                        &full_asset.contract.address,
                        unbond_amounts[i].clone(),
                        meta.contract.clone(),
                    )?);
                    metrics.push(Metric {
                        action: Action::Unbond,
                        context: Context::Unbond,
                        timestamp: env.block.time.seconds(),
                        token: asset.clone(),
                        amount: unbond_amounts[i].clone(),
                        user: meta.contract.address.clone(),
                    });
                }
            }
            METRICS.append(deps.storage, env.block.time, &mut metrics)?;
            return Ok(Response::new().add_messages(messages).set_data(to_binary(
                &adapter::ExecuteAnswer::Unbond {
                    status: ResponseStatus::Success,
                    amount,
                },
            )?));
        } else {
            // unbond token amounts proportional to the ratio of the allocation of the adapter and
            // the sum of the amount allocaitons
            let mut amount_alloc = Uint128::zero();
            for meta in amounts.clone() {
                amount_alloc += meta.amount;
            }
            let mut modified_total_amount_unbonding = total_amount_unbonding;
            for (i, meta) in amounts.iter().enumerate() {
                unbond_amounts[i] += (unbond_amount - total_amount_unbonding)
                    .multiply_ratio(meta.amount, amount_alloc);

                modified_total_amount_unbonding += meta.unbondable;
                // this makes sure that the entire unbond request is fuffiled by the end of this
                // block
                if i == amounts.len() - 1
                    && modified_total_amount_unbonding < unbond_amount
                    && unbond_amount - modified_total_amount_unbonding
                        < meta.unbondable - unbond_amounts[i]
                {
                    unbond_amounts[i] += unbond_amount - total_amount_unbonding;
                }
                if !unbond_amounts[i].is_zero() {
                    messages.push(adapter::unbond_msg(
                        &full_asset.contract.address,
                        unbond_amounts[i],
                        meta.contract.clone(),
                    )?);
                    metrics.push(Metric {
                        action: Action::Unbond,
                        context: Context::Unbond,
                        timestamp: env.block.time.seconds(),
                        token: asset.clone(),
                        amount: unbond_amounts[i].clone(),
                        user: meta.contract.address.clone(),
                    });
                }
            }
            METRICS.append(deps.storage, env.block.time, &mut metrics)?;
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
    let config = CONFIG.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryManager,
        &info.sender,
        &config.admin_auth,
    )?;

    let mut holders = HOLDERS.load(deps.storage)?;
    if holders.contains(&holder.clone()) {
        return Err(StdError::generic_err("Holder already exists"));
    }
    holders.push(holder.clone());
    HOLDERS.save(deps.storage, &holders)?;

    HOLDING.save(deps.storage, holder.clone(), &Holding {
        balances: Vec::new(),
        unbondings: Vec::new(),
        status: Status::Active,
    })?;

    METRICS.push(deps.storage, env.block.time, Metric {
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
    let config = CONFIG.load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::TreasuryManager,
        &info.sender,
        &config.admin_auth,
    )?;

    if holder == config.treasury {
        return Err(StdError::generic_err("Cannot remove treasury as a holder"));
    }

    if let Some(mut holding) = HOLDING.may_load(deps.storage, holder.clone())? {
        holding.status = Status::Closed;
        HOLDING.save(deps.storage, holder.clone(), &holding)?;
    } else {
        return Err(StdError::generic_err("Not an authorized holder"));
    }

    METRICS.push(deps.storage, env.block.time, Metric {
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
