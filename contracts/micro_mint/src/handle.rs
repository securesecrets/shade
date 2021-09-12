use std::convert::TryFrom;
use cosmwasm_std::{debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage, CosmosMsg, HumanAddr, Uint128, from_binary, Empty};
use secret_toolkit::{
    snip20::{
        token_info_query, 
        mint_msg, burn_msg, send_msg,
        register_receive_msg, 
    },
};
use secret_toolkit::utils::Query;
use shade_protocol::{
    micro_mint::{
        HandleAnswer,
        Config,
        SupportedAsset,
    },
    mint::MintMsgHook,
    snip20::{Snip20Asset, token_config_query, TokenConfig},
    oracle::{
        QueryMsg::GetPrice,
    },
    band::ReferenceData,
    asset::Contract,
    generic_response::ResponseStatus,
};

use crate::state::{config_w, config_r, native_asset_r, asset_peg_r, assets_w, assets_r, asset_list_w, total_burned_w, limit_w, limit_r};

pub fn try_burn<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _sender: HumanAddr,
    from: HumanAddr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    // Check if contract enabled
    if !config.activated {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Prevent sender to be native asset
    if native_asset_r(&deps.storage).load()?.contract.address == env.message.sender {
        return Err(StdError::generic_err("Sender cannot be the same as the native asset."))
    }

    // Check that sender is a supported snip20 asset
    let assets = assets_r(&deps.storage);
    let burn_asset = match assets.may_load(env.message.sender.to_string().as_bytes())? {
        Some(supported_asset) => {
            debug_print!("Found Burn Asset: {} {}",
                         &supported_asset.asset.token_info.symbol,
                         env.message.sender.to_string());
            supported_asset
        },
        None => return Err(StdError::NotFound {
            kind: env.message.sender.to_string(),
            backtrace: None
        }),
    };

    // Setup msgs
    let mut messages = vec![];
    let msgs: MintMsgHook = match msg {
        Some(x) => from_binary(&x)?,
        None => return Err(StdError::generic_err("data cannot be empty")),
    };

    let mut burn_amount = amount;
    if let Some(treasury) = config.treasury {
        // Ignore commission if the set commission is 0
        if burn_asset.commission != Uint128(0) {
            let commission_amount = calculate_commission(amount, burn_asset.commission);

            // Commission to treasury
            messages.push(send_msg(treasury.address,
                                   commission_amount,
                                   None,
                                   None,
                                   1,
                                   burn_asset.asset.contract.code_hash.clone(),
                                   burn_asset.asset.contract.address.clone())?);

            burn_amount = (amount - commission_amount)?;
        }
    }

    //TODO: if token_config is None, or cant burn, need to trash

    // Try to burn
    match burn_asset.asset.token_config {
        Some(ref conf) => {
            if conf.burn_enabled {
                messages.push(burn_msg(burn_amount,
                                       None,
                                       256,
                                       burn_asset.asset.contract.code_hash.clone(),
                                       burn_asset.asset.contract.address.clone())?);
            }
        }
        None => {
        }
    }

    // Update burned amount
    total_burned_w(&mut deps.storage).update(
        burn_asset.asset.contract.address.to_string().as_bytes(),
        |burned| {
            match burned {
                Some(burned) => { Ok(burned + burn_amount) }
                None => { Ok(burn_amount) }
            }
        })?;

    let mint_asset = native_asset_r(&deps.storage).load()?;

    // This will calculate the total mint value
    let amount_to_mint: Uint128 = mint_amount(&deps, amount, &burn_asset, &mint_asset)?;

    // Check against slippage amount
    if amount_to_mint < msgs.minimum_expected_amount {
        return Err(StdError::generic_err("Mint amount is less than the minimum expected."))
    }

    // Check against mint cap
    let mut limit_storage = limit_w(&mut deps.storage);
    let mut limit = limit_storage.load()?;

    // When frequency is 0 it means that mint limits are disabled
    if limit.frequency != 0 {
        // Reset total and next epoch
        if limit.next_epoch <= env.block.time {
            limit.next_epoch = env.block.time + limit.frequency;
            limit.total_minted = Uint128(0);
        }

        let new_total = limit.total_minted + amount_to_mint;

        if new_total > limit.mint_capacity {
            return Err(StdError::generic_err("Amount to be minted exceeds mint capacity"))
        }

        limit.total_minted = new_total;

        limit_storage.save(&limit);
    }

    debug_print!("Minting: {} {}", amount_to_mint, &mint_asset.token_info.symbol);

    messages.push(mint_msg(from,
                           amount_to_mint,
                           None,
                           256,
                           mint_asset.contract.code_hash.clone(),
                           mint_asset.contract.address.clone())?);

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some( to_binary( &HandleAnswer::Burn {
            status: ResponseStatus::Success,
            mint_amount: amount_to_mint
        } )? ),
    })
}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    owner: Option<HumanAddr>,
    oracle: Option<Contract>,
    treasury: Option<Contract>,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    // Check if admin
    if env.message.sender != config.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }
    // Check if contract enabled
    if !config.activated {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Save new info
    let mut config = config_w(&mut deps.storage);
    config.update(|mut state| {
        if let Some(owner) = owner {
            state.owner = owner;
        }
        if let Some(oracle) = oracle {
            state.oracle = oracle;
        }
        if let Some(treasury) = treasury {
            state.treasury = Some(treasury);
        }
        Ok(state)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success } )? )
    })
}

