use shade_protocol::{
    c_std::{self, Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage},
    contract_interfaces::{
        dao::adapter,
        sky::{Config, Cycles, Offer, QueryAnswer, SelfAddr, ViewingKeys},
        snip20,
    },
    math_compat::Uint128,
    secret_toolkit::utils::Query,
    utils::storage::plus::ItemStorage,
};
use std::convert::TryInto;

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: Config::load(&deps.storage)?,
    })
}

pub fn get_balances<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    let viewing_key = ViewingKeys::load(&deps.storage)?.0;
    let self_addr = SelfAddr::load(&deps.storage)?.0;
    let config = Config::load(&deps.storage)?;

    let mut res = snip20::QueryMsg::Balance {
        address: self_addr.clone(),
        key: viewing_key.clone(),
    }
    .query(
        &deps.querier,
        config.shd_token_contract.code_hash.clone(),
        config.shd_token_contract.address.clone(),
    )?;

    let mut shd_bal = Uint128::new(0);

    match res {
        snip20::QueryAnswer::Balance { amount } => {
            shd_bal = amount.clone();
        }
        _ => {}
    }

    res = snip20::QueryMsg::Balance {
        address: self_addr.clone(),
        key: viewing_key.clone(),
    }
    .query(
        &deps.querier,
        config.silk_token_contract.code_hash.clone(),
        config.silk_token_contract.address.clone(),
    )?;

    let mut silk_bal = Uint128::new(0);

    match res {
        snip20::QueryAnswer::Balance { amount } => {
            silk_bal = amount;
        }
        _ => {}
    }

    res = snip20::QueryMsg::Balance {
        address: self_addr.clone(),
        key: viewing_key.clone(),
    }
    .query(
        &deps.querier,
        config.sscrt_token_contract.code_hash.clone(),
        config.sscrt_token_contract.address.clone(),
    )?;

    let mut sscrt_bal = Uint128::new(0);

    match res {
        snip20::QueryAnswer::Balance { amount } => {
            sscrt_bal = amount;
        }
        _ => {}
    }

    Ok(QueryAnswer::Balance {
        shd_bal,
        silk_bal,
        sscrt_bal,
    })
}

pub fn get_cycles<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    //Need to make private eventually
    Ok(QueryAnswer::GetCycles {
        cycles: Cycles::load(&deps.storage)?.0,
    })
}

pub fn cycle_profitability<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    amount: Uint128,
    index: Uint128,
) -> StdResult<QueryAnswer> {
    let mut cycles = Cycles::load(&deps.storage)?.0;
    let mut swap_amounts = vec![amount];

    if index.u128() > cycles.len().try_into().unwrap() {
        return Err(StdError::GenericErr {
            msg: "Index passed is out of bounds".to_string(),
            backtrace: None,
        });
    }

    // set up inital offer
    let mut current_offer = Offer {
        asset: cycles[index.u128() as usize].start_addr.clone(),
        amount,
    };

    //loop through the pairs in the cycle
    for arb_pair in cycles[index.u128() as usize].pair_addrs.clone() {
        // simulate swap will run a query with respect to which dex or minting that the pair says
        // it is
        let estimated_return = arb_pair
            .clone()
            .simulate_swap(&deps, current_offer.clone())?;
        swap_amounts.push(estimated_return.clone());
        // set up the next offer with the other token contract in the pair and the expected return
        // from the last query
        if current_offer.asset.code_hash.clone() == arb_pair.token0_contract.code_hash.clone() {
            current_offer = Offer {
                asset: arb_pair.token1_contract.clone(),
                amount: estimated_return,
            };
        } else {
            current_offer = Offer {
                asset: arb_pair.token0_contract.clone(),
                amount: estimated_return,
            };
        }
    }

    if swap_amounts.len() > cycles[index.u128() as usize].pair_addrs.clone().len() {
        return Err(StdError::GenericErr {
            msg: String::from("More swap amounts than arb pairs"),
            backtrace: None,
        });
    }

    // if the last calculated swap is greater than the initial amount, return true
    if current_offer.amount.u128() > amount.u128() {
        return Ok(QueryAnswer::IsCycleProfitable {
            is_profitable: true,
            direction: cycles[index.u128() as usize].clone(),
            swap_amounts,
            profit: current_offer.amount.checked_sub(amount)?,
        });
    }

    // reset these variables in order to check the other way
    swap_amounts = vec![amount];
    current_offer = Offer {
        asset: cycles[index.u128() as usize].start_addr.clone(),
        amount,
    };

    for arb_pair in cycles[index.u128() as usize]
        .pair_addrs
        .clone()
        .iter()
        .rev()
    {
        let estimated_return = arb_pair
            .clone()
            .simulate_swap(&deps, current_offer.clone())?;
        swap_amounts.push(estimated_return.clone());
        if current_offer.asset.code_hash.clone() == arb_pair.token0_contract.code_hash.clone() {
            current_offer = Offer {
                asset: arb_pair.token1_contract.clone(),
                amount: estimated_return,
            };
        } else {
            current_offer = Offer {
                asset: arb_pair.token0_contract.clone(),
                amount: estimated_return,
            };
        }
    }

    if swap_amounts.len() > cycles[index.u128() as usize].pair_addrs.clone().len() {
        return Err(StdError::GenericErr {
            msg: String::from("More swap amounts than arb pairs"),
            backtrace: None,
        });
    }

    if current_offer.amount > amount {
        cycles[index.u128() as usize].pair_addrs.reverse();
        return Ok(QueryAnswer::IsCycleProfitable {
            is_profitable: true,
            direction: cycles[index.u128() as usize].clone(),
            swap_amounts,
            profit: current_offer.amount.checked_sub(amount)?,
        });
    }

    Ok(QueryAnswer::IsCycleProfitable {
        is_profitable: false,
        direction: cycles[0].clone(),
        swap_amounts: vec![],
        profit: Uint128::zero(),
    })
}

