use shade_protocol::c_std::{
    Storage, Api, Querier, DepsMut, Env, StdResult, Response, to_binary,
    StdError, Addr, CosmosMsg, Binary, WasmMsg
};
use shade_protocol::fadroma::scrt::to_cosmos_msg;
use shade_protocol::c_std::Uint128;
use shade_protocol::{
    utils::{asset::Contract, storage::plus::ItemStorage},
    contract_interfaces::{
    sky::sky::{
        Config, HandleAnswer, self
    },
    dex::sienna::{PairQuery, TokenTypeAmount, PairInfoResponse, TokenType, Swap, SwapOffer, CallbackMsg, CallbackSwap},
    mint::mint::{QueryAnswer, QueryMsg, QueryAnswer::Mint, ExecuteMsg::Receive, self},
    snip20::helpers::Snip20Asset,
}};
use shade_protocol::snip20::helpers::send_msg;
use crate::{query::trade_profitability};

pub fn try_update_config(
    deps: DepsMut,
    env: Env,
    shade_admin: Option<Contract>,
    shd_token: Option<Contract>,
    silk_token: Option<Contract>,
    sscrt_token: Option<Contract>,
    treasury: Option<Contract>,
    payback_rate: Option<Decimal>,
) -> StdResult<HandleResponse> {
    //Admin-only
    let mut config = Config::load(&mut deps.storage)?;
    let admin_response: ValidateAdminPermissionResponse =
        admin::QueryMsg::ValidateAdminPermission {
            contract_address: SelfAddr::load(&mut deps.storage)?.0.to_string(),
            admin_address: env.message.sender.to_string(),
        }
        .query(
            &deps.querier,
            config.shade_admin.code_hash.clone(),
            config.shade_admin.address.clone(),
        )?;

    if admin_response.error_msg.is_some() {
        return Err(StdError::unauthorized());
    }

    let mut messages = vec![];

    if let Some(shade_admin) = shade_admin {
        config.shade_admin = shade_admin;
    }
    if let Some(shd_token) = shd_token {
        config.shd_token = shd_token;
        messages.push(set_viewing_key_msg(
            ViewingKeys::load(&deps.storage)?.0,
            None,
            1,
            config.shd_token.code_hash.clone(),
            config.shd_token.address.clone(),
        )?);
    }
    if let Some(silk_token) = silk_token {
        config.silk_token = silk_token;
        messages.push(set_viewing_key_msg(
            ViewingKeys::load(&deps.storage)?.0,
            None,
            1,
            config.silk_token.code_hash.clone(),
            config.silk_token.address.clone(),
        )?);
    }
    if let Some(sscrt_token) = sscrt_token {
        config.sscrt_token = sscrt_token;
        messages.push(set_viewing_key_msg(
            ViewingKeys::load(&deps.storage)?.0,
            None,
            1,
            config.sscrt_token.code_hash.clone(),
            config.sscrt_token.address.clone(),
        )?);
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
    config.save(&mut deps.storage)?;
    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig { status: true })?),
    })
}

pub fn try_set_cycles<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    cycles_to_set: Vec<Cycle>,
) -> StdResult<HandleResponse> {
    //Admin-only
    let shade_admin = Config::load(&mut deps.storage)?.shade_admin;
    let admin_response: ValidateAdminPermissionResponse =
        admin::QueryMsg::ValidateAdminPermission {
            contract_address: SelfAddr::load(&mut deps.storage)?.0.to_string(),
            admin_address: env.message.sender.to_string(),
        }
        .query(&deps.querier, shade_admin.code_hash, shade_admin.address)?;

    if admin_response.error_msg.is_some() {
        return Err(StdError::unauthorized());
    }

    if cycles_to_set.clone().len() > 40 {
        return Err(StdError::generic_err("Too many cycles"));
    }

    // validate cycles
    for cycle in cycles_to_set.clone() {
        cycle.validate_cycle()?;
    }

    let new_cycles = Cycles(cycles_to_set);
    new_cycles.save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetCycles { status: true })?),
    })
}

pub fn try_append_cycle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    cycles_to_add: Vec<Cycle>,
) -> StdResult<HandleResponse> {
    //Admin-only
    let shade_admin = Config::load(&mut deps.storage)?.shade_admin;
    let admin_response: ValidateAdminPermissionResponse =
        admin::QueryMsg::ValidateAdminPermission {
            contract_address: SelfAddr::load(&mut deps.storage)?.0.to_string(),
            admin_address: env.message.sender.to_string(),
        }
        .query(&deps.querier, shade_admin.code_hash, shade_admin.address)?;

    if admin_response.error_msg.is_some() {
        return Err(StdError::unauthorized());
    }

    for cycle in cycles_to_add.clone() {
        cycle.validate_cycle()?;
    }

    let mut cycles = Cycles::load(&deps.storage)?;

    if cycles.0.clone().len() + cycles_to_add.clone().len() > 40 {
        return Err(StdError::generic_err("Too many cycles"));
    }

    cycles.0.append(&mut cycles_to_add.clone());

    cycles.save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AppendCycles { status: true })?),
    })
}

pub fn try_update_cycle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    cycle: Cycle,
    index: Uint128,
) -> StdResult<HandleResponse> {
    let i = index.u128() as usize;
    // Admin-only
    let shade_admin = Config::load(&mut deps.storage)?.shade_admin;
    let admin_response: ValidateAdminPermissionResponse =
        admin::QueryMsg::ValidateAdminPermission {
            contract_address: SelfAddr::load(&mut deps.storage)?.0.to_string(),
            admin_address: env.message.sender.to_string(),
        }
        .query(&deps.querier, shade_admin.code_hash, shade_admin.address)?;

    if admin_response.error_msg.is_some() {
        return Err(StdError::unauthorized());
    }

    cycle.validate_cycle()?;
    let mut cycles = Cycles::load(&deps.storage)?;
    if i > cycles.0.clone().len() - 1 {
        return Err(StdError::generic_err("index out of bounds"));
    }
    cycles.0[i] = cycle;
    cycles.save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateCycle { status: true })?),
    })
}

