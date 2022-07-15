use shade_protocol::c_std::Uint128;
use shade_protocol::c_std::{
    to_binary, Api, Binary, Env, DepsMut, Response, Querier, StdResult, Storage,
};

use shade_protocol::snip20::helpers::{set_viewing_key_msg, token_info_query};

use shade_protocol::contract_interfaces::{
    bonds::{Config, ExecuteMsg, InstantiateMsg, QueryMsg, SnipViewingKey},
    snip20::helpers::Snip20Asset,
};

use shade_protocol::snip20::helpers::token_config;
use shade_protocol::utils::{pad_handle_result, pad_query_result};

use crate::{
    handle::{self, register_receive},
    query,
    state::{
        allocated_allowance_w, allowance_key_w, deposit_assets_w, config_w,
        global_total_claimed_w, global_total_issued_w, issued_asset_w,
    },
};

// Used to pad up responses for better privacy.
pub const RESPONSE_BLOCK_SIZE: usize = 256;

pub fn init(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = Config {
        limit_admin: msg.limit_admin,
        shade_admin: msg.shade_admin,
        oracle: msg.oracle,
        treasury: msg.treasury,
        issued_asset: msg.issued_asset,
        global_issuance_limit: msg.global_issuance_limit,
        global_minimum_bonding_period: msg.global_minimum_bonding_period,
        global_maximum_discount: msg.global_maximum_discount,
        activated: msg.activated,
        discount: msg.discount,
        bond_issuance_limit: msg.bond_issuance_limit,
        bonding_period: msg.bonding_period,
        global_min_accepted_issued_price: msg.global_min_accepted_issued_price,
        global_err_issued_price: msg.global_err_issued_price,
        contract: env.contract.address.clone(),
        airdrop: msg.airdrop,
        query_auth: msg.query_auth,
    };

    config_w(deps.storage).save(&state)?;

    let mut messages = vec![];

    let allowance_key: SnipViewingKey =
        SnipViewingKey::new(&env, Default::default(), msg.allowance_key_entropy.as_ref());
    messages.push(set_viewing_key_msg(
        allowance_key.0.clone(),
        None,
        256,
        state.issued_asset.code_hash.clone(),
        state.issued_asset.address.clone(),
    )?);
    allowance_key_w(deps.storage).save(&allowance_key.0)?;

    let token_info = token_info_query(
        &deps.querier,
        1,
        state.issued_asset.code_hash.clone(),
        state.issued_asset.address.clone(),
    )?;

    let token_config = token_config(
        &deps.querier,
        256,
        state.issued_asset.code_hash.clone(),
        state.issued_asset.address.clone(),
    )?;

    issued_asset_w(deps.storage).save(&Snip20Asset {
        contract: state.issued_asset.clone(),
        token_info,
        token_config: Option::from(token_config),
    })?;

    messages.push(register_receive(&env, &state.issued_asset)?);

    // Write initial values to storage
    global_total_issued_w(deps.storage).save(&Uint128::zero())?;
    global_total_claimed_w(deps.storage).save(&Uint128::zero())?;
    allocated_allowance_w(deps.storage).save(&Uint128::zero())?;
    deposit_assets_w(deps.storage).save(&vec![])?;

    Ok(Response::new())
}

pub fn handle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    pad_handle_result(
        match msg {
            ExecuteMsg::UpdateLimitConfig {
                limit_admin,
                shade_admin,
                global_issuance_limit,
                global_minimum_bonding_period,
                global_maximum_discount,
                reset_total_issued,
                reset_total_claimed,
                ..
            } => handle::try_update_limit_config(
                deps,
                env,
                limit_admin,
                shade_admin,
                global_issuance_limit,
                global_minimum_bonding_period,
                global_maximum_discount,
                reset_total_issued,
                reset_total_claimed,
            ),
            ExecuteMsg::UpdateConfig {
                oracle,
                treasury,
                issued_asset,
                activated,
                bond_issuance_limit,
                bonding_period,
                discount,
                global_min_accepted_issued_price,
                global_err_issued_price,
                allowance_key,
                airdrop,
                query_auth,
                ..
            } => handle::try_update_config(
                deps,
                env,
                oracle,
                treasury,
                activated,
                issued_asset,
                bond_issuance_limit,
                bonding_period,
                discount,
                global_min_accepted_issued_price,
                global_err_issued_price,
                allowance_key,
                airdrop,
                query_auth,
            ),
            ExecuteMsg::OpenBond {
                deposit_asset,
                start_time,
                end_time,
                bond_issuance_limit,
                bonding_period,
                discount,
                max_accepted_deposit_price,
                err_deposit_price,
                minting_bond,
                ..
            } => handle::try_open_bond(
                deps,
                env,
                deposit_asset,
                start_time,
                end_time,
                bond_issuance_limit,
                bonding_period,
                discount,
                max_accepted_deposit_price,
                err_deposit_price,
                minting_bond,
            ),
            ExecuteMsg::CloseBond {
                deposit_asset, ..
            } => handle::try_close_bond(deps, env, info, deposit_asset),
            ExecuteMsg::Receive {
                sender,
                from,
                amount,
                msg,
                ..
            } => handle::try_deposit(deps, &env, sender, from, amount, msg),
            ExecuteMsg::Claim { .. } => handle::try_claim(deps, env),
        },
        RESPONSE_BLOCK_SIZE,
    )
}

pub fn query(
    deps: Deps,
    msg: QueryMsg,
) -> StdResult<Binary> {
    pad_query_result(
        match msg {
            QueryMsg::Config {} => to_binary(&query::config(deps)?),
            QueryMsg::BondOpportunities {} => to_binary(&query::bond_opportunities(deps)?),
            QueryMsg::Account { permit } => to_binary(&query::account(deps, permit)?),
            QueryMsg::DepositAddresses {} => to_binary(&query::list_deposit_addresses(deps)?),
            QueryMsg::PriceCheck { asset } => to_binary(&query::price_check(asset, deps)?),
            QueryMsg::BondInfo {} => to_binary(&query::bond_info(deps)?),
            QueryMsg::CheckAllowance {} => to_binary(&query::check_allowance(deps)?),
            QueryMsg::CheckBalance {} => to_binary(&query::check_balance(deps)?),
        },
        RESPONSE_BLOCK_SIZE,
    )
}
