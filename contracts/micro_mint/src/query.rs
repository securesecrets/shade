use cosmwasm_std::{
    Api, Extern, Querier, StdError, StdResult, Storage, 
};
use crate::state::{
    config_r,
    assets_r,
    asset_list_read,
    total_burned_r,
};
use shade_protocol::{
    micro_mint::{
        QueryAnswer
    },
};

pub fn supported_assets<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::SupportedAssets { assets: asset_list_read(&deps.storage).load()? })
}

pub fn asset<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, contract: String) -> StdResult<QueryAnswer> {
    let assets = assets_r(&deps.storage);

    return match assets.may_load(contract.as_bytes())? {
        Some(asset) => {
            Ok(QueryAnswer::Asset { 
                asset, 
                burned: total_burned_r(&deps.storage).load(contract.as_bytes())?,
            })
        }
        None => Err(StdError::NotFound { kind: contract, backtrace: None }),
    };
}

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config { config: config_r(&deps.storage).load()? })
}
