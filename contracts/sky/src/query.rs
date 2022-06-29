use std::{
    convert::{TryFrom, TryInto},
    thread::current,
};

use cosmwasm_math_compat::{Uint128, Uint64};
use cosmwasm_std::{debug_print, Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage};
use fadroma::prelude::ContractLink; /*
use shadeswap_shared::{
self, msg, TokenAmount,
};*/
use secret_toolkit::utils::Query;
use shade_protocol::{
    contract_interfaces::{
        dao::adapter,
        dex::{
            dex::pool_take_amount,
            shadeswap::{self, TokenAmount, TokenType},
            sienna::{self, PairInfo, PairInfoResponse, PairQuery},
        },
        mint::mint::{self, QueryMsg},
        sky::sky::{ArbPair, Config, Cycles, QueryAnswer, SelfAddr, ViewingKeys},
        snip20,
    },
    utils::{asset::Contract, storage::plus::ItemStorage},
};

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: Config::load(&deps.storage)?,
    })
}

/*pub fn market_rate<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    let config: Config = Config::load(&deps.storage)?;

    //Query mint contract
    let mint_info: mint::QueryAnswer = QueryMsg::Mint {
        offer_asset: config.shd_token_contract.address.clone(),
        amount: Uint128::new(100000000), //1 SHD
    }
    .query(
        &deps.querier,
        config.mint_addr_silk.code_hash.clone(),
        config.mint_addr_silk.address.clone(),
    )?;
    let mut mint_price: Uint128 = Uint128::new(0); // SILK/SHD
    match mint_info {
        mint::QueryAnswer::Mint { asset: _, amount } => {
            mint_price = amount.checked_mul(Uint128::new(100))?; // times 100 to make it have 8 decimals
        }
        _ => {
            mint_price = Uint128::new(0);
        }
    };

    //TODO Query Pool Amount
    let pool_info: PairInfoResponse = PairQuery::PairInfo.query(
        &deps.querier,
        config.market_swap_addr.code_hash.clone(),
        config.market_swap_addr.address.clone(),
    )?;

    Ok(QueryAnswer::GetMarketRate {
        mint_rate: mint_price,
        pair: pool_info,
    })
}*/

/*pub fn trade_profitability<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    amount: Uint128,
) -> StdResult<QueryAnswer> {
    let config: Config = Config::load(&deps.storage)?;

    let market_query = market_rate(&deps)?;
    let mint_price: Uint128;
    let pool_info: PairInfoResponse;

    match market_query {
        QueryAnswer::GetMarketRate { mint_rate, pair } => {
            mint_price = mint_rate;
            pool_info = pair;
        }
        _ => {
            return Err(StdError::generic_err("failed."));
        }
    };

    let mut shd_amount: Uint128 = Uint128::new(1);
    let mut silk_amount: Uint128 = Uint128::new(1);
    let mut silk_8d: Uint128 = Uint128::new(1);

    match pool_info.pair_info.pair.token_0 {
        sienna::TokenType::CustomToken {
            contract_addr,
            token_code_hash: _,
        } => {
            if contract_addr.eq(&config.shd_token_contract.address) {
                shd_amount = pool_info.pair_info.amount_0;
                silk_amount = pool_info.pair_info.amount_1;
                silk_8d = silk_amount.checked_mul(Uint128::new(100))?;
            } else {
                shd_amount = pool_info.pair_info.amount_1;
                silk_amount = pool_info.pair_info.amount_0;
                silk_8d = silk_amount.checked_mul(Uint128::new(100))?;
            }
        }
        _ => {}
    }

    let div_silk_8d: Uint128 = silk_8d.checked_mul(Uint128::new(100000000))?;
    let dex_price: Uint128 = div_silk_8d.checked_div(shd_amount.clone())?;

    let mut first_swap_amount: Uint128 = Uint128::new(0);
    let mut second_swap_amount: Uint128 = Uint128::new(0);
    let mut mint_first: bool = false;

    if mint_price.gt(&dex_price) {
        mint_first = true;
        let mul_mint_price: Uint128 = mint_price.checked_mul(amount)?;
        first_swap_amount = mul_mint_price.checked_div(Uint128::new(100000000))?;
        let mut first_swap_less_fee = first_swap_amount.checked_div(Uint128::new(325))?;
        first_swap_less_fee = first_swap_amount.checked_sub(first_swap_less_fee)?;
        second_swap_amount = pool_take_amount(amount, silk_8d, shd_amount);
    } else {
        mint_first = false;
        let mut amount_less_fee: Uint128 = amount.checked_div(Uint128::new(325))?;
        amount_less_fee = amount.checked_sub(amount_less_fee)?;
        first_swap_amount = pool_take_amount(amount_less_fee, shd_amount, silk_8d);
        let mul_first_swap = first_swap_amount.checked_mul(Uint128::new(100000000))?;
        second_swap_amount = mul_first_swap.checked_div(mint_price)?;
    }

    let is_profitable = second_swap_amount.gt(&amount);

    Ok(QueryAnswer::ArbPegProfitability {
        is_profitable,
        mint_first,
        shd_amount,
        silk_amount,
        first_swap_amount,
        second_swap_amount,
    })
}*/

