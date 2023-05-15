use shade_protocol::c_std::{
    to_binary,
    Addr,
    BankQuery,
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
                TreasuryUnbondings,
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

// token0 must be the base token, token1 must be the derivative
pub fn validate_dex_pair(derivative: &Derivative, pair: &ArbPair) -> bool {
    pair.token1 == derivative.contract &&
        pair.token0 == derivative.base_asset &&
        pair.token1_decimals == derivative.deriv_decimals.into() &&
        pair.token0_decimals == derivative.base_decimals.into()
}

pub fn try_update_config(
    deps: DepsMut,
    info: MessageInfo,
    shade_admin_addr: Option<Contract>,
    treasury: Option<Addr>,
    derivative: Option<Derivative>,
    trading_fees: Option<TradingFees>,
    max_arb_amount: Option<Uint128>,
    min_profit_amount: Option<Uint128>,
    viewing_key: Option<String>,
) -> StdResult<Response> {
    let cur_config = Config::load(deps.storage)?;

    // Admin Only
    validate_admin(
        &deps.querier,
        AdminPermissions::SkyAdmin,
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
                ).map_err(|_| StdError::generic_err(
                        "New admin invalid, must have admin permissions on with new admin"
                ))?;
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
                messages = set_viewing_keys( // replace any current messages
                    deriv, 
                    &cur_config.viewing_key,
                )?;
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
        min_profit_amount: match min_profit_amount {
            Some(min) => min,
            None => cur_config.min_profit_amount,
        },
        viewing_key: match viewing_key {
            Some(key) => {
                messages = set_viewing_keys( // replace any current messages
                    &derivative.unwrap_or(cur_config.derivative), 
                    &key,
                )?;
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
            &derivative.base_asset,
        )?,
        set_viewing_key_msg(
            viewing_key.clone(),
            None,
            &derivative.contract,
        )?,
    ])
}

