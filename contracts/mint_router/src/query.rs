use crate::state::{ 
    config_r, current_assets_r,
    final_asset_r, registered_asset_r,
    asset_path_r,
};
use secret_toolkit::{
    snip20::token_info_query,
    utils::Query,
};
use cosmwasm_std::{Api, Extern, Querier, StdError, StdResult, Storage, Uint128, HumanAddr};
use shade_protocol::{
    mint_router::{ QueryAnswer, PathNode },
    mint,
};
use chrono::prelude::*;

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: config_r(&deps.storage).load()?,
    })
}

pub fn assets<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Assets {
        assets: current_assets_r(&deps.storage).load()?,
    })
}

pub fn route<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, asset: HumanAddr, amount: Uint128) -> StdResult<QueryAnswer> {

    let mut path = vec![];
    let mut input_asset = registered_asset_r(&deps.storage).load(&asset.to_string().as_bytes())?;
    let mut input_amount = amount;

    let final_asset = final_asset_r(&deps.storage).load()?;

    while input_asset.address != final_asset {

        let mint = asset_path_r(&deps.storage).load(&input_asset.address.to_string().as_bytes())?;
        let (output_asset, output_amount) = match (
            mint::QueryMsg::Mint {
                offer_asset: input_asset.address.clone(),
                amount: input_amount,
            }.query(
                &deps.querier,
                mint.code_hash.clone(),
                mint.address.clone(),
            )?
        ) {
            mint::QueryAnswer::Mint { asset, amount } => (asset, amount),
            _ => {
                return Err(StdError::generic_err("Failed to get native asset"));
            }
        };

        path.push(PathNode {
            input_asset: input_asset.address.clone(),
            input_amount,

            mint: mint.address,

            output_asset: output_asset.address.clone(),
            output_amount,
        });

        input_asset = output_asset;
        input_amount = output_amount;
    }

    Ok(QueryAnswer::Route {
        path 
    })
}
