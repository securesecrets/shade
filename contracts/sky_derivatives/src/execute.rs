use shade_protocol::c_std::{
    from_binary,
    to_binary,
    Addr,
    Decimal,
    DepsMut,
    Env,
    MessageInfo,
    Response,
    StdError,
    StdResult,
    Uint128,
};
use shade_admin::admin::{QueryMsg as AdminQueryMsg, ValidateAdminPermissionResponse};
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
                Rollover,
                SelfAddr,
                TradingFees,
            },
        },
    },
    utils::{
        asset::Contract, 
        generic_response::ResponseStatus,
        storage::plus::ItemStorage,
        ExecuteCallback,
        Query,
    },
};
use crate::query;

pub fn try_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    shade_admin_addr: Option<Contract>,
    derivative: Option<Derivative>,
    trading_fees: Option<TradingFees>,
    max_arb_amount: Option<Uint128>,
    arb_period: Option<u32>,
) -> StdResult<Response> {
    let cur_config = Config::load(deps.storage)?;

    // Admin Only
    validate_admin(
        &deps.querier,
        AdminPermissions::SkyAdmin, // TODO does this make sense????
        info.sender.to_string(),
        &cur_config.shade_admin_addr,
    )?;
    
    let config = Config {
        shade_admin_addr: match shade_admin_addr {
            Some(contract) => {
                // Verify new shade admins so contract doesn't rug if new shade admin is broken
                // This means sender also has to have admin permission on new contract
                validate_admin(
                    &deps.querier,
                    AdminPermissions::SkyAdmin, // TODO does this make sense????
                    info.sender.to_string(),
                    &cur_config.shade_admin_addr,
                )?;
                contract
            },
            None => cur_config.shade_admin_addr,
        },
        derivative: match derivative {
            Some(deriv) => {
                // Clear dex pairs because new derivative will invalidate pairs
                DexPairs(vec![]).save(deps.storage)?;
                deriv
            },
            None => cur_config.derivative,
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
        arb_period: match arb_period {
            Some(period) => period,
            None => cur_config.arb_period,
        },
    };
    config.save(deps.storage)?;

    Ok(Response::new()
       .set_data(to_binary(&ExecuteAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_set_dex_pairs(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pairs: Vec<ArbPair>,
) -> StdResult<Response> {
    // Admin Only
    validate_admin(
        &deps.querier,
        AdminPermissions::SkyAdmin, // TODO does this make sense????
        info.sender.to_string(),
        &Config::load(deps.storage)?.shade_admin_addr,
    )?;

    // Clear current pairs, then add individual (using try_add_pair's pair verification)
    DexPairs(vec![]).save(deps.storage)?;
    for pair in pairs {
        try_add_pair(deps, env, info, pair)?;
    }

    Ok(Response::new()
       .set_data(to_binary(&ExecuteAnswer::SetDexPairs {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_set_pair(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pair: ArbPair,
    index: Option<usize>,
) -> StdResult<Response> {
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

    // Validate dex pair
    let deriv = Config::load(deps.storage)?.derivative;
    if (pair.token0 != deriv.original_token || pair.token1 != deriv.contract)
        && (pair.token0 != deriv.contract || pair.token1 != deriv.original_token) {
        return Err(StdError::generic_err("Invalid pair - does not match derivative"));
    }

    pairs[i] = pair;
    // TODO - Test if this is necessary
    // storage::DEX_PAIRS.save(&mut deps.storage, pairs);

    Ok(Response::new()
       .set_data(to_binary(&ExecuteAnswer::SetPair {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_add_pair(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pair: ArbPair,
) -> StdResult<Response> {
    // Admin Only
    validate_admin(
        &deps.querier,
        AdminPermissions::SkyAdmin, // TODO does this make sense????
        info.sender.to_string(),
        &Config::load(deps.storage)?.shade_admin_addr,
    )?;


    // Validate dex pair
    let deriv = Config::load(deps.storage)?.derivative;
    if (pair.token0 != deriv.original_token || pair.token1 != deriv.contract)
        && (pair.token0 != deriv.contract || pair.token1 != deriv.original_token) {
        return Err(StdError::generic_err("Invalid pair - does not match derivative"));
    }

    let mut pairs = DexPairs::load(deps.storage)?.0;
    pairs.push(pair);
    // TODO - Test if this is necessary
    // storage::DEX_PAIRS.save(&mut deps.storage, pairs);

    Ok(Response::new()
       .set_data(to_binary(&ExecuteAnswer::SetDexPairs {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_remove_pair(
    deps: DepsMut,
    env: Env,
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
        })?))
}

pub fn try_arb_pair(
    deps: DepsMut,
    index: usize,
) -> StdResult<Response> {
    let dex_pairs = DexPairs::load(deps.storage)?.0;
    if index >= dex_pairs.len() {
        return Err(StdError::generic_err(format!("Invalid dex_pair index: {}", index)));
    }

    let rollover = Rollover::load(deps.storage)?.0;
    let periodically = Config::load(deps.storage)?.max_arb_amount;
    let max_swap = Uint128::from(rollover + periodically);

    let is_profitable_result = query::is_profitable(deps.as_ref(), index, Some(max_swap));
    let (profitable, swap_amounts_opt, direction_opt) = match is_profitable_result {
        Ok( QueryAnswer::IsProfitable { is_profitable, swap_amounts, direction } ) => 
            (is_profitable, swap_amounts, direction),
        _ => {
            return Err(StdError::generic_err("Invalid query return")); // This shouldn't happen
        }
    };

    // Return failure (error not neccesary) if not profitable.
    if !profitable {
        return Ok(Response::new()
                  .set_data(to_binary(&ExecuteAnswer::Arbitrage {
                status: ResponseStatus::Failure,
            })?))
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

    // create arbitrage messages depending on direction
    let deriv = Config::load(deps.storage)?.derivative;
    let mut messages = vec![];
    match direction {
        Direction::Unbond => {
            messages.push(dex_pairs[index].to_cosmos_msg(
                Offer {
                    asset: deriv.original_token.clone(),
                    amount: swap_amounts[0],
                },
                swap_amounts[1],
            )?);
            messages.push(deriv.unbond_msg(swap_amounts[1])?);
        },
        Direction::Stake => {
            messages.push(deriv.stake_msg(swap_amounts[0])?);
            messages.push(dex_pairs[index].to_cosmos_msg(
                Offer {
                    asset: deriv.contract,
                    amount: swap_amounts[1],
                },
                swap_amounts[0],
            )?);
        },
    };
    
    Ok(Response::new()
       .set_data(to_binary(&ExecuteAnswer::Arbitrage {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_arb_all_pairs(
    deps: DepsMut,
) -> StdResult<Response> {
    let pairs = DexPairs::load(deps.storage)?.0;
    let mut statuses = vec![];
    for index in 0..pairs.len() {
        statuses.push(match try_arb_pair(deps, index)?.data {
            Some(data) => {
                match from_binary(&data)? {
                    ExecuteAnswer::Arbitrage { status } => status,
                    _ => {
                        return Err(StdError::generic_err("Something went wrong with arbitrage"));
                    }
                }
            },
            _ => {
                return Err(StdError::generic_err("Something went wrong with arbitrage"));
            }
        });
    }

    Ok(Response::new()
       .set_data(to_binary(&ExecuteAnswer::ArbAllPairs {
            statuses,
        })?))
}

pub fn try_adapter_unbond(
    deps: DepsMut,
    env: Env,
    asset: Addr,
    amount: Uint128,
) -> StdResult<Response> {

    // TODO

    Ok(Response::new()
       .set_data(to_binary(&adapter::ExecuteAnswer::Unbond {
            status: ResponseStatus::Success,
            amount: shade_protocol::c_std::Uint128::from(amount.u128()),
        })?))
}

pub fn try_adapter_claim(
    deps: DepsMut,
    env: Env,
    asset: Addr,
) -> StdResult<Response> {

    // TODO
    
    Ok(Response::new()
       .set_data(to_binary(&adapter::ExecuteAnswer::Claim {
            status: ResponseStatus::Success,
            amount: shade_protocol::c_std::Uint128::zero(),
        })?))
}

pub fn try_adapter_update(
    deps: DepsMut,
    env: Env,
    asset: Addr,
) -> StdResult<Response> {

    // TODO

    Ok(Response::new()
       .set_data(to_binary(&adapter::ExecuteAnswer::Update {
            status: ResponseStatus::Success,
        })?))
}
