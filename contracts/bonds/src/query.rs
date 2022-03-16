use crate::{
    state::{
        config_r, issuance_cap_r, total_minted_r
    }
};
use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage, Uint128};
use shade_protocol::bonds::QueryAnswer;


pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: config_r(&deps.storage).load()?,
    })
}

pub fn issuance_cap<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::IssuanceCap {
        issuance_cap: issuance_cap_r(&deps.storage).load()?,
    })
}

pub fn total_minted<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::TotalMinted {
        total_minted: total_minted_r(&deps.storage).load()?,
    })
}

pub fn collateral_asset<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::CollateralAsset {
        collateral_asset: collateral_asset_r(&deps.storage).load()?,
    })
}

pub fn claim_status<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::ClaimStatus {
        claim_status: claim_status_r(&deps.storage).load()?,
    })
}
