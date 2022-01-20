use crate::state::{
    asset_list_r, asset_peg_r, assets_r, config_r, limit_r, native_asset_r, total_burned_r,
};
use cosmwasm_std::{Api, Extern, Querier, StdError, StdResult, Storage};
use shade_protocol::micro_mint::QueryAnswer;

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

    return match assets.may_load(contract.as_bytes())? {
        Some(asset) => Ok(QueryAnswer::Asset {
            asset,
            burned: total_burned_r(&deps.storage).load(contract.as_bytes())?,
        }),
        None => Err(StdError::NotFound {
            kind: contract,
            backtrace: None,
        }),
    };
}

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: config_r(&deps.storage).load()?,
    })
}

pub fn limit<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::MintLimit {
        limit: limit_r(&deps.storage).load()?,
    })
}
