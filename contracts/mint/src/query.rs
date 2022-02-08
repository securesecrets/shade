use crate::{
    state::{
        asset_list_r, asset_peg_r, assets_r, config_r, limit_r, native_asset_r, total_burned_r,
        minted_r, limit_refresh_r,
    },
    handle::mint_amount,
};
use cosmwasm_std::{Api, Extern, Querier, StdError, StdResult, Storage, Uint128, HumanAddr};
use shade_protocol::mint::QueryAnswer;
use chrono::prelude::*;

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

pub fn mint<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, offer_asset: HumanAddr, amount: Uint128) -> StdResult<QueryAnswer> {

    let native_asset = native_asset_r(&deps.storage).load()?;

    match assets_r(&deps.storage).may_load(offer_asset.to_string().as_bytes())? {
        Some(asset) => {
            Ok(QueryAnswer::Mint {
                asset: native_asset.contract.clone(),
                amount: mint_amount(deps, amount, &asset, &native_asset)?,
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
