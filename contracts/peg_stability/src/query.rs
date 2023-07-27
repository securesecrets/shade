
use shade_protocol::{
    c_std::{Deps, Isqrt, StdError, StdResult, Uint128, Uint256},
    contract_interfaces::{
        peg_stability::{CalculateRes, Config, QueryAnswer, ViewingKey},
        sky::cycles::Offer,
        snip20,
    },
    snip20::helpers::balance_query,
    utils::{
        callback::Query,
        storage::plus::{GenericItemStorage, ItemStorage},
    },
};
use std::convert::TryFrom;

pub fn get_config(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: Config::load(deps.storage)?,
    })
}

pub fn get_balance(deps: Deps) -> StdResult<QueryAnswer> {
    let viewing_key = ViewingKey::load(deps.storage)?;
    let config = Config::load(deps.storage)?;

    let res = snip20::QueryMsg::Balance {
        address: config.self_addr.clone().to_string(),
        key: viewing_key,
    }
    .query(&deps.querier, &config.snip20)?;

    match res {
        snip20::QueryAnswer::Balance { amount } => Ok(QueryAnswer::Balance { snip20_bal: amount }),
        _ => Err(StdError::generic_err("snip20 bal query failed")),
    }
}

pub fn get_pairs(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::GetPairs {
        pairs: Config::load(deps.storage)?.pairs,
    })
}

pub fn profitable(deps: Deps) -> StdResult<QueryAnswer> {
    let res: CalculateRes = calculate_profit(deps)?;
    Ok(QueryAnswer::Profitable {
        profit: res.profit,
        payback: res.payback,
    })
}

pub fn calculate_profit(deps: Deps) -> StdResult<CalculateRes> {
    let config = Config::load(deps.storage)?;
    if config.pairs.len() < 1 {
        return Err(StdError::generic_err("Must have pairs saved"));
    }
    /*let res: Vec<OraclePrice> = router::QueryMsg::GetPrices {
        keys: config.symbols,
    }
    .query(&deps.querier, &config.oracle)?
    .prices;
    let prices = vec![
        Uint128::new(res[0].data.rate.u128()),
        Uint128::new(res[1].data.rate.u128()),
    ];*/
    let prices = vec![Uint128::zero(); 2];
    let mut max_swap_amount = Uint128::zero();
    let mut index = 0usize;
    let snip20_dec;
    let other_dec;
    if config.pairs[0].token0 == config.snip20 {
        snip20_dec = config.pairs[0].token0_decimals.u128() as u32;
        other_dec = config.pairs[0].token1_decimals.u128() as u32;
    } else {
        snip20_dec = config.pairs[0].token1_decimals.u128() as u32;
        other_dec = config.pairs[0].token0_decimals.u128() as u32;
    }
    for (i, pair) in config.pairs.iter().enumerate() {
        let (t0_amount, t1_amount) = pair.clone().pool_amounts(deps)?;
        let mut temp;
        if config.snip20 == pair.token0 {
            temp = calculate_swap_amount(
                t0_amount.checked_mul(Uint128::new(10).pow(18 - snip20_dec.clone()))?,
                t1_amount.checked_mul(Uint128::new(10).pow(18 - other_dec.clone()))?,
                prices[0],
                prices[1],
            );
        } else {
            temp = calculate_swap_amount(
                t1_amount.checked_mul(Uint128::new(10).pow(18 - snip20_dec.clone()))?,
                t0_amount.checked_mul(Uint128::new(10).pow(18 - other_dec.clone()))?,
                prices[0],
                prices[1],
            );
        }
        temp = temp / Uint128::new(10).pow(18 - snip20_dec);
        if temp > max_swap_amount {
            max_swap_amount = temp;
            index = i;
        }
    }
    let balance = balance_query(
        &deps.querier,
        config.self_addr.clone(),
        ViewingKey::load(deps.storage)?,
        &config.snip20,
    )?;
    if max_swap_amount > balance {
        max_swap_amount = balance;
    }
    let initial_value = Uint256::from(max_swap_amount)
        / Uint256::from(Uint128::new(10).pow(snip20_dec))
        * Uint256::from(prices[0]);
    let offer = Offer {
        asset: config.snip20.clone(),
        amount: max_swap_amount.clone(),
    };
    let swap_res = config.pairs[index]
        .clone()
        .simulate_swap(deps, offer.clone())?;
    let after_swap = Uint256::from(swap_res) / Uint256::from(Uint128::new(10).pow(other_dec))
        * Uint256::from(prices[1]);
    if after_swap > initial_value {
        let profit = Uint128::try_from(after_swap - initial_value)?;
        let payback = profit / prices[1] * config.payback;
        return Ok(CalculateRes {
            profit,
            payback,
            index,
            config,
            offer,
            min_expected: swap_res,
        });
    }
    Ok(CalculateRes {
        profit: Uint128::zero(),
        payback: Uint128::zero(),
        index: 0usize,
        config,
        offer,
        min_expected: Uint128::zero(),
    })
}

fn calculate_swap_amount(
    poolsell: Uint128,
    poolbuy: Uint128,
    pricesell: Uint128,
    pricebuy: Uint128,
) -> Uint128 {
    let nom = Uint256::from(pricebuy.isqrt())
        * Uint256::from(poolbuy.isqrt())
        * Uint256::from(poolsell.isqrt());
    let denominator = Uint256::from(pricesell.isqrt());
    let right = nom / denominator;
    let res = right.checked_sub(Uint256::from(poolsell));
    match res {
        Ok(amount) => match Uint128::try_from(amount) {
            Ok(amount) => amount,
            Err(_error) => Uint128::MAX,
        },
        Err(_error) => Uint128::zero(),
    }
}

#[cfg(test)]
mod test {
    use crate::query::calculate_swap_amount;
    use shade_protocol::{
        c_std::{Uint128},
    };

    #[test]
    fn test_swapamount1() {
        assert_eq!(
            calculate_swap_amount(
                Uint128::new(10_000_000_000_000_000_000_000),
                Uint128::new(100_000_000_000_000_000_000_000),
                Uint128::new(10_000_000_000_000_000_000),
                Uint128::new(1_000_000_000_000_000_000),
            ) / Uint128::new(10).pow(12),
            Uint128::zero()
        )
    }

    #[test]
    fn test_swapamount2() {
        assert_eq!(
            calculate_swap_amount(
                Uint128::new(10_000_000_000_000_000_000_000),
                Uint128::new(100_000_000_000_000_000_000_000),
                Uint128::new(10_000_000_000_000_000_000),
                Uint128::new(1_100_000_000_000_000_000),
            ) / Uint128::new(10).pow(13),
            Uint128::new(48_808_848)
        )
    }
}