pub fn any_cycles_profitable<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    amount: Uint128,
) -> StdResult<QueryAnswer> {
    let cycles = Cycles::load(&deps.storage)?.0;
    let mut return_is_profitable = vec![];
    let mut return_directions = vec![];
    let mut return_swap_amounts = vec![];
    let mut return_profit = vec![];

    for index in 0..cycles.len() {
        let res = cycle_profitability(deps, amount, Uint128::from(index as u128)).unwrap();
        match res {
            QueryAnswer::IsCycleProfitable {
                is_profitable,
                direction,
                swap_amounts,
                profit,
            } => {
                if is_profitable {
                    return_is_profitable.push(is_profitable);
                    return_directions.push(direction);
                    return_swap_amounts.push(swap_amounts);
                    return_profit.push(profit);
                }
            }
            _ => {
                return Err(StdError::GenericErr {
                    msg: "Unexpected result".to_string(),
                    backtrace: None,
                });
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

pub fn adapter_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    let config = Config::load(&deps.storage)?;
    if !(config.shd_token_contract.address == asset
        || config.silk_token_contract.address == asset
        || config.sscrt_token_contract.address == asset)
    {
        return Err(StdError::GenericErr {
            msg: String::from("Unrecognized asset"),
            backtrace: None,
        });
    }
    let res = get_balances(deps)?;
    let mut amount = Uint128::zero();
    match res {
        QueryAnswer::Balance {
            shd_bal,
            silk_bal,
            sscrt_bal,
        } => {
            if config.shd_token_contract.address == asset {
                amount = shd_bal;
            } else if config.silk_token_contract.address == asset {
                amount = silk_bal;
            } else {
                amount = sscrt_bal;
            }
        }
        _ => {}
    }
    Ok(adapter::QueryAnswer::Balance {
        amount: c_std::Uint128(amount.u128()),
    })
}

pub fn adapter_claimable<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    _asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    Ok(adapter::QueryAnswer::Claimable {
        amount: c_std::Uint128::zero(),
    })
}

pub fn adapter_unbondable<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    let config = Config::load(&deps.storage)?;
    if !(config.shd_token_contract.address == asset
        || config.silk_token_contract.address == asset
        || config.sscrt_token_contract.address == asset)
    {
        return Err(StdError::GenericErr {
            msg: String::from("Unrecognized asset"),
            backtrace: None,
        });
    }
    let res = get_balances(deps)?;
    let mut amount = Uint128::zero();
    match res {
        QueryAnswer::Balance {
            shd_bal,
            silk_bal,
            sscrt_bal,
        } => {
            if config.shd_token_contract.address == asset {
                amount = shd_bal;
            } else if config.silk_token_contract.address == asset {
                amount = silk_bal;
            } else {
                amount = sscrt_bal;
            }
        }
        _ => {}
    }
    Ok(adapter::QueryAnswer::Unbondable {
        amount: c_std::Uint128(amount.u128()),
    })
}

pub fn adapter_unbonding<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    _asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    Ok(adapter::QueryAnswer::Unbonding {
        amount: c_std::Uint128::zero(),
    })
}

pub fn adapter_reserves<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    let config = Config::load(&deps.storage)?;
    if !(config.shd_token_contract.address == asset
        || config.silk_token_contract.address == asset
        || config.sscrt_token_contract.address == asset)
    {
        return Err(StdError::GenericErr {
            msg: String::from("Unrecognized asset"),
            backtrace: None,
        });
    }
    let res = get_balances(deps)?;
    let mut amount = Uint128::zero();
    match res {
        QueryAnswer::Balance {
            shd_bal,
            silk_bal,
            sscrt_bal,
        } => {
            if config.shd_token_contract.address == asset {
                amount = shd_bal;
            } else if config.silk_token_contract.address == asset {
                amount = silk_bal;
            } else {
                amount = sscrt_bal;
            }
        }
        _ => {}
    }
    Ok(adapter::QueryAnswer::Reserves {
        amount: c_std::Uint128(amount.u128()),
    })
}
