use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    StdResult, Storage, Uint128,
};
use secret_toolkit::snip20::token_info_query;

use shade_protocol::{
    micro_mint::{Config, HandleMsg, InitMsg, QueryMsg},
    snip20::{token_config_query, Snip20Asset},
};

use crate::{
    handle, query,
    state::{asset_list_w, asset_peg_w, config_w, limit_w, native_asset_w},
};
//use shade_protocol::micro_mint::MintLimit;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    let state = Config {
        admin: match msg.admin {
            None => env.message.sender.clone(),
            Some(admin) => admin,
        },
        oracle: msg.oracle,
        treasury: msg.treasury,
        secondary_burn: msg.secondary_burn,
        limit: msg.limit,
        activated: true,
    };

    config_w(&mut deps.storage).save(&state)?;

    let token_info = token_info_query(
        &deps.querier,
        1,
        msg.native_asset.code_hash.clone(),
        msg.native_asset.address.clone(),
    )?;

    let token_config = token_config_query(&deps.querier, msg.native_asset.clone())?;

    let peg = match msg.peg {
        Some(p) => p,
        None => token_info.symbol.clone(),
    };
    asset_peg_w(&mut deps.storage).save(&peg)?;

    debug_print!("Setting native asset");
    native_asset_w(&mut deps.storage).save(&Snip20Asset {
        contract: msg.native_asset.clone(),
        token_info,
        token_config: Option::from(token_config),
    })?;

    let empty_assets_list: Vec<String> = Vec::new();
    asset_list_w(&mut deps.storage).save(&empty_assets_list)?;

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
    match msg {
        HandleMsg::UpdateConfig {
            config,
        } => handle::try_update_config(deps, env, config),
        HandleMsg::RegisterAsset { contract, capture, unlimited } => 
            handle::try_register_asset(deps, &env, &contract, capture, unlimited),
        HandleMsg::RemoveAsset { address } => handle::try_remove_asset(deps, &env, address),
        HandleMsg::Receive {
            sender,
            from,
            amount,
            msg,
            ..
        } => handle::try_burn(deps, env, sender, from, amount, msg),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::NativeAsset {} => to_binary(&query::native_asset(deps)?),
        QueryMsg::SupportedAssets {} => to_binary(&query::supported_assets(deps)?),
        QueryMsg::Asset { contract } => to_binary(&query::asset(deps, contract)?),
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Limit {} => to_binary(&query::limit(deps)?),
    }
}
