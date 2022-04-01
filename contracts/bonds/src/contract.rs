use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    StdResult, Storage, Uint128,
};
use secret_toolkit::snip20::token_info_query;

use shade_protocol::{
    bonds::{Config, InitMsg, HandleMsg, QueryMsg},
    snip20::{token_config_query, Snip20Asset},
};

use crate::{handle, query, state::{config_w, minted_asset_w}};

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
        mint_asset: msg.mint_asset,
        global_issuance_limit: msg.global_issuance_limit,
        activated: msg.activated,
        global_minimum_claim_time: msg.global_minimum_claim_time,
    };

    config_w(&mut deps.storage).save(&state)?;

    let token_info = token_info_query(
        &deps.querier, 
        1, 
        msg.mint_asset.code_hash.clone(),
        msg.mint_asset.address.clone(),
    )?;

    let token_config = token_config_query(&deps.querier, msg.mint_asset.clone())?;

    debug_print!("Setting minted asset");
    minted_asset_w(&mut deps.storage).save(&Snip20Asset {
        contract: msg.mint_asset.clone(),
        token_info,
        token_config: Option::from(token_config),
    })?;

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
            global_minimum_claim_time,
            global_minimum_bonding_period,
            global_maximum_discount,
        } => handle::try_update_limit_config(deps, env, limit_admin, global_issuance_limit),
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
        } => handle::try_update_config(deps, env, admin, oracle, treasury, activated, issued_asset),
        HandleMsg::OpenBond{
            collateral_asset,
            start_time,
            end_time,
            bond_issuance_limit,
            bonding_period,
            discount,
        } => handle::try_open_bond(deps, env, collateral_asset, start_time, end_time, bond_issuance_limit, bonding_period),
        HandleMsg::Receive { 
            sender,
            from,
            amount,
            msg,
        } => handle::try_deposit(deps, &env, sender, from, amount, msg),
        HandleMsg::RegisterCollateralAsset {collateral_asset} => handle::try_register_collateral_asset(deps, &env, &collateral_asset),
        HandleMsg::RemoveCollateralAsset {collateral_asset} => handle::try_remove_collateral_asset(deps, &env, &collateral_asset),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::IssuanceCap {} => to_binary(&query::issuance_cap(deps)?),
        QueryMsg::TotalMinted {} => to_binary(&query::total_minted(deps)?),
        QueryMsg::CollateralAsset {} => to_binary(&query::collateral_asset(deps)?),
    }
}

