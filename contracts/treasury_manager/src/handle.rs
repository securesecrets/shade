use shade_protocol::c_std::{
    self,
    to_binary,
    Api,
    Binary,
    Env,
    DepsMut,
    Response,
    Addr,
    Querier,
    StdError,
    StdResult,
    Storage,
    Uint128,
};
use shade_protocol::{
    snip20::helpers::{
        allowance_query,
        batch::{SendFromAction, SendAction},
        balance_query,
        batch_send_from_msg,
        batch_send_msg,
        register_receive,
        send_msg,
        set_viewing_key_msg,
    },
};

use shade_protocol::{
    contract_interfaces::{
        dao::{
            adapter,
            treasury_manager::{
                storage::*,
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

use std::collections::HashMap;



pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _sender: Addr,
    from: Addr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<Response> {

    /* TODO
     * All assets received from a "holder" will be credited to their account
     * All other assets from all other addresses will be credited to the treasury (default account)
     */

    let config = CONFIG.load(&deps.storage)?;
    let asset = ASSETS.load(&deps.storage, env.message.sender.clone())?;

    // Do nothing if its an adapter (claimed funds)
    if let Some(adapter) = ALLOCATIONS.load(&deps.storage, env.message.sender.clone())?
        .iter()
        .find(|a| a.contract.address == from) {
            return Ok(HandleResponse {
                messages: vec![],
                log: vec![],
                data: Some(to_binary(&HandleAnswer::Receive {
                    status: ResponseStatus::Success,
                })?),
            });
        }

    // Default to treasury if not sent by a holder
    let holder = match HOLDERS.load(&deps.storage)?.contains(&from) {
        true => from,
        false => config.treasury,
    };

    // Update holdings
    HOLDING.update(&mut deps.storage, holder, |h| -> StdResult<Holding> {
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

    Ok(Response::new().set_data(to_binary(&HandleAnswer::Receive {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: Config,
) -> StdResult<Response> {
    let cur_config = CONFIG.load(&deps.storage)?;

    if info.sender != cur_config.admin {
        return Err(StdError::generic_err("unauthorized"));
    }

    CONFIG.save(&mut deps.storage, &config)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_register_asset(
    deps: DepsMut,
    env: &Env,
    contract: &Contract,
) -> StdResult<Response> {
    let config = CONFIG.load(&deps.storage)?;

    if info.sender != config.admin {
        return Err(StdError::generic_err("unauthorized"));
    }

    ASSET_LIST.update(&mut deps.storage, |mut list| -> StdResult<Vec<Addr>> {
        list.push(contract.address.clone());
        Ok(list)
    })?;

    ASSETS.save(&mut deps.storage, contract.address.clone(),
        &snip20::helpers::fetch_snip20(contract, &deps.querier)?,
    )?;

    ALLOCATIONS.save(&mut deps.storage, contract.address.clone(), &Vec::new())?;

    Ok(Response {
        messages: vec![
            // Register contract in asset
            register_receive(
                env.contract.code_hash.clone(),
                None,
                contract
            )?,
            // Set viewing key
            set_viewing_key_msg(
                VIEWING_KEY.load(&deps.storage)?,
                None,
                256,
                contract.code_hash.clone(),
                contract.address.clone(),
            )?,
        ],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RegisterAsset {
            status: ResponseStatus::Success,
        })?))
}

pub fn allocate(
    deps: DepsMut,
    env: &Env,
    asset: Addr,
    allocation: Allocation,
) -> StdResult<Response> {
    static ONE_HUNDRED_PERCENT: u128 = 10u128.pow(18);

    let config = CONFIG.load(&deps.storage)?;

    /* ADMIN ONLY */
    if info.sender != config.admin {
        return Err(StdError::generic_err("unauthorized"));
    }

    let mut apps = ALLOCATIONS.may_load(&deps.storage, asset.clone())?
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

    ALLOCATIONS.save(&mut deps.storage, asset.clone(), &apps)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::Allocate {
            status: ResponseStatus::Success,
        })?))
}

pub fn claim(
    deps: DepsMut,
    env: &Env,
    asset: Addr,
) -> StdResult<Response> {

    if !ASSET_LIST.load(&deps.storage)?.contains(&asset.clone()) {
        return Err(StdError::generic_err("Unrecognized asset"));
    }
    let full_asset = ASSETS.load(&deps.storage, asset.clone())?;

    let config = CONFIG.load(&deps.storage)?;
    let mut claimer = env.message.sender.clone();

    if claimer == config.admin {
        //assert!(false, "CLAIMER TREASURY");
        claimer = config.treasury;
    }

    let holders = HOLDERS.load(&deps.storage)?;

    if !holders.contains(&claimer.clone()) {
        return Err(StdError::unauthorized());
    }
    let mut holding = HOLDING.load(&deps.storage, claimer.clone())?;

    let unbonding_i = match holding.unbondings.iter_mut().position(|u| u.token == asset.clone()) {
        Some(i) => i,
        None => {
            return Err(StdError::generic_err(
                    format!("{} has no unbondings for {}",
                             claimer.clone(), asset.clone())
            ));
        }
    };

    let reserves = balance_query(
        &deps.querier,
        SELF_ADDRESS.load(&deps.storage)?,
        VIEWING_KEY.load(&deps.storage)?,
        1,
        full_asset.contract.code_hash.clone(),
        full_asset.contract.address.clone(),
    )?.amount;

    let mut messages = vec![];
    let mut total_claimed = Uint128::zero();

    // Claim if more funds are needed
    if holding.unbondings[unbonding_i].amount > reserves {
        //assert!(false, "reduce claim_amount {} - {}", unbonding.amount, reserves);
        let mut claim_amount = (holding.unbondings[unbonding_i].amount - reserves)?;

        for alloc in ALLOCATIONS.load(&deps.storage, asset.clone())? {
            if claim_amount == Uint128::zero() {
                break;
            }

            let claim = adapter::claimable_query(deps, &asset.clone(), alloc.contract.clone())?;

            if claim > Uint128::zero() {
                messages.push(adapter::claim_msg(asset.clone(), alloc.contract)?);
                if claim > claim_amount {
                    claim_amount = Uint128::zero();
                }
                else {
                    claim_amount = (claim_amount - claim)?;
                }
                total_claimed += claim;
            }
        }
    }

    let mut send_amount = Uint128::zero();

    if holding.unbondings[unbonding_i].amount > reserves + total_claimed {
        send_amount = reserves + total_claimed;
    }
    else {
        send_amount = holding.unbondings[unbonding_i].amount;
    }
    // Adjust unbonding amount
    holding.unbondings[unbonding_i].amount = (holding.unbondings[unbonding_i].amount - send_amount)?;
    HOLDING.save(&mut deps.storage, claimer.clone(), &holding)?;

    // Send claimed funds
    messages.push(
        send_msg(
            claimer.clone(),
            send_amount,
            None,
            None,
            None,
            1,
            full_asset.contract.code_hash.clone(),
            full_asset.contract.address.clone(),
        )?
    );

    Ok(Response::new().set_data(to_binary(&adapter::HandleAnswer::Claim {
            status: ResponseStatus::Success,
            amount: reserves + total_claimed,
        })?))
}

pub fn update(
    deps: DepsMut,
    env: &Env,
    asset: Addr,
) -> StdResult<Response> {
    let config = CONFIG.load(&deps.storage)?;

    let full_asset = ASSETS.load(&deps.storage, asset.clone())?;

    let mut allocations = ALLOCATIONS.load(&mut deps.storage, asset.clone())?;

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
    for h in HOLDERS.load(&deps.storage)? {
        let holder = HOLDING.load(&deps.storage, h)?;
        if let Some(u) = holder.unbondings.iter().find(|u| u.token == asset) {
            unbonding += u.amount;
        }
    }

    // Batch send_from actions
    let mut send_from_actions = vec![];
    let mut send_actions = vec![];
    let mut messages = vec![];

    let key = VIEWING_KEY.load(&deps.storage)?;

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
        SELF_ADDRESS.load(&deps.storage)?,
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
                    if desired_input <= threshold {
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

                    if !allowance.is_zero() {
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
    HOLDING.update(&mut deps.storage, config.treasury, |h| -> StdResult<Holding> {
        let mut holding = h.unwrap();
        if let Some(i) = holding.balances.iter().position(|u| u.token == asset) {
            holding.balances[i].amount = holding.balances[i].amount + allowance_used;
        }
        else {
            holding.balances.push(
                Balance {
                    token: asset,
                    amount: allowance_used,
                }
            );
        }
        Ok(holding)
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

    Ok(Response::new().set_data(to_binary(&adapter::HandleAnswer::Update {
            status: ResponseStatus::Success,
        })?))
}

pub fn unbond(
    deps: DepsMut,
    env: &Env,
    asset: Addr,
    amount: Uint128,
) -> StdResult<Response> {

    let config = CONFIG.load(&deps.storage)?;

    let mut unbonder = info.sender.clone();

    // admin unbonds on behalf of treasury
    if unbonder == config.admin {
        unbonder = config.treasury.clone();
    }

    let full_asset = ASSETS.load(&deps.storage, asset.clone())?;

    let holders = HOLDERS.load(&deps.storage)?;

    // Adjust holder balance
    if holders.contains(&unbonder.clone()) {
        let mut holding = HOLDING.load(&deps.storage, unbonder.clone())?;

        if holding.status != Status::Active {
            return Err(StdError::generic_err("Inactive Holding"));
        }

        let balance_i = match holding.balances.iter().position(|h| h.token == asset.clone()) {
            Some(i) => i,
            None => {
                return Err(StdError::generic_err(
                        format!("Cannot unbond, holder has no holdings of {}", asset.clone())
                ));
            }
        };


        // Check balance exceeds unbond amount
        if holding.balances[balance_i].amount < amount {
            return Err(StdError::generic_err("Not enough funds to unbond"));
        }

        else {
            holding.balances[balance_i].amount = (holding.balances[balance_i].amount - amount)?;
        }

        // Add unbonding
        if let Some(u) = holding.unbondings.iter().position(|h| h.token == asset.clone()) {
            holding.unbondings[u].amount += amount;
        }
        else {
            holding.unbondings.push(
                Balance {
                    token: asset.clone(),
                    amount,
                }
            );
        }

        HOLDING.save(&mut deps.storage, unbonder.clone(), &holding)?;
    }
    else {
        return Err(StdError::generic_err("unauthorized"));
    }

    let mut unbond_amount = amount;

    // get other holders unbonding amount to hold
    let mut other_unbondings = Uint128::zero();

    for h in holders {
        if h == unbonder.clone() {
            continue;
        }
        let other_holding = HOLDING.load(&deps.storage, h)?;
        if let Some(u) = other_holding.unbondings.iter().find(|u| u.token == asset.clone()) {
            other_unbondings += u.amount;
        }
    }

    // Reserves to be sent immediately
    let mut reserves = balance_query(
        &deps.querier,
        SELF_ADDRESS.load(&deps.storage)?,
        VIEWING_KEY.load(&deps.storage)?,
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
            HOLDING.update(&mut deps.storage, unbonder, |h| -> StdResult<Holding> {
                let mut holding = h.unwrap();
                if let Some(i) = holding.unbondings.iter().position(|u| u.token == asset) {
                    holding.unbondings[i].amount = (holding.unbondings[i].amount - reserves)?;
                }
                else {
                    return Err(StdError::generic_err("Failed to get unbonding, shouldn't happen"));
                }
                Ok(holding)
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
            HOLDING.update(&mut deps.storage, unbonder, |h| {
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

        let full_asset = ASSETS.load(&deps.storage, asset.clone())?;

        let mut allocations = ALLOCATIONS.load(&mut deps.storage, asset.clone())?;

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
            VIEWING_KEY.load(&deps.storage)?,
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
                    /*
                    let _desired_amount = total.multiply_ratio(
                        allocations[i].amount, 10u128.pow(18)
                    );
                    */

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

    Ok(Response::new().set_data(to_binary(&adapter::HandleAnswer::Unbond {
            status: ResponseStatus::Success,
            amount: unbond_amount,
        })?))
}

pub fn add_holder(
    deps: DepsMut,
    env: &Env,
    info: &MessageInfo,
    holder: Addr,
) -> StdResult<Response> {

    if info.sender != CONFIG.load(&deps.storage)?.admin {
        return Err(StdError::generic_err("unauthorized"));
    }

    HOLDERS.update(&mut deps.storage, |mut h| {
        if h.contains(&holder.clone()) {
            return Err(StdError::generic_err("Holding already exists"));
        }
        h.push(holder.clone());
        Ok(h)
    })?;

    HOLDING.save(&mut deps.storage, holder, &Holding {
        balances: Vec::new(),
        unbondings: Vec::new(),
        status: Status::Active,
    })?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::AddHolder {
            status: ResponseStatus::Success,
        })?))
}

pub fn remove_holder(
    deps: DepsMut,
    env: &Env,
    holder: Addr,
) -> StdResult<Response> {
    // TODO: unbond all or move all funds to treasury?
    // Should probably disallow fully deleting holders, just freeze/transfer
    if env.message.sender != CONFIG.load(&deps.storage)?.admin {
        return Err(StdError::unauthorized());
    }

    if let Some(mut holding) = HOLDING.may_load(&deps.storage, holder.clone())? {
        holding.status = Status::Closed;
        HOLDING.save(&mut deps.storage, holder, &holding)?;
    } else {
        return Err(StdError::generic_err("Not an authorized holder"));
    }

    Ok(Response::new().set_data(to_binary(&HandleAnswer::RemoveHolder {
            status: ResponseStatus::Success,
        })?))
}

/*
pub fn distribute_gain(
    gain: Uint128,
    token: Addr,
    holders: &mut HashMap<Addr, Holding>,
) -> StdResult<HashMap<Addr, Holding>> {
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
    token: Addr,
    holders: mut HashMap<Addr, Holding>,
) -> StdResult<Vec<Holding>> {
    let ratios = holding_ratios(&mut holders);

    Ok(holders)
}
*/

/* Builds a map of { Addr: <asset_portion * 10^18> }
 */
pub fn holding_shares(
    holdings: HashMap<Addr, Holding>,
    asset: Addr,
) -> HashMap<Addr, Uint128> {

    let mut ratios: HashMap<Addr, Uint128> = HashMap::new();
    let denominator = 10u128.pow(18);

    let total = holdings
        .iter()
        .map(|(addr, holding)| {
            match holding.balances.iter().find(|b| b.token == asset) {
                Some(b) => b.amount.u128(),
                None => 0u128,
            }
        }).sum::<u128>();

    for (addr, holding) in holdings {
        let balance = match holding.balances.iter().find(|b| b.token == asset) {
            Some(b) => b.amount,
            None => Uint128::zero(),
        };

        ratios.insert(addr, balance.multiply_ratio(10u128.pow(18), total));
    }

    ratios
}
