use cosmwasm_std::{
    debug_print, from_binary, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse,
    HumanAddr, Querier, StdError, StdResult, Storage, Uint128,
};
use secret_toolkit::{
    snip20::{burn_msg, mint_msg, register_receive_msg, send_msg, token_info_query},
    utils::Query,
};
use shade_protocol::utils::asset::Contract;
use shade_protocol::utils::generic_response::ResponseStatus;
use shade_protocol::{
    band::ReferenceData,
    mint_router::{Config, HandleAnswer},
    mint,
    oracle::QueryMsg::Price,
    snip20::{token_config_query, Snip20Asset, TokenConfig},
};
use std::{cmp::Ordering, convert::TryFrom};
use chrono::prelude::*;

use crate::state::{
    config_r, config_w
};

pub fn receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    sender: HumanAddr,
    from: HumanAddr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    let mut messages = vec![];

    let supported = match (mint::QueryMsg::SupportedAssets {}.query(
        &deps.querier,
        config.path[0].code_hash.clone(),
        config.path[0].address.clone(),
    )?) {
        mint::QueryAnswer::SupportedAssets { assets } => assets,
        _ => {
            return Err(StdError::generic_err("Failed to get supported assets"));
        }
    };

    let mut input_asset: Contract = match supported.into_iter().find(|asset| asset.address == sender) {
        Some(a) => a,
        None => {
            return Err(StdError::NotFound {
                kind: sender.to_string(),
                backtrace: None,
            });
        }
    };

    let mut input_amount = amount;

    let mut output_asset = Contract { address: HumanAddr("".to_string()), code_hash: "".to_string() };
    let mut output_amount = Uint128::zero();

    for mint in config.path {

        let (mint_asset, mint_amount) = match (mint::QueryMsg::Mint {
            offer_asset: input_asset.address.clone(),
            amount: input_amount,
        }.query(
            &deps.querier,
            mint.code_hash.clone(),
            mint.address.clone(),
        )?) {
            mint::QueryAnswer::Mint { asset, amount } => (asset, amount),
            _ => {
                return Err(StdError::generic_err("Failed to get mint query"));
            }

        };


        // send input_asset to mint for mint_asset
        messages.push(send_msg(
            mint.address.clone(),
            input_amount,
            None,
            None,
            None,
            1,
            input_asset.code_hash.clone(),
            input_asset.address.clone(),
        )?);

        // rotate minted->input for next iteration
        input_asset = mint_asset.clone();
        input_amount = mint_amount;

        // carry last out of the loop
        output_asset = mint_asset;
        output_amount = mint_amount;
    }

    // send final output funds to user
    messages.push(send_msg(
        from,
        output_amount,
        None,
        None,
        None,
        1,
        output_asset.code_hash.clone(),
        output_asset.address.clone(),
    )?);

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Mint {
            status: ResponseStatus::Success,
            amount: output_amount,
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

    config_w(&mut deps.storage).save(&config)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}
