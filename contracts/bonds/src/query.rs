use std::arch::global_asm;

use crate::{
    state::{
        config_r, global_total_issued_r, global_total_claimed_r, account_viewkey_r, account_r, bond_opportunity_r, collateral_assets_r, issued_asset_r, validate_account_permit
    }, handle::oracle,
};
use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage, Uint128};
use shade_protocol::{bonds::{QueryAnswer, AccountKey, BondOpportunity, AccountPermit}, snip20::Snip20Asset, oracle};
use shade_protocol::bonds::errors::incorrect_viewing_key;
use query_authentication::viewing_keys::ViewingKey;



pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: config_r(&deps.storage).load()?,
    })
}

pub fn account<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    permit: AccountPermit,
) -> StdResult<QueryAnswer> {
    let config = config_r(&deps.storage).load()?;
    // Validate address
    let contract = config.contract;

    account_information(deps, validate_account_permit(deps, &permit, contract)?)
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

pub fn bond_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    let global_total_issued = global_total_issued_r(&deps.storage).load()?;
    let global_total_claimed = global_total_claimed_r(&deps.storage).load()?;
    let issued_asset = issued_asset_r(&deps.storage).load()?;
    Ok(QueryAnswer::BondInfo {
        global_total_issued: global_total_issued,
        global_total_claimed: global_total_claimed,
        issued_asset: issued_asset,
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

pub fn price_check<S: Storage, A: Api, Q: Querier>(
    asset: String,
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    let price = oracle(deps, asset)?;
    Ok(QueryAnswer::PriceCheck {
        price: price
    })
}