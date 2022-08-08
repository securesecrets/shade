use shade_oracles::{common::OraclePrice, interfaces::router};
use shade_protocol::{
    c_std::{Deps, StdError, StdResult, Uint128},
    contract_interfaces::{
        peg_stability::{CalculateRes, Config, QueryAnswer, ViewingKey},
        sky::cycles::Offer,
        snip20,
    },
    cosmwasm_schema::cw_serde,
    utils::{
        callback::Query,
        storage::plus::{GenericItemStorage, ItemStorage},
    },
};

pub fn get_config(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: Config::load(deps.storage)?,
    })
}

pub fn get_balance(deps: Deps) -> StdResult<QueryAnswer> {
    let viewing_key = ViewingKey::load(deps.storage)?;
    let config = Config::load(deps.storage)?;

    let mut res = snip20::QueryMsg::Balance {
        address: config.self_addr.clone().to_string(),
        key: viewing_key,
    }
    .query(&deps.querier, &config.snip20)?;

    match res {
        snip20::QueryAnswer::Balance { amount } => Ok(QueryAnswer::Balance { snip20_bal: amount }),
        _ => Ok(QueryAnswer::Balance {
            snip20_bal: Uint128::zero(),
        }),
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
    let prices = vec![Uint128::zero(), Uint128::zero()];
    let mut max_swap_amount = Uint128::zero();
    let mut index = 0usize;
    for (i, pair) in config.pairs.iter().enumerate() {
        pair.clone().pool_amounts(deps)?;
        let temp = calculate_swap_amount()?;
        if temp > max_swap_amount {
            max_swap_amount = temp;
            index = i;
        }
    }
    let initial_value = max_swap_amount * prices[0];
    let offer = Offer {
        asset: config.snip20,
        amount: max_swap_amount,
    };
    let swap_res = config.pairs[index].clone().simulate_swap(deps, offer)?;
    let after_swap = swap_res * prices[1];
    if after_swap > initial_value {
        return Ok(CalculateRes {
            profit: after_swap - initial_value,
            payback: (after_swap - initial_value) * config.payback,
            swap_amount: max_swap_amount,
            min_expected: swap_res,
        });
    }
    Ok(CalculateRes {
        profit: Uint128::zero(),
        payback: Uint128::zero(),
        swap_amount: Uint128::zero(),
        min_expected: Uint128::zero(),
    })
}

fn calculate_swap_amount() -> StdResult<Uint128> {
    Ok(Uint128::zero())
}
