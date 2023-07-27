use shade_protocol::{
    c_std::{Addr, Deps, StdError, StdResult, Uint128},
    contract_interfaces::{
        dao::adapter,
        sky::{
            cycles::{Offer},
            Config,
            Cycles,
            QueryAnswer,
            SelfAddr,
            ViewingKeys,
        },
        snip20,
    },
    utils::{storage::plus::ItemStorage, Query},
};

pub fn config(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: Config::load(deps.storage)?,
    })
}

pub fn get_balances(deps: Deps) -> StdResult<QueryAnswer> {
    let viewing_key = ViewingKeys::load(deps.storage)?.0;
    let self_addr = SelfAddr::load(deps.storage)?.0;
    let config = Config::load(deps.storage)?;

    // Query shd balance
    let mut res = snip20::QueryMsg::Balance {
        address: self_addr.clone().to_string(),
        key: viewing_key.clone(),
    }
    .query(&deps.querier, &config.shd_token.clone())?;

    let shd_bal = match res {
        snip20::QueryAnswer::Balance { amount } => amount,
        _ => Uint128::zero(),
    };

    // Query silk balance
    res = snip20::QueryMsg::Balance {
        address: self_addr.clone().to_string(),
        key: viewing_key.clone(),
    }
    .query(&deps.querier, &config.silk_token.clone())?;

    let silk_bal = match res {
        snip20::QueryAnswer::Balance { amount } => amount,
        _ => Uint128::zero(),
    };

    // Query sscrt balance
    res = snip20::QueryMsg::Balance {
        address: self_addr.clone().to_string(),
        key: viewing_key.clone(),
    }
    .query(&deps.querier, &config.sscrt_token.clone())?;

    let sscrt_bal = match res {
        snip20::QueryAnswer::Balance { amount } => amount,
        _ => Uint128::zero(),
    };

    Ok(QueryAnswer::Balance {
        shd_bal,
        silk_bal,
        sscrt_bal,
    })
}

pub fn get_cycles(deps: Deps) -> StdResult<QueryAnswer> {
    //Need to make private eventually
    Ok(QueryAnswer::GetCycles {
        cycles: Cycles::load(deps.storage)?.0,
    })
}

pub fn cycle_profitability(deps: Deps, amount: Uint128, index: Uint128) -> StdResult<QueryAnswer> {
    let mut cycles = Cycles::load(deps.storage)?.0;
    let mut swap_amounts = vec![amount];
    let i = index.u128() as usize;

    if (i) >= cycles.len() {
        return Err(StdError::generic_err("Index passed is out of bounds"));
    }

    // set up inital offer
    let mut current_offer = Offer {
        asset: cycles[i].start_addr.clone(),
        amount,
    };

    //loop through the pairs in the cycle
    for arb_pair in cycles[i].pair_addrs.clone() {
        // simulate swap will run a query with respect to which dex or minting that the pair says
        // it is
        let estimated_return = arb_pair
            .clone()
            .simulate_swap(deps, current_offer.clone())?;
        swap_amounts.push(estimated_return.clone());
        // set up the next offer with the other token contract in the pair and the expected return
        // from the last query
        if current_offer.asset.code_hash.clone() == arb_pair.token0.code_hash.clone() {
            current_offer = Offer {
                asset: arb_pair.token1.clone(),
                amount: estimated_return,
            };
        } else {
            current_offer = Offer {
                asset: arb_pair.token0.clone(),
                amount: estimated_return,
            };
        }
    }

    if swap_amounts.len() > cycles[i].pair_addrs.clone().len() {
        return Err(StdError::generic_err("More swap amounts than arb pairs"));
    }

    // if the last calculated swap is greater than the initial amount, return true
    if current_offer.amount.u128() > amount.u128() {
        return Ok(QueryAnswer::IsCycleProfitable {
            is_profitable: true,
            direction: cycles[i].clone(),
            swap_amounts,
            profit: current_offer.amount.checked_sub(amount)?,
        });
    }

    // reset these variables in order to check the other way
    swap_amounts = vec![amount];
    current_offer = Offer {
        asset: cycles[i].start_addr.clone(),
        amount,
    };

    // this is a fancy way of iterating through a vec in reverse
    for arb_pair in cycles[i].pair_addrs.clone().iter().rev() {
        // get the estimated return from the simulate swap function
        let estimated_return = arb_pair
            .clone()
            .simulate_swap(deps, current_offer.clone())?;
        swap_amounts.push(estimated_return.clone());
        // set the current offer to the other asset we are swapping into
        if current_offer.asset.code_hash.clone() == arb_pair.token0.code_hash.clone() {
            current_offer = Offer {
                asset: arb_pair.token1.clone(),
                amount: estimated_return,
            };
        } else {
            current_offer = Offer {
                asset: arb_pair.token0.clone(),
                amount: estimated_return,
            };
        }
    }

    // check to see if this direction was profitable
    if current_offer.amount > amount {
        // do an inplace reversal of the pair_addrs so that we know which way the opportunity goes
        cycles[i].pair_addrs.reverse();
        return Ok(QueryAnswer::IsCycleProfitable {
            is_profitable: true,
            direction: cycles[i].clone(),
            swap_amounts,
            profit: current_offer.amount.checked_sub(amount)?,
        });
    }

    // If both possible directions are unprofitable, return false
    Ok(QueryAnswer::IsCycleProfitable {
        is_profitable: false,
        direction: cycles[0].clone(),
        swap_amounts: vec![],
        profit: Uint128::zero(),
    })
}

