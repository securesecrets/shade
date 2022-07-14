use chrono::prelude::*;
use shade_protocol::c_std::Uint128;
use shade_protocol::c_std::{
    debug_print,
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
use shade_protocol::secret_toolkit::{
    snip20::{burn_msg, mint_msg, register_receive_msg, send_msg, token_info_query,
             token_config_query, TokenConfig},
    utils::Query,
};
use shade_protocol::{
    contract_interfaces::{
        mint::mint::{Config, HandleAnswer, Limit, MintMsgHook, SupportedAsset},
        oracles::{band::ReferenceData, oracle::QueryMsg::Price},
        snip20::helpers::Snip20Asset,
    },
    utils::{asset::Contract, generic_response::ResponseStatus},
};
use std::{cmp::Ordering, convert::TryFrom};

use crate::state::{
    asset_list_w,
    asset_peg_r,
    assets_r,
    assets_w,
    config_r,
    config_w,
    limit_r,
    limit_refresh_r,
    limit_refresh_w,
    limit_w,
    minted_r,
    minted_w,
    native_asset_r,
    total_burned_w,
};

pub fn try_burn<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    _sender: Addr,
    from: Addr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<Response> {
    let config = config_r(&deps.storage).load()?;
    // Check if contract enabled
    if !config.activated {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let mint_asset = native_asset_r(&deps.storage).load()?;

    // Prevent sender to be native asset
    if mint_asset.contract.address == info.sender {
        return Err(StdError::generic_err(
            "Sender cannot be the same as the native asset.",
        ));
    }

    // Check that sender is a supported snip20 asset
    let burn_asset =
        match assets_r(&deps.storage).may_load(info.sender.to_string().as_bytes())? {
            Some(supported_asset) => {
                debug_print!(
                    "Found Burn Asset: {} {}",
                    &supported_asset.asset.token_info.symbol,
                    info.sender.to_string()
                );
                supported_asset
            }
            None => {
                return Err(StdError::NotFound {
                    kind: info.sender.to_string(),
                    backtrace: None,
                });
            }
        };

    let mut input_amount = amount;
    let mut messages = vec![];

    if burn_asset.fee > Uint128::zero() {
        let fee_amount = calculate_portion(input_amount, burn_asset.fee);
        // Reduce input by fee
        input_amount = input_amount.checked_sub(fee_amount)?;

        // Fee to treasury
        messages.push(send_msg(
            config.treasury.clone(),
            fee_amount.into(),
            None,
            None,
            None,
            1,
            burn_asset.asset.contract.code_hash.clone(),
            burn_asset.asset.contract.address.clone(),
        )?);
    }

    // This will calculate the total mint value
    let amount_to_mint: Uint128 = mint_amount(deps, input_amount, &burn_asset, &mint_asset)?;

    if let Some(limit) = config.limit {
        // Limit Refresh Check
        try_limit_refresh(deps, env, limit)?;

        // Check & adjust limit if a limited asset
        if !burn_asset.unlimited {
            let minted = minted_r(&deps.storage).load()?;
            if (amount_to_mint + minted) > limit_r(&deps.storage).load()? {
                return Err(StdError::generic_err("Limit Exceeded"));
            }

            minted_w(&mut deps.storage).save(&(amount_to_mint + minted))?;
        }
    }

    let mut burn_amount = input_amount;

    // Ignore capture if the set capture is 0
    if burn_asset.capture > Uint128::zero() {
        let capture_amount = calculate_portion(amount, burn_asset.capture);

        // Capture to treasury
        messages.push(send_msg(
            config.treasury.into(),
            capture_amount.into(),
            None,
            None,
            None,
            1,
            burn_asset.asset.contract.code_hash.clone(),
            burn_asset.asset.contract.address.clone(),
        )?);

        burn_amount = input_amount.checked_sub(capture_amount)?;
    }

    if burn_amount > Uint128::zero() {
        // Try to burn
        if let Some(token_config) = &burn_asset.asset.token_config {
            if token_config.burn_enabled {
                messages.push(burn_msg(
                    burn_amount.into(),
                    None,
                    None,
                    256,
                    burn_asset.asset.contract.code_hash.clone(),
                    burn_asset.asset.contract.address.clone(),
                )?);
            } else if let Some(recipient) = config.secondary_burn {
                messages.push(send_msg(
                    recipient,
                    burn_amount.into(),
                    None,
                    None,
                    None,
                    1,
                    burn_asset.asset.contract.code_hash.clone(),
                    burn_asset.asset.contract.address.clone(),
                )?);
            }
        } else if let Some(recipient) = config.secondary_burn {
            messages.push(send_msg(
                recipient,
                burn_amount.into(),
                None,
                None,
                None,
                1,
                burn_asset.asset.contract.code_hash.clone(),
                burn_asset.asset.contract.address.clone(),
            )?);
        }
    }

    total_burned_w(&mut deps.storage).update(
        burn_asset.asset.contract.address.to_string().as_bytes(),
        |burned| match burned {
            Some(burned) => Ok(burned + burn_amount),
            None => Ok(burn_amount),
        },
    )?;

    if let Some(message) = msg {
        let msg: MintMsgHook = from_binary(&message)?;

        // Check Slippage
        if amount_to_mint < msg.minimum_expected_amount {
            return Err(StdError::generic_err(
                "Mint amount is less than the minimum expected.",
            ));
        }
    };

    messages.push(mint_msg(
        from,
        amount_to_mint.into(),
        None,
        None,
        256,
        mint_asset.contract.code_hash.clone(),
        mint_asset.contract.address,
    )?);

    Ok(Response {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Mint {
            status: ResponseStatus::Success,
            amount: amount_to_mint,
        })?),
    })
}

