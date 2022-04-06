use crate::{
    handle::{calculate_portion, mint_amount},
    state::{
        asset_list_r, asset_peg_r, assets_r, config_r, limit_r, limit_refresh_r, minted_r,
        native_asset_r, total_burned_r,
    },
};
use chrono::prelude::*;
use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage, Uint128};
use shade_protocol::mint::QueryAnswer;

pub fn native_asset<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::NativeAsset {
        asset: native_asset_r(&deps.storage).load()?,
        peg: asset_peg_r(&deps.storage).load()?,
    })
}

pub fn supported_assets<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::SupportedAssets {
        assets: asset_list_r(&deps.storage).load()?,
    })
}

pub fn asset<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract: String,
) -> StdResult<QueryAnswer> {
    let assets = assets_r(&deps.storage);

    match assets.may_load(contract.as_bytes())? {
        Some(asset) => Ok(QueryAnswer::Asset {
            asset,
            burned: total_burned_r(&deps.storage).load(contract.as_bytes())?,
        }),
        None => Err(StdError::NotFound {
            kind: contract,
            backtrace: None,
        }),
    }
}

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: config_r(&deps.storage).load()?,
    })
}

pub fn limit<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Limit {
        minted: minted_r(&deps.storage).load()?,
        limit: limit_r(&deps.storage).load()?,
        last_refresh: limit_refresh_r(&deps.storage).load()?,
    })
}

pub fn mint<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    offer_asset: HumanAddr,
    amount: Uint128,
) -> StdResult<QueryAnswer> {

    let native_asset = 

    match assets_r(&deps.storage).may_load(offer_asset.to_string().as_bytes())? {
        Some(asset) => {
            Ok(QueryAnswer::Mint {
                asset: native_asset.contract.clone(),
                amount: mint_amount(deps,
                            (amount - calculate_portion(amount, asset.fee))?,
                            &asset,
                            &native_asset_r(&deps.storage).load()?,
                        )?,
            })
        }
        None => {
            return Err(StdError::NotFound {
                kind: offer_asset.to_string(),
                backtrace: None,
            });
        }
    }
}
