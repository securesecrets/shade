use shade_protocol::{
    c_std::{
        shd_entry_point,
        to_binary,
        Api,
        Binary,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Querier,
        Response,
        StdResult,
        Storage,
    },
    snip20::helpers::{token_config, token_info},
};

use shade_protocol::contract_interfaces::{
    mint::mint::{Config, ExecuteMsg, InstantiateMsg, QueryMsg},
    snip20::helpers::Snip20Asset,
};

use crate::{
    handle,
    query,
    state::{asset_list_w, asset_peg_w, config_w, limit_w, native_asset_w},
};

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = Config {
        admin: match msg.admin {
            None => info.sender.clone(),
            Some(admin) => admin,
        },
        oracle: msg.oracle,
        treasury: msg.treasury,
        secondary_burn: msg.secondary_burn,
        limit: msg.limit,
        activated: true,
    };

    config_w(deps.storage).save(&state)?;

    let token_info = token_info(&deps.querier, &msg.native_asset)?;

    let token_config = token_config(&deps.querier, &msg.native_asset)?;

    let peg = match msg.peg {
        Some(p) => p,
        None => token_info.symbol.clone(),
    };
    asset_peg_w(deps.storage).save(&peg)?;

    deps.api.debug("Setting native asset");
    native_asset_w(deps.storage).save(&Snip20Asset {
        contract: msg.native_asset.clone(),
        token_info,
        token_config: Option::from(token_config),
    })?;

    asset_list_w(deps.storage).save(&vec![])?;

    deps.api
        .debug(&format!("Contract was initialized by {}", info.sender));

    Ok(Response::new())
}

#[shd_entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig { config } => handle::try_update_config(deps, env, info, config),
        ExecuteMsg::RegisterAsset {
            contract,
            capture,
            fee,
            unlimited,
        } => handle::try_register_asset(deps, &env, info, &contract, capture, fee, unlimited),
        ExecuteMsg::RemoveAsset { address } => handle::try_remove_asset(deps, &env, address),
        ExecuteMsg::Receive {
            sender,
            from,
            amount,
            msg,
            ..
        } => handle::try_burn(deps, env, info, sender, from, amount, msg),
    }
}

#[shd_entry_point]
pub fn query(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::NativeAsset {} => to_binary(&query::native_asset(deps)?),
        QueryMsg::SupportedAssets {} => to_binary(&query::supported_assets(deps)?),
        QueryMsg::Asset { contract } => to_binary(&query::asset(deps, contract)?),
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::Limit {} => to_binary(&query::limit(deps)?),
        QueryMsg::Mint {
            offer_asset,
            amount,
        } => to_binary(&query::mint(deps, offer_asset, amount)?),
    }
}
