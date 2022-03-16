use cosmwasm_std::{
    debug_print, from_binary, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse,
    HumanAddr, Querier, StdError, StdResult, Storage, Uint128,
};

use secret_toolkit::{
    snip20::{token_info_query, register_receive_msg, send_msg}
};

use shade_protocol::bonds::{
    errors::{bond_ended, bond_not_started},
    {Config, HandleAnswer}};
use shade_protocol::utils::generic_response::ResponseStatus;
use shade_protocol::utils::asset::Contract;
use shade_protocol::snip20::{token_config_query, Snip20Asset, TokenConfig};

use crate::state::{config_r, config_w, collateral_asset_r, collateral_asset_w};

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    config: Config,
) -> StdResult<HandleResponse> {
    let cur_config = config_r(&deps.storage).load()?;

    // Admin-only
    if env.message.sender != cur_config.admin {
        return Err(StdError::unauthorized());
    }

    config_w(&mut deps.storage).save(&config)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

// Register an asset before receiving it as user deposit
pub fn try_register_asset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    contract: &Contract,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;
    // Check if admin
    if env.message.sender != config.admin {
        return Err(StdError::Unauthorized {backtrace: None });
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
        match token_config_query(&deps.querier, contract.clone()) {
            Ok(c) => Option::from(c),
            Err(_) => None,
        };

    debug_print!("Registering {}", asset_info.symbol);
    collateral_asset_w(&mut deps.storage).save(
        &Snip20Asset {
            contract: contract.clone(),
            token_info: asset_info,
            token_config: asset_config,
        },
    )?;

    // Register contract in asset
    let messages = vec![register_receive(env, contract)?];

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RegisterAsset {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_deposit<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    _sender: HumanAddr,
    from: HumanAddr,
    deposit_amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<HandleResponse>{
    // Check if bond is active
    let config = config_r(&deps.storage).load()?;

    // Check that sender isn't the treasury
    if config.treasury == env.message.sender {
        return Err(StdError::generic_err(
            "Sender cannot be the treasury.",
        ));
    }

    // Check that bond hasn't ended
    available(&config, env)?;

    // Check that sender is a supported snip20 asset
    let deposit_asset = 
        match collateral_asset_r(&deps.storage).may_load()? {
            Some(collateral_asset) => {
                debug_print!(
                    "Found Collateral Asset: {} {}",
                    &collateral_asset.token_info.symbol,
                    env.message.sender.to_string()
                );
                collateral_asset
            }
            None => {
                return Err(StdError::NotFound {
                    kind: env.message.sender.to_string(),
                    backtrace: None,
                });
            }
        };
    
    let mut messages = vec![];
    
    // Collateral to treasury
    messages.push(send_msg(
        config.treasury,
        deposit_amount,
        None,
        None,
        None,
        1,
        deposit_asset.contract.code_hash.clone(),
        deposit_asset.contract.address.clone(),
    )?);

    // Give user their tokens
}

pub fn try_claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    sender: HumanAddr,
    from: HumanAddr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<HandleResponse> {
    //TODO, should check if bonding period has elapsed and allow user to claim
    //however much SHD they paid for with their deposit

    // Return Success response
    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Claim {
            status: ResponseStatus::Success,
            amount: amount_to_mint,
        })?),
    })
}



pub fn available(config: &Config, env: &Env) -> StdResult<()> {
    let current_time = env.block.time;

    // Check if bond has opened
    if let Some(start_date) = config.start_date {
        if current_time < start_date {
            return Err(bond_not_started(
                start_date.to_string().as_str(),
                current_time.to_string().as_str(),
            ));
        }
    }

    // Check if bond is still open
    if let Some(end_date) = config.end_date {
        if current_time > end_date {
            return Err(bond_ended(
                end_date.to_string().as_str(),
                current_time.to_string().as_str(),
            ));
        }
    }

    Ok(())
}

pub fn register_receive(env: &Env, contract: &Contract) -> StdResult<CosmoMsg> {
    register_receive_msg(
        env.contract_code_hash.clone(),
        None,
        256,
        contract.code_hash.clone(),
        contract.address.clone(),
    )
}