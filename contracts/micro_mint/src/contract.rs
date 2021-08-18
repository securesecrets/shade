use cosmwasm_std::{
    debug_print, to_binary, Api, Binary,
    Env, Extern, HandleResponse, InitResponse, 
    Querier, StdResult, Storage, 
};
use secret_toolkit::snip20::token_info_query;

use shade_protocol::{
    micro_mint::{
        InitMsg, HandleMsg,
        QueryMsg, MintConfig, 
    },
    snip20::{
        Snip20Asset,
    },
};

use crate::{
    state::{
        config_w, 
        native_asset_w, 
        asset_peg_w,
        asset_list,
    },
    handle, query,
};


pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    let state = MintConfig {
        owner: match msg.admin {
            None => { env.message.sender.clone() }
            Some(admin) => { admin }
        },
        oracle: msg.oracle,
        treasury: msg.treasury,
        commission: msg.commission,
        activated: true,
    };

    config_w(&mut deps.storage).save(&state)?;
    let token_info = token_info_query(
                        &deps.querier, 1,
                        msg.native_asset.code_hash.clone(),
                        msg.native_asset.address.clone())?;

    let peg = match msg.peg {
        Some(p) => { p }
        None => { token_info.symbol.clone() }
    };
    asset_peg_w(&mut deps.storage).save(&peg)?;

    debug_print!("Setting native asset");
    native_asset_w(&mut deps.storage).save(&Snip20Asset {
        contract: msg.native_asset.clone(),
        token_info: token_info,
    })?;

    let empty_assets_list: Vec<String> = Vec::new();
    asset_list(&mut deps.storage).save(&empty_assets_list)?;

    debug_print!("Contract was initialized by {}", env.message.sender);

    Ok(InitResponse {
        messages: vec![],
        log: vec![]
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::UpdateConfig {
            owner,
            oracle,
            treasury,
            commission,
        } => handle::try_update_config(deps, env, owner, oracle, treasury, commission),
        HandleMsg::RegisterAsset {
            contract,
        } => handle::try_register_asset(deps, &env, &contract),
        HandleMsg::Receive {
            sender,
            from,
            amount,
            msg,
            ..} => handle::try_burn(deps, env, sender, from, amount, msg),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetSupportedAssets {} => to_binary(&query::supported_assets(deps)?),
        QueryMsg::GetAsset { contract } => to_binary(&query::asset(deps, contract)?),
        QueryMsg::GetConfig {} => to_binary(&query::config(deps)?),
    }
}
