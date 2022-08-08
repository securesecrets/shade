//use shade_oracles::{common::OraclePrice, router::QueryMsg};
use shade_protocol::{
    c_std::{Deps, StdError, StdResult, Uint128},
    contract_interfaces::{
        peg_stability::{CalculateRes, Config, QueryAnswer, ViewingKey},
        snip20,
    },
    cosmwasm_schema::cw_serde,
    utils::{
        storage::plus::{GenericItemStorage, ItemStorage},
        Query,
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
    Ok(QueryAnswer::Profitable {
        profit: Uint128::zero(),
    })
}

pub fn calculate_profit(deps: Deps) -> StdResult<CalculateRes> {
    let config = Config::load(deps.storage)?;
    if config.pairs.len() < 1 {
        return Err(StdError::generic_err("Must have pairs saved"));
    }
    /*let res: Vec<OraclePrice> = QueryMsg::GetPrices {
        keys: config.symbols,
    }
    .query(&deps.querier, &config.oracle)?;
    let prices = vec![
        Uint128::new(res[0].data.rate.u128()),
        Uint128::new(res[1].data.rate.u128()),
    ];*/
    let prices = vec![Uint128::zero(), Uint128::zero()];
    let max_swap_amount = Uint128::zero();
    let index = 0usize;
    for (i, pair) in config.pairs.iter().enumerate() {
        pair.pool_info(deps)?;
        let temp = calculate_swap_amount()?;
        if temp > max_swap_amount {
            max_swap_amount = temp;
            index = i;
        }
    }
    Ok(CalculateRes {})
}

fn calculate_swap_amount() -> StdResult<Uint128> {
    Ok(Uint128::zero())
}