pub fn try_update_limit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    epoch_frequency: Option<Uint128>,
    epoch_limit: Option<Uint128>,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;
    // Check if admin
    if env.message.sender != config.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }
    // Check if contract enabled
    if !config.activated {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Reset limit and set new limits
    let mut limit = limit_w(&mut deps.storage);
    limit.update(|mut state| {
        if let Some(frequency) = epoch_frequency {
            state.frequency = frequency.u128() as u64;
        }
        if let Some(limit) = epoch_limit {
            state.mint_capacity = limit
        }
        // Reset total minted
        state.total_minted = Uint128(0);

        // Reset next epoch
        if state.frequency == 0 {
            state.next_epoch = 0;
        } else {
            state.next_epoch = env.block.time + state.frequency;
        }
        Ok(state)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::UpdateMintLimit {
            status: ResponseStatus::Success } )? )
    })
}

pub fn try_register_asset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    contract: &Contract,
    commission: Option<Uint128>
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;
    // Check if admin
    if env.message.sender != config.owner {
        return Err(StdError::Unauthorized { backtrace: None });
    }
    // Check if contract enabled
    if !config.activated {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    let mut assets = assets_w(&mut deps.storage);
    let contract_str = contract.address.to_string();

    // Add the new asset
    let asset_info = token_info_query(&deps.querier, 1,
                                      contract.code_hash.clone(),
                                      contract.address.clone())?;

    let asset_config: Option<TokenConfig> = match token_config_query(&deps.querier, contract.clone()) {
        Ok(c) => { Option::from(c) }
        Err(_) => { None }
    };

    debug_print!("Registering {}", asset_info.symbol);
    assets.save(&contract_str.as_bytes(), &SupportedAsset {
        asset: Snip20Asset {
            contract: contract.clone(),
            token_info: asset_info,
            token_config: asset_config,
        },
        // If commission is not set then default to 0
        commission: match commission {
            None => Uint128(0),
            Some(value) => value
        }
    })?;

    total_burned_w(&mut deps.storage).save(&contract_str.as_bytes(), &Uint128(0))?;

    // Add the asset to list
    asset_list_w(&mut deps.storage).update(|mut state| {
        state.push(contract_str);
        Ok(state)
    })?;

    // Register contract in asset
    let mut messages = vec![];
    messages.push(register_receive(env, contract)?);

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some( to_binary( 
            &HandleAnswer::RegisterAsset {
                status: ResponseStatus::Success } 
            )? 
        )
    })
}


pub fn register_receive (
    env: &Env,
    contract: &Contract,
) -> StdResult<CosmosMsg> {
    let cosmos_msg = register_receive_msg(
        env.contract_code_hash.clone(),
        None,
        256,
        contract.code_hash.clone(),
        contract.address.clone(),
    );

    cosmos_msg
}

pub fn mint_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    burn_amount: Uint128,
    burn_asset: &SupportedAsset,
    mint_asset: &Snip20Asset, 
) -> StdResult<Uint128> {


    debug_print!("Burning {} {} for {}", 
                 burn_amount, 
                 burn_asset.asset.token_info.symbol,
                 mint_asset.token_info.symbol);

    let burn_price = oracle(&deps, &burn_asset.asset.token_info.symbol)?;
    debug_print!("Burn Price: {}", burn_price);

    let mint_price = oracle(&deps, &asset_peg_r(&deps.storage).load()?)?;
    debug_print!("Mint Price: {}", mint_price);

    Ok(calculate_mint(burn_price, burn_amount, burn_asset.asset.token_info.decimals,
                   mint_price, mint_asset.token_info.decimals))
}

pub fn calculate_mint(burn_price: Uint128, burn_amount: Uint128, burn_decimals: u8, 
                  mint_price: Uint128, mint_decimals: u8
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
    if difference < 0 {
        burn_value.multiply_ratio(1u128, 10u128.pow(u32::try_from(difference.abs()).unwrap()))
    }
    else if difference > 0 {
        Uint128(burn_value.u128() * 10u128.pow(u32::try_from(difference).unwrap()))
    }
    else {
        burn_value
    }
}

pub fn calculate_commission(
    amount: Uint128, commission: Uint128
) -> Uint128 {
    /* amount: total amount sent to burn (uSSCRT/uSILK/uSHD)
     * commission: commission_percent * 10,000 e.g. 532 = 5.32% = .0532
     *
     * commission_amount = amount * commission / 10000
     */

    return amount.multiply_ratio(commission,  10000u128);
}

fn oracle<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    symbol: &String,
) -> StdResult<Uint128> {

    let config: Config = config_r(&deps.storage).load()?;
    let answer: ReferenceData = GetPrice { 
        symbol: symbol.to_string() 
    }.query(&deps.querier,
             config.oracle.code_hash,
             config.oracle.address)?;
    Ok(answer.rate)
}