pub fn try_limit_refresh<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    limit: Limit,
) -> StdResult<Uint128> {
    match DateTime::parse_from_rfc3339(&limit_refresh_r(&deps.storage).load()?) {
        Ok(parsed) => {
            let naive = NaiveDateTime::from_timestamp(env.block.time as i64, 0);
            let now: DateTime<Utc> = DateTime::from_utc(naive, Utc);
            let last_refresh: DateTime<Utc> = parsed.with_timezone(&Utc);

            let mut fresh_amount = Uint128::zero();

            let native_asset = native_asset_r(&deps.storage).load()?;

            let token_info = token_info_query(
                &deps.querier,
                1,
                native_asset.contract.code_hash.clone(),
                native_asset.contract.address.clone(),
            )?;

            let supply = match token_info.total_supply {
                Some(s) => s.into(),
                None => return Err(StdError::generic_err("Could not get native token supply")),
            };

            // get amount to add, 0 if not in need of refresh
            match limit {
                Limit::Daily {
                    supply_portion,
                    days,
                } => {
                    // Slight error in annual limit if (days / 365) is not a whole number
                    if now.num_days_from_ce() as u128 - days.u128()
                        >= last_refresh.num_days_from_ce() as u128
                    {
                        fresh_amount = calculate_portion(supply, supply_portion);
                    }
                }
                Limit::Monthly {
                    supply_portion,
                    months,
                } => {
                    if now.year() > last_refresh.year() || now.month() > last_refresh.month() {
                        /* If its a new year or new month, add (year_diff * 12) to the later (now) month
                         * 12-2021 <-> 1-2022 becomes a comparison between 12 <-> (1 + 12)
                         * resulting in a difference of 1 month
                         */
                        let year_diff = now.year() - last_refresh.year();

                        if (now.month() + (year_diff * 12) as u32) - last_refresh.month()
                            >= months.u128() as u32
                        {
                            fresh_amount = calculate_portion(supply, supply_portion);
                        }
                    }
                }
            }

            if fresh_amount > Uint128::zero() {
                let minted = minted_r(&deps.storage).load()?;

                limit_w(&mut deps.storage).update(|state| {
                    // Stack with previous unminted limit
                    Ok(state.checked_sub(minted)? + fresh_amount)
                })?;
                limit_refresh_w(&mut deps.storage).save(&now.to_rfc3339())?;
                minted_w(&mut deps.storage).save(&Uint128::zero())?;
            }

            Ok(fresh_amount)
        }
        Err(e) => return Err(StdError::generic_err("Failed to parse previous datetime")),
    }
}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    config: Config,
) -> StdResult<Response> {
    let cur_config = config_r(&deps.storage).load()?;

    // Admin-only
    if info.sender != cur_config.admin {
        return Err(StdError::unauthorized());
    }

    config_w(&mut deps.storage).save(&config)?;

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_register_asset<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: &Env,
    contract: &Contract,
    capture: Option<Uint128>,
    fee: Option<Uint128>,
    unlimited: Option<bool>,
) -> StdResult<Response> {
    let config = config_r(&deps.storage).load()?;
    // Check if admin
    if info.sender != config.admin {
        return Err(StdError::Unauthorized { backtrace: None });
    }
    // Check if contract enabled
    if !config.activated {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let contract_str = contract.address.to_string();

    // Add the new asset
    let asset_info = token_info_query(
        &deps.querier,
        1,
        contract.code_hash.clone(),
        contract.address.clone(),
    )?;

    let asset_config: Option<TokenConfig> =
        match token_config_query(&deps.querier, 256, contract.code_hash.clone(), contract.address.clone()) {
            Ok(c) => Option::from(c),
            Err(_) => None,
        };

    debug_print!("Registering {}", asset_info.symbol);
    assets_w(&mut deps.storage).save(contract_str.as_bytes(), &SupportedAsset {
        asset: Snip20Asset {
            contract: contract.clone(),
            token_info: asset_info,
            token_config: asset_config,
        },
        // If capture is not set then default to 0
        capture: match capture {
            None => Uint128::zero(),
            Some(value) => value,
        },
        fee: match fee {
            None => Uint128::zero(),
            Some(value) => value,
        },
        unlimited: match unlimited {
            None => false,
            Some(u) => u,
        },
    })?;

    total_burned_w(&mut deps.storage).save(contract_str.as_bytes(), &Uint128::zero())?;

    // Add the asset to list
    asset_list_w(&mut deps.storage).update(|mut state| {
        state.push(contract.clone());
        Ok(state)
    })?;

    // Register contract in asset
    let messages = vec![register_receive(env, contract)?];

    Ok(Response {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RegisterAsset {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_remove_asset<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    _env: &Env,
    address: Addr,
) -> StdResult<Response> {
    let address_str = address.to_string();

    // Remove asset from the array
    asset_list_w(&mut deps.storage).update(|mut state| {
        state.retain(|value| value.address != address);
        Ok(state)
    })?;

    // Remove supported asset
    assets_w(&mut deps.storage).remove(address_str.as_bytes());

    // We wont remove the total burned since we want to keep track of all the burned assets

    Ok(Response {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RemoveAsset {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn register_receive(env: &Env, contract: &Contract) -> StdResult<CosmosMsg> {
    register_receive_msg(
        env.contract_code_hash.clone(),
        None,
        256,
        contract.code_hash.clone(),
        contract.address.clone(),
    )
}

pub fn mint_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    burn_amount: Uint128,
    burn_asset: &SupportedAsset,
    mint_asset: &Snip20Asset,
) -> StdResult<Uint128> {
    debug_print!(
        "Burning {} {} for {}",
        burn_amount,
        burn_asset.asset.token_info.symbol,
        mint_asset.token_info.symbol
    );

    let burn_price = oracle(deps, burn_asset.asset.token_info.symbol.clone())?;
    debug_print!("Burn Price: {}", burn_price);

    let mint_price = oracle(deps, asset_peg_r(&deps.storage).load()?)?;
    debug_print!("Mint Price: {}", mint_price);

    Ok(calculate_mint(
        burn_price,
        burn_amount,
        burn_asset.asset.token_info.decimals,
        mint_price,
        mint_asset.token_info.decimals,
    ))
}

pub fn calculate_mint(
    burn_price: Uint128,
    burn_amount: Uint128,
    burn_decimals: u8,
    mint_price: Uint128,
    mint_decimals: u8,
) -> Uint128 {
    // Math must only be made in integers
    // in_decimals  = x
    // target_decimals = y
    // in_price     = p1 * 10^18
    // target_price = p2 * 10^18
    // in_amount    = a1 * 10^x
    // return       = a2 * 10^y

    // (a1 * 10^x) * (p1 * 10^18) = (a2 * 10^y) * (p2 * 10^18)

    //                (p1 * 10^18)
    // (a1 * 10^x) * --------------  = (a2 * 10^y)
    //                (p2 * 10^18)

    let burn_value = burn_amount.multiply_ratio(burn_price, mint_price);

    // burn_value * 10^(y - x) = (a2 * 10^y)
    let difference: i32 = mint_decimals as i32 - burn_decimals as i32;

    // To avoid a mess of different types doing math
    match difference.cmp(&0) {
        Ordering::Greater => {
            Uint128::new(burn_value.u128() * 10u128.pow(u32::try_from(difference).unwrap()))
        }
        Ordering::Less => {
            burn_value.multiply_ratio(1u128, 10u128.pow(u32::try_from(difference.abs()).unwrap()))
        }
        Ordering::Equal => burn_value,
    }
}

/*
pub fn calculate_fee_curve(
    // "Centered"
    base_fee: Uint128,
    // How far off from where we want (abs(desired_price - cur_price))
    price_skew: Uint128,
    // skew we should never reach (where fee maxes out)
    asymptote: Uint128,
) -> Uint128 {

    /*  aggressiveness is how sharply it turns up at the asymptote
     *  speed is the overall speed of increase
     *  how to include asymptote to push the threshold before acceleration?
     * y = (x + speed) ^ (2 * aggressiveness)
     */
}
*/

pub fn calculate_portion(amount: Uint128, portion: Uint128) -> Uint128 {
    /* amount: total amount sent to burn (uSSCRT/uSILK/uSHD)
     * portion: percent * 10^18 e.g. 5_320_000_000_000_000_000 = 5.32% = .0532
     *
     * return portion = amount * portion / 10^18
     */
    if portion == Uint128::zero() {
        return Uint128::zero();
    }

    amount.multiply_ratio(portion, 10u128.pow(18))
}

fn oracle<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    symbol: String,
) -> StdResult<Uint128> {
    let config: Config = config_r(&deps.storage).load()?;
    let answer: ReferenceData = Price { symbol }.query(
        &deps.querier,
        config.oracle.code_hash,
        config.oracle.address,
    )?;

    Ok(Uint128::from(answer.rate))
}