pub fn conversion_mint_profitability<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    amount: Uint128,
) -> StdResult<QueryAnswer> {
    let config: Config = Config::load(&deps.storage)?;
    let mut first_swap_result;

    let mut res = mint::QueryMsg::Mint {
        offer_asset: config.shd_token_contract.address.clone(),
        amount,
    }
    .query(
        &deps.querier,
        config.mint_contract_silk.code_hash.clone(),
        config.mint_contract_silk.address.clone(),
    )?;

    match res {
        mint::QueryAnswer::Mint { asset, amount } => {
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

    let mut res = mint::QueryMsg::Mint {
        offer_asset: config.silk_token_contract.address.clone(),
        amount: first_swap_result,
    }
    .query(
        &deps.querier,
        config.mint_contract_shd.code_hash.clone(),
        config.mint_contract_shd.address.clone(),
    )?;

    match res {
        mint::QueryAnswer::Mint { asset, amount } => {
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
        });
    }

    Ok(QueryAnswer::ArbPegProfitability {
        is_profitable: false,
        mint_first: false,
        first_swap_result: Uint128::zero(),
    })
}

pub fn get_balances<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    let viewing_key = ViewingKeys::load(&deps.storage)?.0;
    let self_addr = SelfAddr::load(&deps.storage)?.0;
    let config = Config::load(&deps.storage)?;
    let mut is_error = false;

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
        _ => is_error = true,
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
        _ => is_error = true,
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
    let mut new_pair_addrs: Vec<ArbPair>;
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
                        contract_addr,
                        token_code_hash,
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

    if swap_amounts
        .len()
        .gt(&cycles[index.u128() as usize].pair_addrs.clone().len())
    {
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
                        contract_addr,
                        token_code_hash,
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

    if swap_amounts
        .len()
        .gt(&cycles[index.u128() as usize].pair_addrs.clone().len())
    {
        return Err(StdError::GenericErr {
            msg: String::from("More swap amounts than arb pairs"),
            backtrace: None,
        });
    }

    if current_offer.amount.u128() > amount.u128() {
        cycles[index.u128() as usize].pair_addrs.reverse();
        return Ok(QueryAnswer::IsCycleProfitable {
            is_profitable: true,
            direction: cycles[index.u128() as usize].clone(),
            swap_amounts,
        });
    }

    Ok(QueryAnswer::IsCycleProfitable {
        is_profitable: false,
        direction: cycles[0].clone(),
        swap_amounts: vec![],
    })
}

pub fn any_cycles_profitable<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    amount: Uint128,
) -> StdResult<QueryAnswer> {
    let mut cycles = Cycles::load(&deps.storage)?.0;
    let mut return_is_profitable = vec![];
    let mut return_directions = vec![];
    let mut return_swap_amounts = vec![];

    for index in 0..cycles.len() {
        let res = cycle_profitability(deps, amount, Uint128::from(index as u128)).unwrap();
        match res {
            QueryAnswer::IsCycleProfitable {
                is_profitable,
                direction,
                swap_amounts,
            } => {
                if is_profitable {
                    return_is_profitable.push(is_profitable);
                    return_directions.push(direction);
                    return_swap_amounts.push(swap_amounts);
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

    //TODO Fix this
    Ok(QueryAnswer::IsCycleProfitable {
        is_profitable: false,
        direction: cycles[0].clone(),
        swap_amounts: vec![],
    })
}

pub fn adapter_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    Ok(adapter::QueryAnswer::Balance {
        amount: cosmwasm_std::Uint128::zero(),
    })
}
pub fn adapter_claimable<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    Ok(adapter::QueryAnswer::Balance {
        amount: cosmwasm_std::Uint128::zero(),
    })
}

pub fn adapter_unbondable<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    Ok(adapter::QueryAnswer::Balance {
        amount: cosmwasm_std::Uint128::zero(),
    })
}

pub fn adapter_unbonding<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    Ok(adapter::QueryAnswer::Balance {
        amount: cosmwasm_std::Uint128::zero(),
    })
}

pub fn adapter_reserves<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    asset: HumanAddr,
) -> StdResult<adapter::QueryAnswer> {
    Ok(adapter::QueryAnswer::Balance {
        amount: cosmwasm_std::Uint128::zero(),
    })
}
