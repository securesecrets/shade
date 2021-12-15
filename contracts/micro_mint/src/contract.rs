use cosmwasm_std::{debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdResult, Storage, Uint128};
use secret_toolkit::snip20::token_info_query;

use shade_protocol::{
    micro_mint::{
        InitMsg, HandleMsg,
        QueryMsg, Config,
    },
    snip20::{
        Snip20Asset,
        token_config_query,
    },
};

use crate::{
    state::{
        config_w,
        native_asset_w,
        asset_peg_w,
        asset_list_w,
    },
    handle, query,
};
use shade_protocol::micro_mint::MintLimit;
use crate::state::limit_w;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    let state = Config {
        admin: match msg.admin {
            None => { env.message.sender.clone() }
            Some(admin) => { admin }
        },
        oracle: msg.oracle,
        treasury: msg.treasury,
        secondary_burn: msg.secondary_burn,
        activated: true,
    };

    // Set the minting limit
    let mut limit = MintLimit {
        frequency: match msg.epoch_frequency {
            None => 0,
            Some(frequency) => frequency.u128() as u64,
        },
        mint_capacity: match msg.epoch_mint_limit {
            None => Uint128(0),
            Some(capacity) => capacity
        },
        total_minted: Uint128(0),
        next_epoch: match msg.epoch_frequency {
            None => 0,
            Some(frequency) => env.block.time + frequency.u128() as u64,
        },
    };
    // Override the next epoch
    if let Some(next_epoch) = msg.start_epoch {
        limit.next_epoch = next_epoch.u128() as u64;
    }

    limit_w(&mut deps.storage).save(&limit)?;

    config_w(&mut deps.storage).save(&state)?;
    let token_info = token_info_query(
                        &deps.querier, 1,
                        msg.native_asset.code_hash.clone(),
                        msg.native_asset.address.clone())?;

    let token_config = token_config_query(&deps.querier,
                                          msg.native_asset.clone())?;

    let peg = match msg.peg {
        Some(p) => { p }
        None => { token_info.symbol.clone() }
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
            admin: owner,
            oracle,
            treasury,
            secondary_burn,
        } => handle::try_update_config(deps, env, owner, oracle, treasury, secondary_burn),
        HandleMsg::UpdateMintLimit {
            start_epoch,
            epoch_frequency,
            epoch_limit,
        } => handle::try_update_limit(deps, env, start_epoch, epoch_frequency, epoch_limit),
        HandleMsg::RegisterAsset {
            contract,
            capture,
        } => handle::try_register_asset(deps, &env, &contract, capture),
        HandleMsg::RemoveAsset {
            address
        } => handle::try_remove_asset(deps, &env, address),
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
        QueryMsg::GetNativeAsset {} => to_binary(&query::native_asset(deps)?),
        QueryMsg::GetSupportedAssets {} => to_binary(&query::supported_assets(deps)?),
        QueryMsg::GetAsset { contract } => to_binary(&query::asset(deps, contract)?),
        QueryMsg::GetConfig {} => to_binary(&query::config(deps)?),
        QueryMsg::GetMintLimit {} => to_binary(&query::limit(deps)?),
    }
}
