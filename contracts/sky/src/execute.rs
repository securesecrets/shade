use crate::query::{any_cycles_profitable, cycle_profitability};
use shade_protocol::{
    admin::helpers::{validate_admin, AdminPermissions},
    c_std::{
        to_binary,
        Addr,
        Decimal,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
        SubMsg,
        Uint128,
    },
    contract_interfaces::{
        dao::adapter,
        sky::{
            self,
            cycles::{Cycle, Offer},
            Config,
            Cycles,
            ExecuteAnswer,
            ViewingKeys,
        },
    },
    snip20::helpers::{send_msg, set_viewing_key_msg},
    utils::{
        asset::Contract,
        generic_response::ResponseStatus,
        storage::plus::ItemStorage,
        ExecuteCallback,
    },
};

pub fn try_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    shade_admin: Option<Contract>,
    shd_token: Option<Contract>,
    silk_token: Option<Contract>,
    sscrt_token: Option<Contract>,
    treasury: Option<Contract>,
    payback_rate: Option<Decimal>,
) -> StdResult<Response> {
    //Admin-only
    let mut config = Config::load(deps.storage)?;
    validate_admin(
        &deps.querier,
        AdminPermissions::SkyAdmin,
        info.sender.to_string(),
        &config.shade_admin,
    )?;

    let mut messages = vec![];

    if let Some(shade_admin) = shade_admin {
        config.shade_admin = shade_admin;
    }
    if let Some(shd_token) = shd_token {
        config.shd_token = shd_token;
        messages.push(SubMsg::new(set_viewing_key_msg(
            ViewingKeys::load(deps.storage)?.0,
            None,
            &config.shd_token.clone(),
        )?));
    }
    if let Some(silk_token) = silk_token {
        config.silk_token = silk_token;
        messages.push(SubMsg::new(set_viewing_key_msg(
            ViewingKeys::load(deps.storage)?.0,
            None,
            &config.silk_token.clone(),
        )?));
    }
    if let Some(sscrt_token) = sscrt_token {
        config.sscrt_token = sscrt_token;
        messages.push(SubMsg::new(set_viewing_key_msg(
            ViewingKeys::load(deps.storage)?.0,
            None,
            &config.sscrt_token.clone(),
        )?));
    }
    if let Some(treasury) = treasury {
        config.treasury = treasury;
    }
    if let Some(payback_rate) = payback_rate {
        if payback_rate == Decimal::zero() {
            return Err(StdError::generic_err("payback_rate cannot be zero"));
        }
        config.payback_rate = payback_rate;
    }
    config.save(deps.storage)?;
    Ok(Response::new()
        .set_data(to_binary(&ExecuteAnswer::UpdateConfig { status: true })?)
        .add_submessages(messages))
}

pub fn try_set_cycles(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cycles_to_set: Vec<Cycle>,
) -> StdResult<Response> {
    //Admin-only
    let shade_admin = Config::load(deps.storage)?.shade_admin;
    validate_admin(
        &deps.querier,
        AdminPermissions::SkyAdmin,
        info.sender.to_string(),
        &shade_admin,
    )?;

    if cycles_to_set.clone().len() > 40 {
        return Err(StdError::generic_err("Too many cycles"));
    }

    // validate cycles
    for cycle in cycles_to_set.clone() {
        cycle.validate_cycle()?;
    }

    let new_cycles = Cycles(cycles_to_set);
    new_cycles.save(deps.storage)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::SetCycles { status: true })?))
}

pub fn try_append_cycle(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cycles_to_add: Vec<Cycle>,
) -> StdResult<Response> {
    //Admin-only
    let shade_admin = Config::load(deps.storage)?.shade_admin;
    validate_admin(
        &deps.querier,
        AdminPermissions::SkyAdmin,
        info.sender.to_string(),
        &shade_admin,
    )?;

    for cycle in cycles_to_add.clone() {
        cycle.validate_cycle()?;
    }

    let mut cycles = Cycles::load(deps.storage)?;

    if cycles.0.clone().len() + cycles_to_add.clone().len() > 40 {
        return Err(StdError::generic_err("Too many cycles"));
    }

    cycles.0.append(&mut cycles_to_add.clone());

    cycles.save(deps.storage)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::AppendCycles { status: true })?))
}

