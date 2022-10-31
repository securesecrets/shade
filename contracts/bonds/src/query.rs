use crate::{
    handle::oracle,
    state::{
        account_r,
        allowance_key_r,
        bond_opportunity_r,
        config_r,
        deposit_assets_r,
        global_total_claimed_r,
        global_total_issued_r,
        issued_asset_r,
        number_of_interactions_r,
    },
};

use cosmwasm_math_compat::Uint128;

use secret_toolkit::{
    snip20::{allowance_query, balance_query},
    utils::Query,
};

use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdResult, Storage};
use shade_protocol::contract_interfaces::bonds::{
    errors::{permit_revoked, query_auth_bad_response},
    BondOpportunity,
    QueryAnswer,
};

use shade_protocol::contract_interfaces::query_auth::{
    self,
    QueryMsg::ValidatePermit,
    QueryPermit,
};

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: config_r(&deps.storage).load()?,
    })
}

pub fn account<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    permit: QueryPermit,
) -> StdResult<QueryAnswer> {
    let config = config_r(&deps.storage).load()?;
    // Validate address
    let authorized: query_auth::QueryAnswer = ValidatePermit { permit }.query(
        &deps.querier,
        config.query_auth.code_hash,
        config.query_auth.address,
    )?;
    match authorized {
        query_auth::QueryAnswer::ValidatePermit { user, is_revoked } => {
            if !is_revoked {
                account_information(deps, user)
            } else {
                return Err(permit_revoked(user.as_str()));
            }
        }
        _ => return Err(query_auth_bad_response()),
    }
}

fn account_information<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account_address: HumanAddr,
) -> StdResult<QueryAnswer> {
    let account = account_r(&deps.storage).load(account_address.as_str().as_bytes())?;

    // Return pending bonds

    Ok(QueryAnswer::Account {
        pending_bonds: account.pending_bonds,
    })
}

pub fn bond_opportunities<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    let deposit_assets = deposit_assets_r(&deps.storage).load()?;
    if deposit_assets.is_empty() {
        return Ok(QueryAnswer::BondOpportunities {
            bond_opportunities: vec![],
        });
    } else {
        let iter = deposit_assets.iter();
        let mut bond_opportunities: Vec<BondOpportunity> = vec![];
        for asset in iter {
            bond_opportunities
                .push(bond_opportunity_r(&deps.storage).load(asset.as_str().as_bytes())?);
        }
        return Ok(QueryAnswer::BondOpportunities { bond_opportunities });
    }
}

pub fn bond_info<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    let global_total_issued = global_total_issued_r(&deps.storage).load()?;
    let global_total_claimed = global_total_claimed_r(&deps.storage).load()?;
    let issued_asset = issued_asset_r(&deps.storage).load()?;
    let config = config_r(&deps.storage).load()?;
    Ok(QueryAnswer::BondInfo {
        global_total_issued,
        global_total_claimed,
        issued_asset,
        global_min_accepted_issued_price: config.global_min_accepted_issued_price,
        global_err_issued_price: config.global_err_issued_price,
    })
}

pub fn list_deposit_addresses<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    let deposit_addresses = deposit_assets_r(&deps.storage).load()?;
    Ok(QueryAnswer::DepositAddresses { deposit_addresses })
}

pub fn price_check<S: Storage, A: Api, Q: Querier>(
    asset: String,
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    let price = oracle(deps, asset)?;
    Ok(QueryAnswer::PriceCheck { price })
}

pub fn check_allowance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    let config = config_r(&deps.storage).load()?;

    // Check bond issuance amount against snip20 allowance and allocated_allowance
    let snip20_allowance = allowance_query(
        &deps.querier,
        config.treasury,
        config.contract,
        allowance_key_r(&deps.storage).load()?.to_string(),
        1,
        config.issued_asset.code_hash,
        config.issued_asset.address,
    )?;

    Ok(QueryAnswer::CheckAllowance {
        allowance: Uint128::from(snip20_allowance.allowance),
    })
}

pub fn check_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    let config = config_r(&deps.storage).load()?;

    let balance = balance_query(
        &deps.querier,
        config.contract,
        allowance_key_r(&deps.storage).load()?,
        256,
        config.issued_asset.code_hash,
        config.issued_asset.address,
    )?;

    Ok(QueryAnswer::CheckBalance {
        balance: Uint128::from(balance.amount),
    })
}

pub fn get_interactions<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::NumBondsPurchased {
        interactions: number_of_interactions_r(&deps.storage).load()?,
    })
}
