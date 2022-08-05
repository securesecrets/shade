use chrono::prelude::*;
use shade_protocol::c_std::{Deps, MessageInfo, QuerierWrapper, Uint128};
use shade_protocol::c_std::{
    from_binary,
    to_binary,
    Api,
    Binary,
    CosmosMsg,
    Env,
    DepsMut,
    Response,
    Addr,
    Querier,
    StdError,
    StdResult,
    Storage,
};
use shade_protocol::{
    snip20::helpers::{self, burn_msg, mint_msg, send_msg, TokenConfig},
};
use shade_protocol::{
    contract_interfaces::{
        mint::liability_mint::{Config, HandleAnswer},
        snip20::helpers::Snip20Asset,
    },
    utils::{asset::Contract, generic_response::ResponseStatus},
};
use std::{cmp::Ordering, convert::TryFrom};
use std::borrow::BorrowMut;
use std::fmt::format;
use shade_protocol::snip20::helpers::{token_config, token_info};
use shade_protocol::utils::Query;

use crate::storage::*;

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

        messages.push(burn_msg(
            burn_amount,
            None,
            None,
            &token.contract
        )?);

        LIABILITIES.save(deps.storage, &(liab - burn_amount))?;

        Ok(Response::new()
           .add_messages(messages)
           .set_data(to_binary(&HandleAnswer::Mint {
                status: ResponseStatus::Success,
                amount,
            })?))

    }
    else {
        return Err(StdError::generic_err(format!("Unrecognized token {}", info.sender)));
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

    Ok(Response::new().set_data(to_binary(&HandleAnswer::UpdateConfig {
        status: ResponseStatus::Success,
    })?))
}
pub fn mint(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    amount: Uint128,
) -> StdResult<Response> {

    let config = CONFIG.load(deps.storage)?;

    // Check if admin
    if !WHITELIST.load(deps.storage)?.contains(&info.sender) {
        return Err(StdError::generic_err("Unauthorized"));
    }

    // TODO check limit
    let liab = LIABILITIES.load(deps.storage)?;
    LIABILITIES.save(deps.storage, &(liab + amount))?;

    Ok(Response::new()
       .add_message(
           mint_msg(
               info.sender,
               amount,
               None,
               None,
               &TOKEN.load(deps.storage)?.contract,
           )?
       )
       .set_data(to_binary(&HandleAnswer::Mint {
        status: ResponseStatus::Success,
        amount,
    })?))
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
    }
    else {
        return Err(StdError::generic_err("Not on whitelist"));
    }

    WHITELIST.save(deps.storage, &ws)?;

    Ok(Response::new()
       .set_data(to_binary(&HandleAnswer::RemoveWhitelist {
        status: ResponseStatus::Success,
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

    Ok(Response::new()
       .set_data(to_binary(&HandleAnswer::AddWhitelist {
        status: ResponseStatus::Success,
    })?))
}
