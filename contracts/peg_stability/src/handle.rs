use crate::query::calculate_profit;
use shade_protocol::{
    admin::validate_admin,
    c_std::{
        to_binary,
        Decimal,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
        Uint128,
    },
    contract_interfaces::{
        peg_stability::{CalculateRes, Config, ExecuteAnswer, ViewingKey},
        sky::cycles::ArbPair,
    },
    snip20::helpers::{fetch_snip20, send_msg, set_viewing_key_msg, TokenInfo},
    utils::{
        asset::Contract,
        storage::plus::{GenericItemStorage, ItemStorage},
    },
};

pub fn try_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    shd_admin: Option<Contract>,
    snip20: Option<Contract>,
    treasury: Option<Contract>,
    oracle: Option<Contract>,
    payback: Option<Decimal>,
    dump_contract: Option<Contract>,
) -> StdResult<Response> {
    //Admin-only
    let mut config = Config::load(deps.storage)?;
    validate_admin(
        &deps.querier,
        env.contract.address.to_string(),
        info.sender.to_string(),
        &config.shd_admin,
    )?;
    let mut messages = vec![];
    if let Some(shd_admin) = shd_admin {
        config.shd_admin = shd_admin;
    }
    if let Some(snip20) = snip20 {
        if !(config.pairs.len() == 0) {
            return Err(StdError::generic_err(
                "You must remove all pairs before chaning the snip20 asset",
            ));
        }
        config.snip20 = snip20;
        let viewing_key = ViewingKey::load(deps.storage)?;
        messages.push(set_viewing_key_msg(viewing_key, None, &config.snip20)?)
    }
    if let Some(treasury) = treasury {
        config.treasury = treasury;
    }
    if let Some(oracle) = oracle {
        config.oracle = oracle;
    }
    if let Some(payback) = payback {
        config.payback = payback;
    }
    if let Some(dump_contract) = dump_contract {
        config.dump_contract = dump_contract;
    }
    Ok(Response::new()
        .add_messages(messages)
        .set_data(to_binary(&ExecuteAnswer::UpdateConfig {
            config,
            status: true,
        })?))
}

pub fn try_set_pairs(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pairs: Vec<ArbPair>,
) -> StdResult<Response> {
    //Admin-only
    let mut config = Config::load(deps.storage)?;
    validate_admin(
        &deps.querier,
        env.contract.address.to_string(),
        info.sender.to_string(),
        &config.shd_admin,
    )?;
    if pairs.len() < 1 {
        return Err(StdError::generic_err("Must pass at least one pair"));
    }
    let res0: TokenInfo = fetch_snip20(&pairs[0].token0, &deps.querier)?.token_info;
    let res1: TokenInfo = fetch_snip20(&pairs[0].token1, &deps.querier)?.token_info;
    let other_asset;
    if config.snip20 == pairs[0].token0 {
        config.symbols = vec![res0.symbol, res1.symbol];
        other_asset = pairs[0].token1.clone();
    } else {
        config.symbols = vec![res1.symbol, res0.symbol];
        other_asset = pairs[0].token0.clone();
    }
    for pair in pairs.clone() {
        pair.validate_pair()?;
        if !((pair.token0 == config.snip20 && pair.token1 == other_asset)
            || (pair.token0 == other_asset && pair.token1 == config.snip20))
        {
            return Err(StdError::generic_err(
                "pairs must have the same assets as the rest of the pairs",
            ));
        }
    }
    config.pairs = pairs;
    config.save(deps.storage)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::SetPairs {
            pairs: config.pairs,
            status: true,
        })?),
    )
}

pub fn try_append_pairs(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    pairs: Vec<ArbPair>,
) -> StdResult<Response> {
    let mut config = Config::load(deps.storage)?;
    if config.pairs.len() == 0 {
        return Ok(try_set_pairs(deps, env, info, pairs)?);
    } else if pairs.len() < 1 {
        return Err(StdError::generic_err("Must pass at least 1 pair"));
    }
    //Admin-only
    validate_admin(
        &deps.querier,
        env.contract.address.to_string(),
        info.sender.to_string(),
        &config.shd_admin,
    )?;
    let other_asset;
    if config.snip20 == config.pairs[0].token0 {
        other_asset = config.pairs[0].token1.clone();
    } else {
        other_asset = config.pairs[0].token0.clone();
    }
    for pair in pairs.clone() {
        pair.validate_pair()?;
        if !((pair.token0 == config.snip20 && pair.token1 == other_asset)
            || (pair.token0 == other_asset && pair.token1 == config.snip20))
        {
            return Err(StdError::generic_err(
                "pairs must have the same assets as the rest of the pairs",
            ));
        }
    }
    config.pairs.append(&mut pairs.clone());
    config.save(deps.storage)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::AppendPairs {
            pairs: config.pairs,
            status: true,
        })?),
    )
}

pub fn try_remove_pair(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    index: Uint128,
) -> StdResult<Response> {
    //Admin-only
    let mut config = Config::load(deps.storage)?;
    validate_admin(
        &deps.querier,
        env.contract.address.to_string(),
        info.sender.to_string(),
        &config.shd_admin,
    )?;
    let i = index.u128() as usize;
    if config.pairs.len() == 0 {
        return Err(StdError::generic_err("No pairs to remove"));
    }
    if i > config.pairs.len() - 1 {
        return Err(StdError::generic_err("Index out of bounds"));
    }
    config.pairs.remove(i);
    config.save(deps.storage)?;
    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::RemovePair {
            pairs: config.pairs,
            status: true,
        })?),
    )
}

pub fn try_swap(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let res: CalculateRes = calculate_profit(deps.as_ref())?;
    let other_asset;
    if res.config.snip20 == res.config.pairs[0].token0 {
        other_asset = res.config.pairs[0].token1.clone();
    } else {
        other_asset = res.config.pairs[0].token0.clone();
    }
    let messages = vec![
        res.config.pairs[res.index].to_cosmos_msg(res.offer, res.min_expected)?,
        send_msg(
            res.config.dump_contract.address.to_string(),
            res.min_expected - res.payback,
            None,
            None,
            None,
            &other_asset,
        )?,
        send_msg(
            info.sender.to_string(),
            res.payback,
            None,
            None,
            None,
            &other_asset,
        )?,
    ];
    Ok(Response::new()
        .add_messages(messages)
        .set_data(to_binary(&ExecuteAnswer::Swap {
            profit: res.profit,
            payback: res.payback,
            status: true,
        })?))
}
