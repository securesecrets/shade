use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{debug_print, Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage};
use secret_toolkit::utils::Query;
use shade_protocol::{
    contract_interfaces::{
        dao::adapter,
        dex::shadeswap::{self, TokenAmount, TokenType},
        mint::mint,
        sky::sky::{Config, Cycles, QueryAnswer, SelfAddr, ViewingKeys},
        snip20,
    },
    utils::storage::plus::ItemStorage,
};
use std::convert::TryInto;

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: Config::load(&deps.storage)?,
    })
}

pub fn conversion_mint_profitability<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    amount: Uint128,
) -> StdResult<QueryAnswer> {
    let config: Config = Config::load(&deps.storage)?;
    let mut first_swap_result;

    let res = mint::QueryMsg::Mint {
        offer_asset: config.shd_token_contract.address.clone(),
        amount,
    }
    .query(
        &deps.querier,
        config.mint_contract_silk.code_hash.clone(),
        config.mint_contract_silk.address.clone(),
    )?;

    match res {
        mint::QueryAnswer::Mint { amount, .. } => {
            first_swap_result = amount;
        }
        _ => {
            return Err(StdError::GenericErr {
                msg: String::from("Unexpected query result"),
                backtrace: None,
            });
        }
    }

    let mut offer = TokenAmount {
        token: shadeswap::TokenType::CustomToken {
            contract_addr: config.silk_token_contract.address.clone(),
            token_code_hash: config.silk_token_contract.code_hash.clone(),
        },
        amount: first_swap_result,
    };

    let mut res2 = shadeswap::PairQuery::GetEstimatedPrice { offer }.query(
        &deps.querier,
        config.market_swap_contract.code_hash.clone(),
        config.market_swap_contract.address.clone(),
    )?;

    let mut final_amount;

    match res2 {
        shadeswap::QueryMsgResponse::EstimatedPrice { estimated_price } => {
            final_amount = estimated_price;
        }
        _ => {
            return Err(StdError::GenericErr {
                msg: String::from("unexpected query result"),
                backtrace: None,
            });
        }
    }

    if final_amount > amount {
        return Ok(QueryAnswer::ArbPegProfitability {
            is_profitable: true,
            mint_first: true,
            first_swap_result,
            profit: final_amount.checked_sub(amount)?,
        });
    }

    offer = TokenAmount {
        token: shadeswap::TokenType::CustomToken {
            contract_addr: config.shd_token_contract.address.clone(),
            token_code_hash: config.shd_token_contract.code_hash.clone(),
        },
        amount,
    };

    res2 = shadeswap::PairQuery::GetEstimatedPrice { offer }.query(
        &deps.querier,
        config.market_swap_contract.code_hash.clone(),
        config.market_swap_contract.address.clone(),
    )?;

    match res2 {
        shadeswap::QueryMsgResponse::EstimatedPrice { estimated_price } => {
            first_swap_result = estimated_price;
        }
        _ => {
            return Err(StdError::GenericErr {
                msg: String::from("unexpected query response"),
                backtrace: None,
            });
        }
    }

    let res = mint::QueryMsg::Mint {
        offer_asset: config.silk_token_contract.address.clone(),
        amount: first_swap_result,
    }
    .query(
        &deps.querier,
        config.mint_contract_shd.code_hash.clone(),
        config.mint_contract_shd.address.clone(),
    )?;

    match res {
        mint::QueryAnswer::Mint { amount, .. } => {
            final_amount = amount;
        }
        _ => {
            return Err(StdError::GenericErr {
                msg: String::from("Unexpected query result"),
                backtrace: None,
            });
        }
    }

    if final_amount > amount {
        return Ok(QueryAnswer::ArbPegProfitability {
            is_profitable: true,
            mint_first: false,
            first_swap_result,
            profit: final_amount.checked_sub(amount)?,
        });
    }

    Ok(QueryAnswer::ArbPegProfitability {
        is_profitable: false,
        mint_first: false,
        first_swap_result: Uint128::zero(),
        profit: Uint128::zero(),
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

    debug_print!("{}", viewing_key);

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

    Ok(QueryAnswer::Balance { shd_bal, silk_bal })
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
    let config = Config::load(&deps.storage)?;
    let mut cycles = Cycles::load(&deps.storage)?.0;
    let mut swap_amounts = vec![amount];

    if index.u128() > cycles.len().try_into().unwrap() {
        return Err(StdError::GenericErr {
            msg: "Index passed is out of bounds".to_string(),
            backtrace: None,
        });
    }

    let mut current_offer: TokenAmount = TokenAmount {
        token: TokenType::CustomToken {
            contract_addr: config.shd_token_contract.address.clone(),
            token_code_hash: config.shd_token_contract.code_hash.clone(),
        },
        amount,
    };

    for arb_pair in cycles[index.u128() as usize].pair_addrs.clone() {
        let res = shadeswap::PairQuery::GetEstimatedPrice {
            offer: current_offer.clone(),
        }
        .query(
            &deps.querier,
            arb_pair.pair_contract.code_hash.clone(),
            arb_pair.pair_contract.address.clone(),
        )?;
        match res {
            shadeswap::QueryMsgResponse::EstimatedPrice { estimated_price } => {
                match current_offer.token {
                    TokenType::CustomToken {
                        token_code_hash, ..
                    } => {
                        if token_code_hash == arb_pair.token0_contract.code_hash {
                            current_offer = TokenAmount {
                                token: TokenType::CustomToken {
                                    contract_addr: arb_pair.token1_contract.address.clone(),
                                    token_code_hash: arb_pair.token1_contract.code_hash,
                                },
                                amount: estimated_price,
                            };
                        } else {
                            current_offer = TokenAmount {
                                token: TokenType::CustomToken {
                                    contract_addr: arb_pair.token0_contract.address.clone(),
                                    token_code_hash: arb_pair.token0_contract.code_hash,
                                },
                                amount: estimated_price,
                            };
                        }
                        swap_amounts.push(estimated_price.clone());
                    }
                    _ => {}
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

    if swap_amounts.len() > cycles[index.u128() as usize].pair_addrs.clone().len() {
        return Err(StdError::GenericErr {
            msg: String::from("More swap amounts than arb pairs"),
            backtrace: None,
        });
    }

    if current_offer.amount.u128() > amount.u128() {
        return Ok(QueryAnswer::IsCycleProfitable {
            is_profitable: true,
            direction: cycles[index.u128() as usize].clone(),
            swap_amounts,
            profit: current_offer.amount.checked_sub(amount)?,
        });
    }

    swap_amounts = vec![amount];
    current_offer = TokenAmount {
        token: TokenType::CustomToken {
            contract_addr: config.shd_token_contract.address,
            token_code_hash: config.shd_token_contract.code_hash,
        },
        amount,
    };

    for arb_pair in cycles[index.u128() as usize]
        .pair_addrs
        .clone()
        .iter()
        .rev()
    {
        let res = shadeswap::PairQuery::GetEstimatedPrice {
            offer: current_offer.clone(),
        }
        .query(
            &deps.querier,
            arb_pair.pair_contract.code_hash.clone(),
            arb_pair.pair_contract.address.clone(),
        )?;
        match res {
            shadeswap::QueryMsgResponse::EstimatedPrice { estimated_price } => {
                match current_offer.token {
                    TokenType::CustomToken {
                        token_code_hash, ..
                    } => {
                        if token_code_hash == arb_pair.token0_contract.code_hash {
                            current_offer = TokenAmount {
                                token: TokenType::CustomToken {
                                    contract_addr: arb_pair.token1_contract.address.clone(),
                                    token_code_hash: arb_pair.token1_contract.code_hash.clone(),
                                },
                                amount: estimated_price,
                            };
                        } else {
                            current_offer = TokenAmount {
                                token: TokenType::CustomToken {
                                    contract_addr: arb_pair.token0_contract.address.clone(),
                                    token_code_hash: arb_pair.token0_contract.code_hash.clone(),
                                },
                                amount: estimated_price,
                            };
                        }
                    }
                    _ => {}
                }
                swap_amounts.push(estimated_price.clone());
            }
            _ => {
                return Err(StdError::GenericErr {
                    msg: "Unexpected result".to_string(),
                    backtrace: None,
                });
            }
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
    if !(config.shd_token_contract.address == asset)
        && !(config.silk_token_contract.address == asset)
    {
        return Err(StdError::GenericErr {
            msg: String::from("Unrecognized asset"),
            backtrace: None,
        });
    }
    let res = get_balances(deps)?;
    let mut amount = Uint128::zero();
    match res {
        QueryAnswer::Balance { shd_bal, silk_bal } => {
            if config.shd_token_contract.address == asset {
                amount = shd_bal;
            } else {
                amount = silk_bal;
            }
        }
        _ => {}
    }
    Ok(adapter::QueryAnswer::Balance {
        amount: cosmwasm_std::Uint128(amount.u128()),
    })
}

pub fn adapter_claimable<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    _asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    Ok(adapter::QueryAnswer::Claimable {
        amount: cosmwasm_std::Uint128::zero(),
    })
}

pub fn adapter_unbondable<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    let config = Config::load(&deps.storage)?;
    if !(config.shd_token_contract.address == asset)
        && !(config.silk_token_contract.address == asset)
    {
        return Err(StdError::GenericErr {
            msg: String::from("Unrecognized asset"),
            backtrace: None,
        });
    }
    let res = get_balances(deps)?;
    let mut amount = Uint128::zero();
    match res {
        QueryAnswer::Balance { shd_bal, silk_bal } => {
            if config.shd_token_contract.address == asset {
                amount = shd_bal;
            } else {
                amount = silk_bal;
            }
        }
        _ => {}
    }
    Ok(adapter::QueryAnswer::Unbondable {
        amount: cosmwasm_std::Uint128(amount.u128()),
    })
}

pub fn adapter_unbonding<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    _asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    Ok(adapter::QueryAnswer::Unbonding {
        amount: cosmwasm_std::Uint128::zero(),
    })
}

pub fn adapter_reserves<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    let config = Config::load(&deps.storage)?;
    if !(config.shd_token_contract.address == asset)
        && !(config.silk_token_contract.address == asset)
    {
        return Err(StdError::GenericErr {
            msg: String::from("Unrecognized asset"),
            backtrace: None,
        });
    }
    let res = get_balances(deps)?;
    let mut amount = Uint128::zero();
    match res {
        QueryAnswer::Balance { shd_bal, silk_bal } => {
            if config.shd_token_contract.address == asset {
                amount = shd_bal;
            } else if config.silk_token_contract.address == asset {
                amount = silk_bal;
            }
        }
        _ => {}
    }
    Ok(adapter::QueryAnswer::Reserves {
        amount: cosmwasm_std::Uint128(amount.u128()),
    })
}