pub fn try_update_cycle(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    cycle: Cycle,
    index: Uint128,
) -> StdResult<Response> {
    let i = index.u128() as usize;
    //Admin-only
    let shade_admin = Config::load(deps.storage)?.shade_admin;
    validate_admin(
        &deps.querier,
        AdminPermissions::SkyAdmin,
        info.sender.to_string(),
        &shade_admin,
    )?;

    cycle.validate_cycle()?;
    let mut cycles = Cycles::load(deps.storage)?;
    if i > cycles.0.clone().len() - 1 {
        return Err(StdError::generic_err("index out of bounds"));
    }
    cycles.0[i] = cycle;
    cycles.save(deps.storage)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::UpdateCycle { status: true })?))
}

pub fn try_remove_cycle(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    index: Uint128,
) -> StdResult<Response> {
    let i = index.u128() as usize;
    //Admin-only
    let shade_admin = Config::load(deps.storage)?.shade_admin;
    validate_admin(
        &deps.querier,
        AdminPermissions::SkyAdmin,
        info.sender.to_string(),
        &shade_admin,
    )?;

    // I'm pissed I couldn't do this in one line
    let mut cycles = Cycles::load(deps.storage)?.0;

    if i > cycles.clone().len() - 1 {
        return Err(StdError::generic_err("index out of bounds"));
    }

    cycles.remove(i);
    Cycles(cycles).save(deps.storage)?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::RemoveCycle { status: true })?))
}

pub fn try_arb_cycle(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Uint128,
    index: Uint128,
) -> StdResult<Response> {
    let mut messages = vec![];
    let mut return_swap_amounts = vec![];
    let mut payback_amount = Uint128::zero();
    let i = index.u128() as usize;
    // cur_asset will keep track of the asset that we currently "have"
    let mut cur_asset = Contract {
        address: info.sender.clone(),
        code_hash: "".to_string(),
    };

    // don't need to check for an index out of bounds since that check will happen in
    // cycle_profitability
    let res = cycle_profitability(deps.as_ref(), amount, index)?; // get profitability data from query
    match res {
        sky::QueryAnswer::IsCycleProfitable {
            is_profitable,
            direction,
            swap_amounts,
            profit,
        } => {
            return_swap_amounts = swap_amounts.clone();
            if direction.pair_addrs[0] // test to see which of the token attributes are the proposed starting addr
                .token0
                == direction.start_addr.clone()
            {
                cur_asset = direction.pair_addrs[0].token0.clone();
            } else {
                cur_asset = direction.pair_addrs[0].token1.clone();
            }
            // if tx is unprofitable, err out
            if !is_profitable {
                return Err(StdError::generic_err("Unprofitable"));
            }
            //loop through the pairs in the cycle
            for (i, arb_pair) in direction.pair_addrs.clone().iter().enumerate() {
                // if it's the last pair, set our minimum expected amount, otherwise, this field
                // should be zero
                if direction.pair_addrs.len() - 1 == i {
                    messages.push(SubMsg::new(arb_pair.to_cosmos_msg(
                        Offer {
                            asset: cur_asset.clone(),
                            amount: swap_amounts[i],
                        },
                        amount,
                    )?));
                } else {
                    messages.push(SubMsg::new(arb_pair.to_cosmos_msg(
                        Offer {
                            asset: cur_asset.clone(),
                            amount: swap_amounts[i],
                        },
                        Uint128::zero(),
                    )?));
                }
                // reset cur asset to the other asset held in the struct
                if cur_asset == arb_pair.token0.clone() {
                    cur_asset = arb_pair.token1.clone();
                } else {
                    cur_asset = arb_pair.token0.clone();
                }
            }
            // calculate payback amount
            payback_amount = profit * Config::load(deps.storage)?.payback_rate;

            // add the payback msg
            messages.push(SubMsg::new(send_msg(
                info.sender,
                Uint128::new(payback_amount.u128()),
                None,
                None,
                None,
                &cur_asset.clone(),
            )?));
        }
        _ => {}
    }

    // the final cur_asset should be the same as the start_addr
    if !(cur_asset.clone() == Cycles::load(deps.storage)?.0[i].start_addr) {
        return Err(StdError::generic_err(
            "final asset not equal to start asset",
        ));
    }

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::ExecuteArbCycle {
            status: true,
            swap_amounts: return_swap_amounts,
            payback_amount,
        })?),
    )
}

