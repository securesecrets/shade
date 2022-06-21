use chrono::prelude::*;
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{
    debug_print,
    from_binary,
    to_binary,
    Api,
    Binary,
    CosmosMsg,
    Env,
    Extern,
    HandleResponse,
    HumanAddr,
    Querier,
    StdError,
    StdResult,
    Storage,
};
use secret_toolkit::{
    snip20::{burn_msg, mint_msg, register_receive_msg, send_msg, token_info_query, TokenConfig, token_config_query},
    utils::Query,
};
use shade_protocol::{
    contract_interfaces::{
        mint::{
            mint,
            mint_router::{Config, HandleAnswer},
        },
        oracles::{band::ReferenceData, oracle::{QueryMsg::GetPrice, OracleAnswer}},
        snip20::helpers::Snip20Asset,
    },
    utils::{asset::Contract, generic_response::ResponseStatus},
};
use std::{cmp::Ordering, convert::TryFrom};

use crate::state::{
    asset_path_r,
    asset_path_w,
    config_r,
    config_w,
    current_assets_r,
    current_assets_w,
    final_asset_r,
    final_asset_w,
    registered_asset_r,
    registered_asset_w,
    user_r,
    user_w,
};

pub fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    sender: HumanAddr,
    from: HumanAddr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<HandleResponse> {
    let mut messages = vec![];
    let asset_paths = asset_path_r(&deps.storage);

    let mut input_asset =
        registered_asset_r(&deps.storage).load(&env.message.sender.to_string().as_bytes())?;
    let mut input_amount = amount;

    let final_asset = final_asset_r(&deps.storage).load()?;

    while input_asset.address != final_asset {
        let mint = asset_paths.load(input_asset.address.to_string().as_bytes())?;
        let (output_asset, output_amount) = match (mint::QueryMsg::Mint {
            offer_asset: input_asset.address.clone(),
            amount: input_amount,
        }
        .query(&deps.querier, mint.code_hash.clone(), mint.address.clone())?)
        {
            mint::QueryAnswer::Mint { asset, amount } => (asset, amount),
            _ => {
                return Err(StdError::generic_err("Failed to get mint asset/amount"));
            }
        };

        if output_asset.address == final_asset {
            // Send with the msg for slippage
            messages.push(send_msg(
                mint.address.clone(),
                input_amount.into(),
                msg.clone(),
                None,
                None,
                1,
                input_asset.code_hash.clone(),
                input_asset.address.clone(),
            )?);
        } else {
            // ignore slippage for intermediate steps
            messages.push(send_msg(
                mint.address.clone(),
                input_amount.into(),
                None,
                None,
                None,
                1,
                input_asset.code_hash.clone(),
                input_asset.address.clone(),
            )?);
        }

        input_asset = output_asset;
        input_amount = output_amount;
    }

    messages.push(send_msg(
        from.clone(),
        input_amount.into(),
        None,
        None,
        None,
        1,
        input_asset.code_hash.clone(),
        input_asset.address.clone(),
    )?);

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Mint {
            status: ResponseStatus::Success,
            amount,
        })?),
    })
}

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

    let mut messages = vec![];

    if cur_config.path != config.path {
        messages.append(&mut build_path(deps, env, config.path.clone())?);
    }

    config_w(&mut deps.storage).save(&config)?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn build_path<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    path: Vec<Contract>,
) -> StdResult<Vec<CosmosMsg>> {
    let mut messages = vec![];
    let mut all_assets = vec![];

    for mint in path.clone() {
        let entry_assets = match (mint::QueryMsg::SupportedAssets {}.query(
            &deps.querier,
            mint.code_hash.clone(),
            mint.address.clone(),
        )?) {
            mint::QueryAnswer::SupportedAssets { assets } => assets,
            _ => {
                return Err(StdError::generic_err("Failed to get entry assets"));
            }
        };

        all_assets.append(&mut entry_assets.clone());

        // Make sure all new assets are registered
        for asset in entry_assets.clone() {
            // Register receive if it hasn't been before
            if (registered_asset_r(&deps.storage)
                .may_load(&asset.address.to_string().as_bytes())?)
            .is_none()
            {
                messages.push(register_receive_msg(
                    env.contract_code_hash.clone(),
                    None,
                    1,
                    asset.code_hash.clone(),
                    asset.address.clone(),
                )?);
                registered_asset_w(&mut deps.storage)
                    .save(&asset.address.to_string().as_bytes(), &asset)?;
            }

            // Set this assets node to the current mint contract
            asset_path_w(&mut deps.storage).save(&asset.address.to_string().as_bytes(), &mint)?;
        }
    }

    let exit = path.last().unwrap();
    let final_asset = match (mint::QueryMsg::NativeAsset {}.query(
        &deps.querier,
        exit.code_hash.clone(),
        exit.address.clone(),
    )?) {
        mint::QueryAnswer::NativeAsset { asset, peg } => asset.contract,
        _ => {
            return Err(StdError::generic_err("Failed to get final asset"));
        }
    };

    // Ensure final asset is registered
    if (registered_asset_r(&deps.storage).may_load(&final_asset.address.to_string().as_bytes())?)
        .is_none()
    {
        messages.push(register_receive_msg(
            env.contract_code_hash.clone(),
            None,
            1,
            final_asset.code_hash.clone(),
            final_asset.address.clone(),
        )?);
        registered_asset_w(&mut deps.storage)
            .save(&final_asset.address.to_string().as_bytes(), &final_asset)?;
    }

    // remove final asset to prevent circles
    if let Some(index) = all_assets.iter().position(|a| *a == final_asset) {
        all_assets.remove(index);
    }

    final_asset_w(&mut deps.storage).save(&final_asset.address)?;
    current_assets_w(&mut deps.storage).save(&all_assets)?;

    Ok(messages)
}
