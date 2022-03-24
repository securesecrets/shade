use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse,
    HumanAddr, Querier, StdError, StdResult, Storage, Uint128,
};

use secret_toolkit::{
    snip20::{token_info_query, register_receive_msg, send_msg},
    utils::Query,
};

use shade_protocol::bonds::{
    errors::{bond_ended, bond_not_started, limit_reached, mint_exceeds_limit},
    {Config, HandleAnswer}};
use shade_protocol::utils::generic_response::ResponseStatus;
use shade_protocol::utils::asset::Contract;
use shade_protocol::{
    snip20::{token_config_query, Snip20Asset, TokenConfig},
    oracle::QueryMsg::Price,
    band::ReferenceData,
};

use crate::state::{config_r, config_w, collateral_asset_r, collateral_asset_w, issuance_cap_r, total_minted_r};

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    admin: Option<HumanAddr>,
    oracle: Option<Contract>,
    treasury: Option<HumanAddr>,
    activated: Option<bool>,
    issuance_cap: Option<Uint128>,
    start_date: Option<u64>,
    end_date: Option<u64>,
) -> StdResult<HandleResponse> {
    let cur_config = config_r(&deps.storage).load()?;

    // Admin-only
    if env.message.sender != cur_config.admin {
        return Err(StdError::unauthorized());
    }

    let mut config = config_w(&mut deps.storage);
    config.update(|mut state| {
        if let Some(admin) = admin {
            state.admin = admin;
        }
        if let Some(oracle) = oracle {
            state.oracle = oracle;
        }
        if let Some(treasury) = treasury {
            state.treasury = treasury;
        }
        if let Some(activated) = activated {
            state.activated = activated;
        }
        if let Some(issuance_cap) = issuance_cap {
            state.issuance_cap = issuance_cap;
        }
        if let Some(start_date) = start_date {
            state.start_date = Some(start_date);
        }
        if let Some(end_date) = end_date {
            state.end_date = Some(end_date);
        }
        Ok(state)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

// Register an asset before receiving it as user deposit
pub fn try_register_collateral_asset<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    contract: &Contract,
) -> StdResult<HandleResponse> {
    let config = config_r(&deps.storage).load()?;
    // Check if admin
    if env.message.sender != config.admin {
        return Err(StdError::Unauthorized {backtrace: None });
    }
    
    // Adding the Snip20Asset to the contract's storage
    // First acquiring TokenInfo
    let asset_info = token_info_query(
        &deps.querier,
        1,
        contract.code_hash.clone(),
        contract.address.clone(),
    )?;

    // Acquiring TokenConfig
    let asset_config: Option<TokenConfig> = 
        match token_config_query(&deps.querier, contract.clone()) {
            Ok(c) => Option::from(c),
            Err(_) => None,
        };

    // Saving Snip20Asset with contract, TokenInfo, and TokenConfig copies
    debug_print!("Registering {}", asset_info.symbol);
    collateral_asset_w(&mut deps.storage).save(
        &Snip20Asset {
            contract: contract.clone(),
            token_info: asset_info,
            token_config: asset_config,
        },
    )?;

    // Enact register receive so funds sent to Bonds will call Receive
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

    // Check that bond date window hasn't ended and that the limit hasn't been reached
    let total_minted = total_minted_r(&deps.storage).load()?;
    let issuance_cap = issuance_cap_r(&deps.storage).load()?;
    active(&config, env, &total_minted)?;
    let available = (issuance_cap - total_minted).unwrap();

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

    // Calculate conversion of collateral to SHD
    let mint_amount = amount_to_mint(&deps, deposit_amount, available).unwrap();
    

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



pub fn active(config: &Config, env: &Env, total_minted: &Uint128) -> StdResult<()> {
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

    // Check whether mint limit has been reached
    if total_minted >= &config.issuance_cap {
        return Err(limit_reached(config.issuance_cap))
    }

    Ok(())
}

pub fn amount_to_mint<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    deposit_amount: Uint128,
    available: Uint128
) -> StdResult<Uint128> {
    let oracle_ratio = Uint128(1u128); // Placeholder for Oracle lookup
    let SHD_price
    let mint_amount = deposit_amount.multiply_ratio(oracle_ratio, Uint128(1)); // Potential placeholder for mint calculation, depending on what oracle returns
    if mint_amount > available {
        return Err(mint_exceeds_limit(mint_amount, available))
    }
    Ok(mint_amount)
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
    Ok(answer.rate)
}