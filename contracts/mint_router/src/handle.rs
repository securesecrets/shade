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
    _sender: HumanAddr,
    from: HumanAddr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<HandleResponse> {

    let config = config_r(&deps.storage).load()?;

    let mut messages = vec![];

    let mut input_asset = assets_r(&deps.storage).load(env.message.sender.to_string().as_bytes())?;
    let mut input_amount = amount;

    let mut output_asset: Snip20Asset;
    let mut output_amount: Uint128; 

    for mint in config.path {

        // Get amount/denom received
        let mint_asset: mint::QueryAnswer::NativeAsset = mint::QueryMsg::NativeAsset {};
        let mint_amount: mint::QueryAnswer::Mint = mint::QueryMsg::Mint {
            offer_asset: input_asset.contract.address.clone(),
            amount: input_amount,
        };

        // carry these and they will be set properly after last iteration
        output_asset = mint_asset.asset;
        output_amount = mint_amount.amount;

        // send input_asset to mint for mint_asset
        messages.push(send_msg(
            mint.address,
            input_amount,
            None,
            None,
            None,
            1,
            input_asset.contract.code_hash.clone(),
            input_asset.contract.address.clone(),
        )?);

        // rotate output->input for next iteration
        input_asset = mint_asset.asset;
        input_amount = mint_amount.amount;
    }

    // send final funds to user
    messages.push(send_msg(
        from,
        output_amount,
        None,
        None,
        None,
        1,
        output_asset.contract.code_hash.clone(),
        output_asset.contract.address.clone(),
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

    config_w(&mut deps.storage).save(&config);

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success,
        })?),
    })
}