pub fn any_cycles_profitable(deps: Deps, amount: Uint128) -> StdResult<QueryAnswer> {
    let cycles = Cycles::load(deps.storage)?.0;
    let mut return_is_profitable = vec![];
    let mut return_directions = vec![];
    let mut return_swap_amounts = vec![];
    let mut return_profit = vec![];

    // loop through the cycles with an index
    for index in 0..cycles.len() {
        // for each cycle, check its profitability
        let res = cycle_profitability(deps, amount, Uint128::from(index as u128)).unwrap();
        match res {
            QueryAnswer::IsCycleProfitable {
                is_profitable,
                direction,
                swap_amounts,
                profit,
            } => {
                if is_profitable {
                    // push the results to a vec
                    return_is_profitable.push(is_profitable);
                    return_directions.push(direction);
                    return_swap_amounts.push(swap_amounts);
                    return_profit.push(profit);
                }
            }
            _ => {
                return Err(StdError::generic_err("Unexpected result"));
            }
        }
    }

    Ok(QueryAnswer::IsAnyCycleProfitable {
        is_profitable: return_is_profitable,
        direction: return_directions,
        swap_amounts: return_swap_amounts,
        profit: return_profit,
    })
}

pub fn adapter_balance(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let config = Config::load(deps.storage)?;
    let viewing_key = ViewingKeys::load(deps.storage)?.0;
    let self_addr = SelfAddr::load(deps.storage)?.0;

    let contract;
    if config.shd_token.address == asset {
        contract = config.shd_token.clone();
    } else if config.silk_token.address == asset {
        contract = config.silk_token.clone();
    } else if config.sscrt_token.address == asset {
        contract = config.sscrt_token.clone();
    } else {
        return Ok(adapter::QueryAnswer::Unbondable {
            amount: Uint128::zero(),
        });
    }

    let res = snip20::QueryMsg::Balance {
        address: self_addr.clone().to_string(),
        key: viewing_key.clone(),
    }
    .query(&deps.querier, &contract.clone())?;

    let amount = match res {
        snip20::QueryAnswer::Balance { amount } => amount,
        _ => Uint128::zero(),
    };

    Ok(adapter::QueryAnswer::Unbondable {
        amount: Uint128::new(amount.u128()),
    })
}

pub fn adapter_claimable(_deps: Deps, _asset: Addr) -> StdResult<adapter::QueryAnswer> {
    Ok(adapter::QueryAnswer::Claimable {
        amount: Uint128::zero(),
    })
}

// Same as adapter_balance
pub fn adapter_unbondable(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let config = Config::load(deps.storage)?;
    let viewing_key = ViewingKeys::load(deps.storage)?.0;
    let self_addr = SelfAddr::load(deps.storage)?.0;

    let contract;
    if config.shd_token.address == asset {
        contract = config.shd_token.clone();
    } else if config.silk_token.address == asset {
        contract = config.silk_token.clone();
    } else if config.sscrt_token.address == asset {
        contract = config.sscrt_token.clone();
    } else {
        return Ok(adapter::QueryAnswer::Unbondable {
            amount: Uint128::zero(),
        });
    }

    let res = snip20::QueryMsg::Balance {
        address: self_addr.clone().to_string(),
        key: viewing_key.clone(),
    }
    .query(&deps.querier, &contract.clone())?;

    let amount = match res {
        snip20::QueryAnswer::Balance { amount } => amount,
        _ => Uint128::zero(),
    };

    Ok(adapter::QueryAnswer::Unbondable {
        amount: Uint128::new(amount.u128()),
    })
}

pub fn adapter_unbonding(_deps: Deps, _asset: Addr) -> StdResult<adapter::QueryAnswer> {
    Ok(adapter::QueryAnswer::Unbonding {
        amount: Uint128::zero(),
    })
}

// Same as adapter_balance
pub fn adapter_reserves(deps: Deps, asset: Addr) -> StdResult<adapter::QueryAnswer> {
    let config = Config::load(deps.storage)?;
    let viewing_key = ViewingKeys::load(deps.storage)?.0;
    let self_addr = SelfAddr::load(deps.storage)?.0;

    let contract;
    if config.shd_token.address == asset {
        contract = config.shd_token.clone();
    } else if config.silk_token.address == asset {
        contract = config.silk_token.clone();
    } else if config.sscrt_token.address == asset {
        contract = config.sscrt_token.clone();
    } else {
        return Ok(adapter::QueryAnswer::Unbondable {
            amount: Uint128::zero(),
        });
    }

    let res = snip20::QueryMsg::Balance {
        address: self_addr.clone().to_string(),
        key: viewing_key.clone(),
    }
    .query(&deps.querier, &contract.clone())?;

    let amount = match res {
        snip20::QueryAnswer::Balance { amount } => amount,
        _ => Uint128::zero(),
    };

    Ok(adapter::QueryAnswer::Unbondable {
        amount: Uint128::new(amount.u128()),
    })
}
