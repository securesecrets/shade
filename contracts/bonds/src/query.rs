use crate::{
    state::{
        config_r, global_total_issued_r, global_total_claimed_r, account_viewkey_r, account_r, bond_opportunity_r, collateral_assets_r, issued_asset_r
    }
};
use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage, Uint128};
use shade_protocol::bonds::{QueryAnswer, AccountKey, BondOpportunity};
use shade_protocol::bonds::errors::incorrect_viewing_key;
use query_authentication::viewing_keys::ViewingKey;



pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: config_r(&deps.storage).load()?,
    })
}

pub fn account_with_key<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: HumanAddr,
    key: String,
) -> StdResult<QueryAnswer> {
    // Validate address
    let stored_hash = account_viewkey_r(&deps.storage).load(account.to_string().as_bytes())?;

    if !AccountKey(key).compare(&stored_hash) {
        return Err(incorrect_viewing_key());
    }

    account_information(deps, account)
}

fn account_information<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account_address: HumanAddr,
) -> StdResult<QueryAnswer> {
    let account = account_r(&deps.storage).load(account_address.as_str().as_bytes())?;

    // Return pending bonds

    Ok(QueryAnswer::Account { 
        pending_bonds: account.pending_bonds 
    })
}

pub fn bond_opportunities<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    let collateral_assets = collateral_assets_r(&deps.storage).load()?;
    if collateral_assets.is_empty(){
        return Ok(QueryAnswer::BondOpportunities {
            bond_opportunities: vec![]
        })
    } else {
        let iter = collateral_assets.iter();
        let mut bond_opportunities: Vec<BondOpportunity> = vec![];
        for asset in iter {
            bond_opportunities.push(bond_opportunity_r(&deps.storage).load(asset.as_str().as_bytes())?);
        }
        return Ok(QueryAnswer::BondOpportunities {
            bond_opportunities: bond_opportunities
        })
    }
}

pub fn global_total_issued<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    let global_total_issued = global_total_issued_r(&deps.storage).load()?;
    Ok(QueryAnswer::GlobalTotalIssued {
        global_total_issued: global_total_issued
    })
}

pub fn global_total_claimed<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    let global_total_claimed = global_total_claimed_r(&deps.storage).load()?;
    Ok(QueryAnswer::GlobalTotalClaimed {
        global_total_claimed: global_total_claimed
    })
}

pub fn list_collateral_addresses<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    let collateral_addresses = collateral_assets_r(&deps.storage).load()?;
    Ok(QueryAnswer::CollateralAddresses {
        collateral_addresses: collateral_addresses
    })
}

pub fn issued_asset<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::IssuedAsset {
        issued_asset: issued_asset_r(&deps.storage).load()?
    })
}