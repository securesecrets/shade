use shade_protocol::{
    c_std::{
        to_binary,
        Api,
        Binary,
        Env,
        Extern,
        HandleResponse,
        InitResponse,
        Querier,
        StdResult,
        Storage,
    },
    math_compat::Uint128,
};

use shade_protocol::secret_toolkit::snip20::set_viewing_key_msg;

use shade_protocol::contract_interfaces::{
    bonds::{
        errors::{bond_discount_above_maximum_rate, bonding_period_below_minimum_time},
        Config,
        HandleMsg,
        InitMsg,
        QueryMsg,
        SnipViewingKey,
    },
    snip20::helpers::fetch_snip20,
};

use shade_protocol::secret_toolkit::utils::{pad_handle_result, pad_query_result};

use crate::{
    handle::{self, register_receive},
    query,
    state::{
        allocated_allowance_w,
        allowance_key_w,
        config_w,
        deposit_assets_w,
        global_total_claimed_w,
        global_total_issued_w,
        issued_asset_w,
        number_of_interactions_w,
    },
};

// Used to pad up responses for better privacy.
pub const RESPONSE_BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    if msg.bonding_period < msg.global_minimum_bonding_period {
        return Err(bonding_period_below_minimum_time(
            msg.bonding_period,
            msg.global_minimum_bonding_period,
        ));
    }

    if msg.discount > msg.global_maximum_discount {
        return Err(bond_discount_above_maximum_rate(
            msg.discount,
            msg.global_maximum_discount,
        ));
    }

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

    config_w(&mut deps.storage).save(&state)?;

    let mut messages = vec![];

    let allowance_key: SnipViewingKey =
        SnipViewingKey::new(&env, Default::default(), msg.allowance_key_entropy.as_ref());
    messages.push(set_viewing_key_msg(
        allowance_key.0.clone(),
        None,
        RESPONSE_BLOCK_SIZE,
        state.issued_asset.code_hash.clone(),
        state.issued_asset.address.clone(),
    )?);
    allowance_key_w(&mut deps.storage).save(&allowance_key.0)?;

    let issued_asset_info = fetch_snip20(&state.issued_asset.clone(), &deps.querier)?;

    issued_asset_w(&mut deps.storage).save(&issued_asset_info)?;

    messages.push(register_receive(&env, &state.issued_asset)?);

    // Write initial values to storage
    global_total_issued_w(&mut deps.storage).save(&Uint128::zero())?;
    global_total_claimed_w(&mut deps.storage).save(&Uint128::zero())?;
    allocated_allowance_w(&mut deps.storage).save(&Uint128::zero())?;
    deposit_assets_w(&mut deps.storage).save(&vec![])?;
    number_of_interactions_w(&mut deps.storage).save(&0u64)?;

    Ok(InitResponse {
        messages,
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    pad_handle_result(
        match msg {
            HandleMsg::UpdateLimitConfig {
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
            HandleMsg::UpdateConfig {
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
            HandleMsg::OpenBond {
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
            HandleMsg::CloseBond { deposit_asset, .. } => {
                handle::try_close_bond(deps, env, deposit_asset)
            }
            HandleMsg::Receive {
                sender,
                from,
                amount,
                msg,
                ..
            } => handle::try_deposit(deps, &env, sender, from, amount, msg),
            HandleMsg::Claim { .. } => handle::try_claim(deps, env),
        },
        RESPONSE_BLOCK_SIZE,
    )
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
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
            QueryMsg::Metrics {} => to_binary(&query::get_interactions(deps)?),
        },
        RESPONSE_BLOCK_SIZE,
    )
}
