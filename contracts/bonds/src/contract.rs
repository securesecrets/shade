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
        admin: msg.admin,
        oracle: msg.oracle,
        treasury: msg.treasury,
        issuance_cap: msg.issuance_cap,
        activated: msg.activated,
        start_date: msg.start_date,
        end_date: msg.end_date,
    };

    config_w(&mut deps.storage).save(&state)?;

    let token_info = token_info_query(
        &deps.querier, 
        1, 
        msg.minted_asset.code_hash.clone(),
        msg.minted_asset.address.clone(),
    )?;

    let token_config = token_config_query(&deps.querier, msg.minted_asset.clone())?;

    debug_print!("Setting minted asset");
    minted_asset_w(&mut deps.storage).save(&Snip20Asset {
        contract: msg.minted_asset.clone(),
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
        HandleMsg::UpdateConfig { 
            admin,
            oracle,
            treasury,
            issuance_cap,
            activated,
            start_date,
            end_date,
         } => handle::try_update_config(deps, env, admin, oracle, treasury, minted_asset, activated, issuance_cap, start_date),
        HandleMsg::Receive { 
            sender,
            from,
            amount,
            msg,
        } => handle::try_deposit(deps, &env, sender, from, amount, msg),
        HandleMsg::RegisterAsset {contract} => handle::try_register_asset(deps, &env, &contract),
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

