use shade_protocol::{
    c_std::{
        from_binary,
        to_binary,
        Addr,
        Api,
        Binary,
        CosmosMsg,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Querier,
        QuerierWrapper,
        Response,
        StdError,
        StdResult,
        Storage,
        Uint128,
    },
    chrono::prelude::*,
    dao::adapter,
    mint::liability_mint::{Config, HandleAnswer},
    snip20::helpers::{
        self,
        burn_msg,
        fetch_snip20,
        mint_msg,
        send_msg,
        token_config,
        token_info,
        Snip20Asset,
        TokenConfig,
    },
    utils::{asset::Contract, callback::Query, generic_response::ResponseStatus},
};

use crate::storage::*;
use shade_oracles::{
    common::{querier::query_prices, OraclePrice},
    interfaces::router,
};

pub fn receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _sender: Addr,
    from: Addr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<Response> {
    let token = TOKEN.load(deps.storage)?;

    if info.sender == token.contract.address {
        //TODO Burn tokens
        let liab = LIABILITIES.load(deps.storage)?;

        let mut messages = vec![];
        let mut burn_amount = amount;

        // Handle excess tokens
        if liab < amount {
            burn_amount = liab;

            //TODO to treasury?
            messages.push(send_msg(
                from,
                liab - amount,
                None,
                None,
                None,
                &token.contract,
            )?);
        }

        messages.push(burn_msg(burn_amount, None, None, &token.contract)?);

        LIABILITIES.save(deps.storage, &(liab - burn_amount))?;

        Ok(Response::new()
            .add_messages(messages)
            .set_data(to_binary(&ExecuteAnswer::Mint {
                status: ResponseStatus::Success,
                amount,
            })?))
    } else {
        return Err(StdError::generic_err(format!(
            "Unrecognized token {}",
            info.sender
        )));
    }
}

pub fn try_update_config(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    config: Config,
) -> StdResult<Response> {
    let cur_config = CONFIG.load(deps.storage)?;

    // Admin-only
    if info.sender != cur_config.admin {
        return Err(StdError::generic_err("unauthorized"));
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    )
}

/* Queries the treasury for 'collateral_assets' balances
 * Queries oracle for 'collateral_assets' + 'debt_asset' USD prices
 * Returns the debt limit of 'debt_asset'
 *
 * NOTE
 *  This is an N time operation, treasury should
 *  implement batch queries
 *  maybe manager & adapter as well?
 */
pub fn debt_limit(
    deps: Deps,
    debt_asset: Snip20Asset,
    collateral_assets: Vec<Snip20Asset>,
    debt_ratio: Uint128,
    oracle: Contract,
    treasury: Contract,
) -> StdResult<Uint128> {
    let mut balances: Vec<Uint128> = vec![];
    let mut symbols: Vec<String> = vec![];
    for asset in COLLATERAL.load(deps.storage)? {
        balances.push(adapter::balance_query(
            deps.querier,
            &asset.contract.address,
            treasury,
        )?);
        symbols.push(asset.token_info.symbol);
    }

    symbols.push(debt_asset.token_info.symbol);

    let mut prices = query_prices(&oracle, &deps.querier, symbols.iter())?
        .iter()
        .map(|p| p.data.rate)
        .collect();

    let debt_price = prices.pop().data.rate;
    let asset_value = prices
        .iter()
        .zip(balances.iter())
        .map(|(b, p)| b * p.data.rate)
        .sum::<Uint128>();

    Ok(asset_value.multiply_ratio(debt_ratio, 10u128.pow(18)))
}

pub fn mint(deps: DepsMut, env: Env, info: MessageInfo, amount: Uint128) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    // Check if admin
    if !WHITELIST.load(deps.storage)?.contains(&info.sender) {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let limit = debt_limit(
        deps.as_ref(),
        TOKEN.load(deps.storage)?,
        COLLATERAL.load(deps.storage)?,
        config.debt_ratio,
        config.oracle,
        config.treasury,
    )?;
    // change to 'debt' nomenclature everywhere?
    let debt = LIABILITIES.load(deps.storage)?;

    if debt + amount > limit {
        return Err(StdError::generic_err(format!(
            "Additional debt would exceed limit, current: {} / {}",
            debt, limit,
        )));
    }

    LIABILITIES.save(deps.storage, &(debt + amount))?;

    Ok(Response::new()
        .add_message(mint_msg(
            info.sender,
            amount,
            None,
            None,
            &TOKEN.load(deps.storage)?.contract,
        )?)
        .set_data(to_binary(&ExecuteAnswer::Mint {
            status: ResponseStatus::Success,
            amount,
        })?))
}

pub fn add_whitelist(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: Addr,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    // Check if admin
    if info.sender != config.admin {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let mut ws = WHITELIST.load(deps.storage)?;
    ws.push(address);
    WHITELIST.save(deps.storage, &ws)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::AddWhitelist {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn rm_whitelist(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    address: Addr,
) -> StdResult<Response> {
    let config = CONFIG.load(deps.storage)?;

    // Check if admin
    if info.sender != config.admin {
        return Err(StdError::generic_err("Unauthorized"));
    }

    let mut ws = WHITELIST.load(deps.storage)?;

    if let Some(i) = ws.iter().position(|a| *a == address.clone()) {
        ws.remove(i);
    } else {
        return Err(StdError::generic_err("Not on whitelist"));
    }

    WHITELIST.save(deps.storage, &ws)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::RemoveWhitelist {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn add_collateral(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: Contract,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;

    // Check if admin
    if info.sender != config.admin {
        return Err(StdError::generic_err("Unauthorized"));
    }

    //TODO verify snip20 with msg
    let mut collateral = COLLATERAL.load(deps.storage)?;
    collateral.push(fetch_snip20(&asset, &deps.querier)?);
    COLLATERAL.save(deps.storage, &collateral)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::AddCollateral {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn rm_collateral(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset: Contract,
) -> StdResult<Response> {
    let mut config = CONFIG.load(deps.storage)?;

    // Check if admin
    if info.sender != config.admin {
        return Err(StdError::generic_err("Unauthorized"));
    }

    //TODO verify snip20 with msg
    let mut collateral = COLLATERAL.load(deps.storage)?;
    if let Some(pos) = collateral.iter().position(|a| a.contract == asset) {
        collateral.swap_remove(pos);
    } else {
        return Err(StdError::generic_err("Not valid collateral"));
    }

    COLLATERAL.save(deps.storage, &collateral)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::RemoveCollateral {
            status: ResponseStatus::Success,
        })?),
    )
}