pub fn try_set_pairs(
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
            return Err(StdError::generic_err(
                "Invalid pair - original token must be token 0 and derivative must be token 1, decimals must match derivative"
            ));
        }
        new_pairs.push(pair);
    }
    DexPairs(new_pairs).save(deps.storage)?;

    Ok(Response::new()
        .set_data(to_binary(&ExecuteAnswer::SetPairs {
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
        return Err(StdError::generic_err(
            "Invalid pair - original token must be token 0 and derivative must be token 1, decimals must match derivative"
        ));
    }

    pairs[i] = pair;
    DexPairs(pairs).save(deps.storage)?;

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
        return Err(StdError::generic_err(
            "Invalid pair - original token must be token 0 and derivative must be token 1, decimals must match derivative"
        ));
    }

    let mut pairs = DexPairs::load(deps.storage)?.0;
    pairs.push(pair);
    DexPairs(pairs).save(deps.storage)?;

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
    DexPairs(pairs).save(deps.storage)?;
    
    Ok(Response::new()
        .set_data(to_binary(&ExecuteAnswer::RemovePair {
            status: ResponseStatus::Success,
        })?)
    )
}

struct ArbResult {
    messages: Vec<CosmosMsg>,
    arb_amount: Uint128,
    expected_profit: Uint128,
}

// Helper function to return messages for arbitrage depending on profitability
fn arbitrage(
    querier: &QuerierWrapper,
    dex_pair: &ArbPair,
    config: &Config,
    self_addr: &Addr,
    unbondings: Uint128,
) -> StdResult<ArbResult> {
    // Query balance to make sure arb doesn't use more than availabe balance
    let balance = config.derivative.query_base_balance(
        querier,
        self_addr.clone(),
        config.viewing_key.clone(),
    )?;

    let max_swap = Uint128::min(
        balance.saturating_sub(unbondings),
        config.max_arb_amount,
    );
    if max_swap.is_zero() {  // return early if no balance
        return Ok(ArbResult {
            messages: vec![],
            arb_amount: Uint128::zero(),
            expected_profit: Uint128::zero(),
        })
    }

    // Check profitability
    let is_profitable_result = query::is_arb_profitable(querier, &config, &dex_pair, max_swap)?;
    let (profitable, swap_amounts_opt, direction_opt) = match is_profitable_result {
        QueryAnswer::IsProfitable { is_profitable, swap_amounts, direction } => 
            (is_profitable, swap_amounts, direction),
        _ => {
            return Err(StdError::generic_err("Invalid query return")); // This shouldn't happen
        }
    };

    let swap_amounts = swap_amounts_opt.unwrap_or_default();
    if !profitable {  // Return failure (error not neccesary) if not profitable.
        return Ok(ArbResult {
            messages: vec![],
            arb_amount: swap_amounts.optimal_swap,
            expected_profit: swap_amounts.swap2_result
                .saturating_sub(swap_amounts.optimal_swap),
        }) 
    }

    let direction = match direction_opt {
        Some(direction) => direction,
        _ => {
            return Err(StdError::generic_err("Invalid query return"));
        }
    };

    // Execute arbitrage, create arbitrage messages depending on direction
    // Unbonding:
    //  1) swap sSCRT for stkd-SCRT
    //  2) unbond stkd-SCRT
    //      [ once funds unbonded, in later execute ]
    //  3) wrap SCRT
    //
    // Staking:
    //  1) unwrap sSCRT
    //  2) stake SCRT
    //  3) swap sktd-SCRT for sSCRT
    match direction {
        Direction::Unbond => {
            Ok(ArbResult {
                messages: vec![
                    dex_pair.to_cosmos_msg(
                        Offer {
                            asset: config.derivative.base_asset.clone(),
                            amount: swap_amounts.optimal_swap,
                        },
                        swap_amounts.swap1_result, // - BUFFER,
                    )?,
                    config.derivative.unbond_msg(swap_amounts.swap1_result)?,
                ],
                arb_amount: swap_amounts.optimal_swap,
                expected_profit: swap_amounts.swap2_result - swap_amounts.optimal_swap,
            })
        },
        Direction::Stake => {
            Ok(ArbResult {
                messages: vec![
                    config.derivative.unwrap_base(swap_amounts.optimal_swap)?,
                    config.derivative.stake_msg(swap_amounts.optimal_swap)?,
                    dex_pair.to_cosmos_msg(
                        Offer {
                            asset: config.derivative.contract.clone(),
                            amount: swap_amounts.swap1_result,
                        },
                        swap_amounts.optimal_swap,
                    )?,
                ],
                arb_amount: swap_amounts.optimal_swap,
                expected_profit: swap_amounts.swap2_result - swap_amounts.optimal_swap,
            })
        },
    }
}

pub fn try_arb_pair(
    deps: DepsMut,
    _info: MessageInfo,
    index: Option<usize>,
) -> StdResult<Response> {
    let index = index.unwrap_or_default();
    let dex_pairs = DexPairs::load(deps.storage)?.0;
    if index >= dex_pairs.len() {
        return Err(StdError::generic_err(format!("Invalid dex_pair index: {}", index)));
    }

    let config = Config::load(deps.storage)?;
    let self_addr = SelfAddr::load(deps.storage)?.0;
    let unbondings = TreasuryUnbondings::load(deps.storage)?.0;
    let arb_response = arbitrage(&deps.querier, &dex_pairs[index], &config, &self_addr, unbondings)?;
    let status = if arb_response.messages.is_empty() {
        ResponseStatus::Failure
    } else {
        ResponseStatus::Success
    };

    Ok(Response::new()
        .add_messages(arb_response.messages)
        .set_data(to_binary(&ExecuteAnswer::Arbitrage {
            status,
            arb_amount: arb_response.arb_amount,
            expected_profit: arb_response.expected_profit,
        })?)
    )
}

pub fn try_arb_all_pairs(
    deps: DepsMut, 
    _info: MessageInfo
) -> StdResult<Response> {
    let pairs = DexPairs::load(deps.storage)?.0;
    let mut statuses = vec![];
    let mut arb_amounts = vec![];
    let mut expected_profits = vec![];
    let mut messages: Vec<CosmosMsg> = vec![];

    let config = Config::load(deps.storage)?;
    let self_addr = SelfAddr::load(deps.storage)?.0;
    let unbondings = TreasuryUnbondings::load(deps.storage)?.0;
    for index in 0..pairs.len() {
        let response = arbitrage(&deps.querier, &pairs[index], &config, &self_addr, unbondings);
        match response {
            Ok(mut result) => {
                if result.messages.is_empty() {
                    statuses.push(ResponseStatus::Failure);
                } else {
                    statuses.push(ResponseStatus::Success);
                }
                messages.append(&mut result.messages);
                arb_amounts.push(result.arb_amount);
                expected_profits.push(result.expected_profit);
            },
            Err(err) => {
                return Err(StdError::generic_err(
                        format!("Arbitrage issue on pair {}, message: {}", index, err.to_string())));
            }
        }
    }

    Ok(Response::new()
        .add_messages(messages)
        .set_data(to_binary(&ExecuteAnswer::ArbAllPairs {
            statuses,
            arb_amounts,
            expected_profits,
        })?)
    )
}

pub fn try_adapter_unbond(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    asset: String,
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
    if asset != derivative.base_asset.address {  // Only relevant token held
        return Err(StdError::generic_err("Unrecognized asset"));
    }

    let self_addr = SelfAddr::load(deps.storage)?.0;
    let balance = derivative.query_base_balance(
        &deps.querier, 
        self_addr.clone(), 
        config.viewing_key.clone(),
    )?;

    // Cap treasury unbond to no more than unbondable
    let unbondable = match query::adapter_unbondable(deps.as_ref(), asset)? {
        adapter::QueryAnswer::Unbondable { amount } => amount,
        _ => Uint128::zero(), // shouldn't happen
    };
    let amount = Uint128::min(amount, unbondable);
    
    if balance.is_zero() {
        let unbondings = TreasuryUnbondings::load(deps.storage)?.0;
        TreasuryUnbondings(unbondings.checked_add(amount)?).save(deps.storage)?;

        return Ok(Response::new()
           .set_data(to_binary(&adapter::ExecuteAnswer::Unbond {
               status: ResponseStatus::Success,
               amount,
           })?)
        )
    }

    let claimed = match amount.checked_sub(balance) {
        Ok(difference) => {
            let unbondings = TreasuryUnbondings::load(deps.storage)?.0;
            TreasuryUnbondings(unbondings.checked_add(difference)?).save(deps.storage)?;
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
        &derivative.base_asset,
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
    asset: String,
) -> StdResult<Response> {
    // Verify comes from treasury
    let config = Config::load(deps.storage)?;
    if config.treasury != info.sender {
        return Err(StdError::generic_err("unauthorized"));
    }

    let derivative = config.derivative;
    if asset != derivative.base_asset.address {  // Only relevant token held
        return Err(StdError::generic_err("Unrecognized asset"));
    }

    // Send all of balance up to "Unbondings" 
    let self_addr = SelfAddr::load(deps.storage)?.0;
    let balance = derivative.query_base_balance(
        &deps.querier, 
        self_addr.clone(), 
        config.viewing_key.clone(),
    )?;

    let unbondings = TreasuryUnbondings::load(deps.storage)?.0;
    let amount = Uint128::min(unbondings, balance);
    let message = send_msg(
        config.treasury,
        amount,
        None,
        None,
        None,
        &derivative.base_asset,
    )?;

    if amount.is_zero() {
        return Ok(Response::new()
           .set_data(to_binary(&adapter::ExecuteAnswer::Claim {
               status: ResponseStatus::Failure, // Nothing to claim
               amount: Uint128::zero(),
           })?)
        )
    }

    let new_unbondings = unbondings - amount; // will not overflow
    TreasuryUnbondings(new_unbondings).save(deps.storage)?;

    Ok(Response::new()
        .set_data(to_binary(&adapter::ExecuteAnswer::Claim {
            status: ResponseStatus::Success,
            amount,
        })?)
        .add_message(message)
    )
}

pub fn try_adapter_update(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: String,
) -> StdResult<Response> {
    // Verify comes from treasury
    let config = Config::load(deps.storage)?;
    if config.treasury != info.sender {
        return Err(StdError::generic_err("unauthorized"));
    }

    let derivative = config.derivative;
    if asset != derivative.base_asset.address {
        return Err(StdError::generic_err("Unrecognized asset"));
    }

    // Check for left over bonded derivative.
    let self_addr = SelfAddr::load(deps.storage)?.0;
    let deriv_balance = derivative.query_deriv_balance(
        &deps.querier,
        self_addr.clone(),
        config.viewing_key.clone(),
    )?;

    // If derivative balance is nonzero, unbond for base asset.
    let mut messages = vec![];
    if !deriv_balance.is_zero() {
        messages.push(derivative.unbond_msg(deriv_balance)?);
    }

    // Claim any fully unbonded derivative
    let claimed = derivative.query_claimable(
        &deps.querier,
        self_addr.clone(),
        config.viewing_key.clone(),
        env.block.time.seconds(),
    )?;
    messages.push(derivative.claim_msg()?);

    let self_addr = SelfAddr::load(deps.storage)?.0;
    // TODO try to wrap all of L1 balance
    // TODO implement l1_balance query function in cycles
    BankQuery::Balance {
        address: self_addr.to_string(),
        denom: derivative.base_denom.clone(),
    };

    // Wrap base
    if !claimed.is_zero() {
        messages.push(derivative.wrap_base(claimed)?);
    }

    Ok(Response::new()
        .set_data(to_binary(&adapter::ExecuteAnswer::Update {
            status: ResponseStatus::Success,
        })?)
        .add_messages(messages),
    )
}