pub fn try_remove_cycle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    index: Uint128,
) -> StdResult<HandleResponse> {
    let i = index.u128() as usize;
    //Admin-only
    let shade_admin = Config::load(&mut deps.storage)?.shade_admin;
    let admin_response: ValidateAdminPermissionResponse =
        admin::QueryMsg::ValidateAdminPermission {
            contract_address: SelfAddr::load(&mut deps.storage)?.0.to_string(),
            admin_address: env.message.sender.to_string(),
        }
        .query(&deps.querier, shade_admin.code_hash, shade_admin.address)?;

    if admin_response.error_msg.is_some() {
        return Err(StdError::unauthorized());
    }

    // I'm pissed I couldn't do this in one line
    let mut cycles = Cycles::load(&deps.storage)?.0;

    if i > cycles.clone().len() - 1 {
        return Err(StdError::generic_err("index out of bounds"));
    }

    cycles.remove(i);
    Cycles(cycles).save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RemoveCycle { status: true })?),
    })
}

pub fn try_arb_cycle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
    index: Uint128,
) -> StdResult<HandleResponse> {
    let mut messages = vec![];
    let mut return_swap_amounts = vec![];
    let mut payback_amount = Uint128::zero();
    let i = index.u128() as usize;
    // cur_asset will keep track of the asset that we currently "have"
    let mut cur_asset = Contract {
        address: Addr::default(),
        code_hash: "".to_string(),
    };

    // don't need to check for an index out of bounds since that check will happen in
    // cycle_profitability
    let res = cycle_profitability(deps, amount, index)?; // get profitability data from query
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
                    messages.push(arb_pair.to_cosmos_msg(
                        Offer {
                            asset: cur_asset.clone(),
                            amount: swap_amounts[i],
                        },
                        amount,
                    )?);
                } else {
                    messages.push(arb_pair.to_cosmos_msg(
                        Offer {
                            asset: cur_asset.clone(),
                            amount: swap_amounts[i],
                        },
                        Uint128::zero(),
                    )?);
                }
                // reset cur asset to the other asset held in the struct
                if cur_asset == arb_pair.token0.clone() {
                    cur_asset = arb_pair.token1.clone();
                } else {
                    cur_asset = arb_pair.token0.clone();
                }
            }
            // calculate payback amount
            payback_amount = profit * Config::load(&deps.storage)?.payback_rate;

            // add the payback msg
            messages.push(send_msg(
                env.message.sender,
                c_std::Uint128(payback_amount.u128()),
                None,
                None,
                None,
                1,
                cur_asset.code_hash.clone(),
                cur_asset.address.clone(),
            )?);
        }
        _ => {}
    }

    // the final cur_asset should be the same as the start_addr
    if !(cur_asset.clone() == Cycles::load(&deps.storage)?.0[i].start_addr) {
        return Err(StdError::generic_err(
            "final asset not equal to start asset",
        ));
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ExecuteArbCycle {
            status: true,
            swap_amounts: return_swap_amounts,
            payback_amount,
        })?),
    })
}

pub fn try_arb_all_cycles<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let mut total_profit = Uint128::zero();
    let mut messages = vec![];
    let res = any_cycles_profitable(deps, amount)?; // get profitability data from query
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
                    messages.push(
                        sky::HandleMsg::ArbCycle {
                            amount,
                            index: Uint128::from(i as u128),
                            padding: None,
                        }
                        .to_cosmos_msg(
                            env.contract_code_hash.clone(),
                            env.contract.address.clone(),
                            None,
                        )?,
                    );
                    total_profit = total_profit.clone().checked_add(profit[i])?;
                }
            }
        }
        _ => {}
    }
    // calculate payback_amount
    let payback_amount = total_profit * Config::load(&deps.storage)?.payback_rate;
    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::ArbAllCycles {
            status: true,
            payback_amount,
        })?),
    })
}

pub fn try_adapter_unbond<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    asset: Addr,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let config = Config::load(&deps.storage)?;
    // Error out if anyone other than the treasury is asking for money
    if !(env.message.sender == config.treasury.address) {
        return Err(StdError::unauthorized());
    }
    // Error out if the treasury is asking for an asset sky doesn't account for
    if !(config.shd_token.address == asset
        || config.silk_token.address == asset
        || config.sscrt_token.address == asset)
    {
        return Err(StdError::GenericErr {
            msg: String::from("Unrecognized asset"),
            backtrace: None,
        });
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
        c_std::Uint128::from(amount.u128()),
        None,
        None,
        None,
        256,
        contract.code_hash,
        contract.address,
    )?];

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Unbond {
            status: ResponseStatus::Success,
            amount: c_std::Uint128::from(amount.u128()),
        })?),
    })
}

// Unessesary for sky
pub fn try_adapter_claim<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _asset: Addr,
) -> StdResult<HandleResponse> {
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Claim {
            status: ResponseStatus::Success,
            amount: c_std::Uint128::zero(),
        })?),
    })
}

// Unessesary for sky
pub fn try_adapter_update<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    _asset: Addr,
) -> StdResult<HandleResponse> {
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&adapter::HandleAnswer::Update {
            status: ResponseStatus::Success,
        })?),
    })
}
