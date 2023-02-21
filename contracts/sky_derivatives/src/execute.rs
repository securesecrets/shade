use shade_protocol::c_std::{
    to_binary,
    Addr,
    CosmosMsg,
    Decimal,
    DepsMut,
    Env,
    MessageInfo,
    Response,
    StdError,
    StdResult,
    QuerierWrapper,
    Uint128,
};
use shade_protocol::{
    admin::helpers::{validate_admin, AdminPermissions},
    contract_interfaces::{
        dao::adapter,
        sky::{
            cycles::{ArbPair, Derivative, Offer},
            sky_derivatives::{
                Config,
                Direction,
                DexPairs,
                ExecuteAnswer,
                QueryAnswer,
                Unbondings,
                SelfAddr,
                TradingFees,
            },
        },
    },
    snip20::helpers::{send_msg, set_viewing_key_msg},
    utils::{
        asset::Contract, 
        generic_response::ResponseStatus,
        storage::plus::ItemStorage,
    },
};
use crate::query;

// token0 must be the original token, token1 must be the derivative
pub fn validate_dex_pair(derivative: &Derivative, pair: &ArbPair) -> bool {
    pair.token1 == derivative.contract
}

pub fn try_update_config(
    deps: DepsMut,
    info: MessageInfo,
    shade_admin_addr: Option<Contract>,
    treasury: Option<Addr>,
    derivative: Option<Derivative>,
    trading_fees: Option<TradingFees>,
    max_arb_amount: Option<Uint128>,
    viewing_key: Option<String>,
) -> StdResult<Response> {
    let cur_config = Config::load(deps.storage)?;

    // Admin Only
    validate_admin(
        &deps.querier,
        AdminPermissions::SkyAdmin, // TODO does this make sense????
        info.sender.to_string(),
        &cur_config.shade_admin_addr,
    )?;
    
    let mut messages = vec![];
    let config = Config {
        shade_admin_addr: match shade_admin_addr {
            Some(contract) => {
                // Verify new shade admins so contract doesn't break if new shade admin is broken
                // This means sender also has to have admin permission on new contract
                validate_admin(
                    &deps.querier,
                    AdminPermissions::SkyAdmin, // TODO does this make sense????
                    info.sender.to_string(),
                    &contract,
                )?;
                contract
            },
            None => cur_config.shade_admin_addr,
        },
        treasury: match treasury {
            Some(contract) => contract,
            None => cur_config.treasury,
        },
        derivative: match derivative {
            Some(ref deriv) => {
                // Clear dex pairs because new derivative will invalidate pairs
                DexPairs(vec![]).save(deps.storage)?;

                // If viewing key is also updated, it will be changed again below
                messages.push(set_viewing_key_msg(
                    cur_config.viewing_key.clone(),
                    None,
                    &deriv.contract,
                )?);
                messages.push(set_viewing_key_msg(
                    cur_config.viewing_key.clone(),
                    None,
                    &deriv.original_asset,
                )?);
                deriv.clone()
            },
            None => cur_config.derivative.clone(),
        },
        trading_fees: match trading_fees {
            Some(trading_fees) => {
                if trading_fees.dex_fee > Decimal::one() || trading_fees.stake_fee > Decimal::one()
                        || trading_fees.unbond_fee > Decimal::one() {
                    return Err(StdError::generic_err("Trading fee cannot be over 1.0"));
                }
                trading_fees
            },
            None => cur_config.trading_fees,
        },
        max_arb_amount: match max_arb_amount {
            Some(max) => max,
            None => cur_config.max_arb_amount,
        },
        viewing_key: match viewing_key {
            Some(key) => {
                println!("{} ::: {}", key, cur_config.viewing_key);
                set_viewing_keys(&derivative.unwrap_or(cur_config.derivative), &key)?;
                key
            },
            None => cur_config.viewing_key,
        },
    };
    config.save(deps.storage)?;

    Ok(Response::new()
        .set_data(to_binary(&ExecuteAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?)
        .add_messages(messages)
    )
}

pub fn set_viewing_keys(
    derivative: &Derivative,
    viewing_key: &String,
) -> StdResult<Vec<CosmosMsg>> {
    Ok(vec![
        set_viewing_key_msg(
            viewing_key.clone(),
            None,
            &derivative.original_asset,
        )?,
        set_viewing_key_msg(
            viewing_key.clone(),
            None,
            &derivative.contract,
        )?,
    ])
}

pub fn try_set_dex_pairs(
    deps: DepsMut,
    info: MessageInfo,
    pairs: Vec<ArbPair>,
) -> StdResult<Response> {
    let config = &Config::load(deps.storage)?;

    // Admin Only
    validate_admin(
        &deps.querier,
        AdminPermissions::SkyAdmin, // TODO does this make sense????
        info.sender.to_string(),
        &config.shade_admin_addr,
    )?;

    // Clear current pairs, then add individual (using try_add_pair's pair verification)
    let mut new_pairs = vec![];
    for pair in pairs {
        if !validate_dex_pair(&config.derivative, &pair) {
            return Err(StdError::generic_err("Invalid pair - does not match derivative"));
        }
        new_pairs.push(pair);
    }
    DexPairs(new_pairs).save(deps.storage)?;

    Ok(Response::new()
        .set_data(to_binary(&ExecuteAnswer::SetDexPairs {
            status: ResponseStatus::Success,
        })?)
    )
}

pub fn try_set_pair(
    deps: DepsMut,
    info: MessageInfo,
    pair: ArbPair,
    index: Option<usize>,
) -> StdResult<Response> {
    let config = &Config::load(deps.storage)?;

    // Admin Only
    validate_admin(
        &deps.querier,
        AdminPermissions::SkyAdmin, // TODO does this make sense????
        info.sender.to_string(),
        &Config::load(deps.storage)?.shade_admin_addr,
    )?;

    let i = match index {
        Some(i) => i,
        None => 0,
    };
    let mut pairs = DexPairs::load(deps.storage)?.0;
    if i >= pairs.len() {
        return Err(StdError::generic_err(format!("Invalid dex_pair index: {}", i)));
    }

    if !validate_dex_pair(&config.derivative, &pair) {
        return Err(StdError::generic_err("Invalid pair - does not match derivative"));
    }
    
    pairs[i] = pair;

    // TODO - Test if this is necessary
    // NOTE - this definitely should be necessary
    // storage::DEX_PAIRS.save(&mut deps.storage, pairs);

    Ok(Response::new()
        .set_data(to_binary(&ExecuteAnswer::SetPair {
            status: ResponseStatus::Success,
        })?)
    )
}

pub fn try_add_pair(
    deps: DepsMut,
    info: MessageInfo,
    pair: ArbPair,
) -> StdResult<Response> {
    let config = &Config::load(deps.storage)?;

    // Admin Only
    validate_admin(
        &deps.querier,
        AdminPermissions::SkyAdmin, // TODO does this make sense????
        info.sender.to_string(),
        &Config::load(deps.storage)?.shade_admin_addr,
    )?;

    if !validate_dex_pair(&config.derivative, &pair) {
        return Err(StdError::generic_err("Invalid pair - does not match derivative"));
    }

    let mut pairs = DexPairs::load(deps.storage)?.0;
    pairs.push(pair);

    // TODO - Test if this is necessary
    // NOTE - this definitely should be necessary
    // storage::DEX_PAIRS.save(&mut deps.storage, pairs);

    Ok(Response::new()
        .set_data(to_binary(&ExecuteAnswer::AddPair {
            status: ResponseStatus::Success,
        })?)
    )
}

pub fn try_remove_pair(
    deps: DepsMut,
    info: MessageInfo,
    index: usize,
) -> StdResult<Response> {
    // Admin Only
    validate_admin(
        &deps.querier,
        AdminPermissions::SkyAdmin, // TODO does this make sense????
        info.sender.to_string(),
        &Config::load(deps.storage)?.shade_admin_addr,
    )?;

    let mut pairs = DexPairs::load(deps.storage)?.0;
    if index >= pairs.len() {
        return Err(StdError::generic_err(format!("Invalid dex_pair index: {}", index)));
    }
    pairs.remove(index);
    
    Ok(Response::new()
        .set_data(to_binary(&ExecuteAnswer::RemovePair {
            status: ResponseStatus::Success,
        })?)
    )
}

// Helper function to return messages for arbitrage depending on profitability
fn arbitrage(
    querier: &QuerierWrapper,
    dex_pair: &ArbPair,
    config: &Config,
    self_addr: &Addr,
    unbondings: Uint128,
) -> StdResult<Vec<CosmosMsg>> {
    // Query balance to make sure arb doesn't use more than availabe balance
    let balance = config.derivative.query_original_balance(
        querier,
        self_addr.clone(),
        config.viewing_key.clone(),
    )?;

    let max_swap = Uint128::max(
        balance.saturating_sub(unbondings),
        config.max_arb_amount,
    );
    if max_swap.is_zero() {  // return early if no balance
        return Ok(vec![]) 
    }

    // Check profitability
    let is_profitable_result = query::is_arb_profitable(querier, &config, &dex_pair, Some(max_swap));
    let (profitable, swap_amounts_opt, direction_opt) = match is_profitable_result {
        Ok( QueryAnswer::IsProfitable { is_profitable, swap_amounts, direction } ) => 
            (is_profitable, swap_amounts, direction),
        _ => {
            return Err(StdError::generic_err("Invalid query return")); // This shouldn't happen
        }
    };
    if !profitable {  // Return failure (error not neccesary) if not profitable.
        return Ok(vec![])
    }

    let swap_amounts = match swap_amounts_opt {
        Some(amounts) => amounts,
        _ => {
            return Err(StdError::generic_err("Invalid query return"));
        }
    };
    let direction = match direction_opt {
        Some(direction) => direction,
        _ => {
            return Err(StdError::generic_err("Invalid query return"));
        }
    };

    // Execute arbitrage, create arbitrage messages depending on direction
    match direction {
        Direction::Unbond => {
            Ok(vec![
                dex_pair.to_cosmos_msg(
                    Offer {
                        asset: config.derivative.original_asset.clone(),
                        amount: swap_amounts.optimal_swap,
                    },
                    swap_amounts.swap1_result,
                )?,
                config.derivative.unbond_msg(swap_amounts.swap1_result)?,
            ])
        },
        Direction::Stake => {
            Ok(vec![
                config.derivative.stake_msg(swap_amounts.optimal_swap)?,
                dex_pair.to_cosmos_msg(
                    Offer {
                        asset: config.derivative.contract.clone(),
                        amount: swap_amounts.swap1_result,
                    },
                    swap_amounts.optimal_swap,
                )?
            ])
        },
    }
}

pub fn try_arb_pair(
    deps: DepsMut,
    _info: MessageInfo,
    index: Option<usize>,
) -> StdResult<Response> {
    let index = match index {
        Some(i) => i,
        None => 0,
    };
    let dex_pairs = DexPairs::load(deps.storage)?.0;
    if index >= dex_pairs.len() {
        return Err(StdError::generic_err(format!("Invalid dex_pair index: {}", index)));
    }

    let config = Config::load(deps.storage)?;
    let self_addr = SelfAddr::load(deps.storage)?.0;
    let unbondings = Unbondings::load(deps.storage)?.0;
    let messages = arbitrage(&deps.querier, &dex_pairs[index], &config, &self_addr, unbondings)?;
    let status = if messages.is_empty() {
        ResponseStatus::Success
    } else {
        ResponseStatus::Failure
    };

    Ok(Response::new()
        .add_messages(messages)
        .set_data(to_binary(&ExecuteAnswer::Arbitrage {
            status,
        })?)
    )
}

pub fn try_arb_all_pairs(
    deps: DepsMut, 
    _info: MessageInfo
) -> StdResult<Response> {
    let pairs = DexPairs::load(deps.storage)?.0;
    let mut statuses = vec![];
    let mut messages: Vec<CosmosMsg> = vec![];
    let config = Config::load(deps.storage)?;
    let self_addr = SelfAddr::load(deps.storage)?.0;
    let unbondings = Unbondings::load(deps.storage)?.0;
    for index in 0..pairs.len() {
        let response = arbitrage(&deps.querier, &pairs[index], &config, &self_addr, unbondings);
        match response {
            Ok(mut msgs) => {
                if msgs.is_empty() {
                    statuses.push(ResponseStatus::Success);
                } else {
                    statuses.push(ResponseStatus::Failure);
                }
                messages.append(&mut msgs);
            },
            Err(_) => {
                return Err(StdError::generic_err(
                        format!("Arbitrage issue on pair {}", index)
                        ));
            }
        }
    }

    Ok(Response::new()
        .add_messages(messages)
        .set_data(to_binary(&ExecuteAnswer::ArbAllPairs {
            statuses,
        })?)
    )
}

pub fn try_adapter_unbond(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    asset: Addr,
    amount: Uint128,
) -> StdResult<Response> {
    // Verify comes from treasury
    let config = Config::load(deps.storage)?;
    if config.treasury != info.sender {
        return Err(StdError::generic_err("unauthorized"));
    }

    // Send all of balance held up to amount. If remaining amount not accounted for, add that
    // amount to unbondings
    let derivative = config.derivative;
    if asset != derivative.original_asset.address {  // Only relevant token held
        return Err(StdError::generic_err("Unrecognized asset"));
    }

    let self_addr = SelfAddr::load(deps.storage)?.0;
    let balance = derivative.query_original_balance(
        &deps.querier, 
        self_addr.clone(), 
        config.viewing_key.clone(),
    )?;
    
    if balance.is_zero() {
        let unbondings = Unbondings::load(deps.storage)?.0;
        Unbondings(unbondings.checked_add(amount)?).save(deps.storage)?;

        return Ok(Response::new()
           .set_data(to_binary(&adapter::ExecuteAnswer::Unbond {
               status: ResponseStatus::Success,
               amount, // TODO: verify this amount makes sense even though none is claimed
           })?)
        )
    }

    let claimed = match amount.checked_sub(balance) {
        Ok(difference) => {
            let unbondings = Unbondings::load(deps.storage)?.0;
            Unbondings(unbondings.checked_add(difference)?).save(deps.storage)?;
            balance
        },
        _ => amount,
    };

    let message = send_msg(
        config.treasury,
        claimed,
        None,
        None,
        None,
        &derivative.original_asset,
    )?;

    Ok(Response::new()
        .set_data(to_binary(&adapter::ExecuteAnswer::Unbond {
            status: ResponseStatus::Success,
            amount,
        })?)
        .add_message(message)
    )
}

pub fn try_adapter_claim(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    asset: Addr,
) -> StdResult<Response> {
    // Verify comes from treasury
    let config = Config::load(deps.storage)?;
    if config.treasury != info.sender {
        return Err(StdError::generic_err("unauthorized"));
    }

    let derivative = config.derivative;
    if asset != derivative.original_asset.address {  // Only relevant token held
        return Err(StdError::generic_err("Unrecognized asset"));
    }

    // Send all of balance up to "Unbondings" 
    let self_addr = SelfAddr::load(deps.storage)?.0;
    let balance = derivative.query_original_balance(
        &deps.querier, 
        self_addr.clone(), 
        config.viewing_key.clone(),
    )?;

    if balance.is_zero() {
        return Ok(Response::new()
           .set_data(to_binary(&adapter::ExecuteAnswer::Unbond {
               status: ResponseStatus::Success,
               amount: Uint128::zero(),
           })?)
        )
    }

    let unbondings = Unbondings::load(deps.storage)?.0;
    let amount = Uint128::max(unbondings, balance);
    let message = send_msg(
        config.treasury,
        amount,
        None,
        None,
        None,
        &derivative.original_asset,
    )?;

    Ok(Response::new()
        .set_data(to_binary(&adapter::ExecuteAnswer::Claim {
            status: ResponseStatus::Success,
            amount,
        })?)
        .add_message(message)
    )
}

pub fn try_adapter_update(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _asset: Addr,
) -> StdResult<Response> {

    // Not necessary
    // TODO: verify this makes sense

    Ok(Response::new()
       .set_data(to_binary(&adapter::ExecuteAnswer::Update {
            status: ResponseStatus::Success,
        })?)
    )
}