pub fn try_arb_all_cycles(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    amount: Uint128,
) -> StdResult<Response> {
    let mut total_profit = Uint128::zero();
    let mut messages = vec![];
    let res = any_cycles_profitable(deps.as_ref(), amount)?; // get profitability data from query
    match res {
        sky::QueryAnswer::IsAnyCycleProfitable {
            is_profitable,
            profit,
            ..
        } => {
            // loop through the data returned for each cycle
            for (i, profit_bool) in is_profitable.iter().enumerate() {
                // if a cycle is profitable call the try_arb_cycle fn and keep track of the
                // total_profit
                if profit_bool.clone() {
                    messages.push(SubMsg::new(
                        sky::ExecuteMsg::ArbCycle {
                            amount,
                            index: Uint128::from(i as u128),
                            padding: None,
                        }
                        .to_cosmos_msg(
                            &Contract {
                                address: env.contract.address.clone(),
                                code_hash: env.contract.code_hash.clone(),
                            },
                            vec![],
                        )?,
                    ));
                    total_profit = total_profit.clone().checked_add(profit[i])?;
                }
            }
        }
        _ => {}
    }
    // calculate payback_amount
    let payback_amount = total_profit * Config::load(deps.storage)?.payback_rate;
    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::ArbAllCycles {
            status: true,
            payback_amount,
        })?),
    )
}

pub fn try_adapter_unbond(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    asset: Addr,
    amount: Uint128,
) -> StdResult<Response> {
    let config = Config::load(deps.storage)?;
    // Error out if anyone other than the treasury is asking for money
    if !(info.sender == config.treasury.address) {
        return Err(StdError::generic_err("Unauthorized"));
    }
    // Error out if the treasury is asking for an asset sky doesn't account for
    if !(config.shd_token.address == asset
        || config.silk_token.address == asset
        || config.sscrt_token.address == asset)
    {
        return Err(StdError::generic_err("Unrecognized asset"));
    }
    // initialize this var to whichever token the treasury is asking for
    let contract;
    if config.shd_token.address == asset {
        contract = config.shd_token;
    } else if config.silk_token.address == asset {
        contract = config.silk_token;
    } else {
        contract = config.sscrt_token;
    }
    // send the msg
    let messages = vec![send_msg(
        config.treasury.address,
        Uint128::new(amount.u128()),
        None,
        None,
        None,
        &contract,
    )?];

    Ok(Response::new()
        .set_data(to_binary(&adapter::ExecuteAnswer::Unbond {
            status: ResponseStatus::Success,
            amount: Uint128::new(amount.u128()),
        })?)
        .add_messages(messages))
}

// Unessesary for sky
pub fn try_adapter_claim(_deps: DepsMut, _env: Env, _asset: Addr) -> StdResult<Response> {
    Ok(
        Response::new().set_data(to_binary(&adapter::ExecuteAnswer::Claim {
            status: ResponseStatus::Success,
            amount: Uint128::zero(),
        })?),
    )
}

// Unessesary for sky
pub fn try_adapter_update(_deps: DepsMut, _env: Env, _asset: Addr) -> StdResult<Response> {
    Ok(
        Response::new().set_data(to_binary(&adapter::ExecuteAnswer::Update {
            status: ResponseStatus::Success,
        })?),
    )
}
