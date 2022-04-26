use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    StdResult, Storage, Uint128, HumanAddr,
};
use secret_toolkit::snip20::token_info_query;

use shade_protocol::{
    bonds::{Config, InitMsg, HandleMsg, QueryMsg},
    snip20::{token_config_query, Snip20Asset},
};

use crate::{handle::{self, try_set_viewing_key}, query, state::{config_w, issued_asset_w, global_total_issued_w, collateral_assets_w}};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = Config {
        limit_admin: msg.limit_admin,
        admin: msg.admin,
        oracle: msg.oracle,
        treasury: msg.treasury,
        issued_asset: msg.issued_asset,
        global_issuance_limit: msg.global_issuance_limit,
        global_minimum_bonding_period: msg.global_minimum_bonding_period,
        global_maximum_discount: msg.global_maximum_discount,
        activated: msg.activated,
        minting_bond: msg.minting_bond,
        discount: msg.discount,
        bond_issuance_limit: msg.bond_issuance_limit,
        bonding_period: msg.bonding_period,
        };

    config_w(&mut deps.storage).save(&state)?;

    let token_info = token_info_query(
        &deps.querier,
        1,
        state.issued_asset.code_hash.clone(),
        state.issued_asset.address.clone(),
    )?;

    let token_config = token_config_query(&deps.querier, state.issued_asset.clone())?;

    debug_print!("Setting minted asset");
    issued_asset_w(&mut deps.storage).save(&Snip20Asset {
        contract: state.issued_asset.clone(),
        token_info,
        token_config: Option::from(token_config),
    })?;

    // Write initial values to storage
    global_total_issued_w(&mut deps.storage).save(&Uint128(0))?;
    let assets: Vec<HumanAddr> = vec![];
    collateral_assets_w(&mut deps.storage).save(&assets)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse {
        messages: vec![],
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg{
        HandleMsg::UpdateLimitConfig {
            limit_admin,
            global_issuance_limit,
            global_minimum_bonding_period,
            global_maximum_discount,
            reset_total_issued,
            reset_total_claimed,
        } => handle::try_update_limit_config(deps, env, limit_admin, global_issuance_limit, global_minimum_bonding_period, global_maximum_discount, reset_total_issued, reset_total_claimed),
        HandleMsg::UpdateConfig { 
            admin,
            oracle,
            treasury,
            issued_asset,
            activated,
            minting_bond,
            bond_issuance_limit,
            bonding_period,
            discount,
        } => handle::try_update_config(deps, env, admin, oracle, treasury, activated, issued_asset, minting_bond, bond_issuance_limit, bonding_period, discount),
        HandleMsg::OpenBond{
            collateral_asset,
            start_time,
            end_time,
            bond_issuance_limit,
            bonding_period,
            discount,
        } => handle::try_open_bond(deps, env, collateral_asset, start_time, end_time, bond_issuance_limit, bonding_period, discount),
        HandleMsg::CloseBond{
            collateral_asset
        } => handle::try_close_bond(deps, env, collateral_asset),
        HandleMsg::Receive { 
            sender,
            from,
            amount,
            msg,
        } => handle::try_deposit(deps, &env, sender, from, amount, msg),
        HandleMsg::Claim {} => handle::try_claim(deps, env),
        HandleMsg::SetViewingKey { key } => try_set_viewing_key(deps, &env, key),
        //HandleMsg::RegisterCollateralAsset {collateral_asset} => handle::try_register_collateral_asset(deps, &env, &collateral_asset),
        //HandleMsg::RemoveCollateralAsset {collateral_asset} => handle::try_remove_collateral_asset(deps, &env, &collateral_asset),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::IssuedAsset {} => to_binary(&query::issued_asset(deps)?),
        QueryMsg::GlobalTotalIssued {} => to_binary(&query::global_total_issued(deps)?),
        QueryMsg::GlobalTotalClaimed {} => to_binary(&query::global_total_claimed(deps)?),
        QueryMsg::BondOpportunities {} => to_binary(&query::bond_opportunities(deps)?),
        QueryMsg::AccountWithKey {account, key} => to_binary(&query::account_with_key(deps, account, key)?),
        QueryMsg::CollateralAddresses {} => to_binary(&query::list_collateral_addresses(deps)?),
    }
}

