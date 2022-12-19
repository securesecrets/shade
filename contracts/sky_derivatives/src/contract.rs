use shade_protocol::c_std::{
    shd_entry_point,
    to_binary,
    Binary,
    Decimal,
    Deps,
    DepsMut,
    Env,
    MessageInfo,
    Response,
    StdError,
    StdResult,
    SubMsg,
    Uint128,
};

use shade_protocol::{
    admin::helpers::{validate_admin, AdminPermissions},
    contract_interfaces::{
        dao::adapter,
        sky::sky_derivatives::{
            Config,
            DexPairs,
            ExecuteMsg, 
            InstantiateMsg, 
            QueryMsg,
            Rollover,
            SelfAddr,
            ViewingKey,
        },
    },
    snip20::helpers::set_viewing_key_msg,
    utils::storage::plus::ItemStorage,
};

use crate::{
    execute,
    query,
};

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    SelfAddr(env.contract.address.clone()).save(deps.storage)?;
    ViewingKey(msg.viewing_key.clone()).save(deps.storage)?;
    Rollover(Uint128::zero()).save(deps.storage)?;

    // Validate shade admin works
    validate_admin(
        &deps.querier,
        AdminPermissions::SkyAdmin,
        info.sender.to_string(),
        &msg.shade_admin_addr,
    )?;

    // Validate trading fees
    if msg.trading_fees.dex_fee > Decimal::one() || msg.trading_fees.stake_fee > Decimal::one() ||
            msg.trading_fees.unbond_fee > Decimal::one() {
        return Err(StdError::generic_err("Trading fee cannot be over 100%"));
    }

    Config {
        shade_admin_addr: msg.shade_admin_addr,
        derivative: msg.derivative.clone(),
        trading_fees: msg.trading_fees,
        max_arb_amount: msg.max_arb_amount,
        arb_period: msg.arb_period,
    }.save(deps.storage)?;

    // Clear current pairs, then add individual (validating each)
    let mut new_pairs = vec![];
    for pair in msg.dex_pairs {
        // derivative must be the 2nd entry in the dex_pair
        if !execute::validate_dex_pair(&msg.derivative, &pair) {
            return Err(StdError::generic_err(
                "Invalid pair - original tokeken must be token 0 and derivative must be token 1"
            ));
        }
        new_pairs.push(pair);
    }
    DexPairs(new_pairs).save(deps.storage)?;

    // Viewing keys
    let mut messages = vec![];
    messages.push(SubMsg::new(set_viewing_key_msg(
        msg.viewing_key.clone(),
        None,
        &msg.derivative.contract,
    )?));
    messages.push(SubMsg::new(set_viewing_key_msg(
        msg.viewing_key,
        None,
        &msg.derivative.original_token,
    )?));

    Ok(Response::new().add_submessages(messages))
}

#[shd_entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig { 
            shade_admin_addr,
            derivative,
            trading_fees,
            max_arb_amount,
            arb_period,
        } => execute::try_update_config(
            deps, 
            info,
            shade_admin_addr, 
            derivative, 
            trading_fees,
            max_arb_amount,
            arb_period,
        ),
        ExecuteMsg::SetDexPairs { pairs } => execute::try_set_dex_pairs(deps, info, pairs),
        ExecuteMsg::SetPair { pair, index } => execute::try_set_pair(deps, info, pair, index),
        ExecuteMsg::AddPair { pair } => execute::try_add_pair(deps, info, pair),
        ExecuteMsg::RemovePair { index } => execute::try_remove_pair(deps, info, index),
        ExecuteMsg::Arbitrage { index } => execute::try_arb_pair(deps.as_ref(), index),
        ExecuteMsg::ArbAllPairs {} => execute::try_arb_all_pairs(deps.as_ref()),
        ExecuteMsg::Adapter(adapter) => match adapter {
            adapter::SubExecuteMsg::Unbond { asset, amount } =>
                execute::try_adapter_unbond(deps, env, asset, Uint128::from(amount.u128())),
            adapter::SubExecuteMsg::Claim { asset } => execute::try_adapter_claim(deps, env, asset),
            adapter::SubExecuteMsg::Update { asset } => execute::try_adapter_update(deps, env, asset),
        },
    }
}

#[shd_entry_point]
pub fn query(
    deps: Deps,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::DexPairs {} => to_binary(&query::dex_pairs(deps)?),
        QueryMsg::CurrentRollover {} => to_binary(&query::current_rollover(deps)?),
        QueryMsg::IsProfitable { index, max_swap } => {
            match index {
                Some(i) => to_binary(&query::is_profitable(deps, i, max_swap)?),
                None => to_binary(&query::is_profitable(deps, 0, max_swap)?),
            }
        },
        QueryMsg::IsAnyPairProfitable { max_swap } => 
            to_binary(&query::is_any_pair_profitable(deps, max_swap)?),
        QueryMsg::Adapter(adapter) => match adapter {
            adapter::SubQueryMsg::Balance { asset } =>
                to_binary(&query::adapter_balance(deps, asset)?),
            adapter::SubQueryMsg::Claimable { asset } =>
                to_binary(&query::adapter_claimable(deps, asset)?),
            adapter::SubQueryMsg::Unbonding { asset } =>
                to_binary(&query::adapter_unbonding(deps, asset)?),
            adapter::SubQueryMsg::Unbondable { asset } =>
                to_binary(&query::adapter_unbondable(deps, asset)?),
            adapter::SubQueryMsg::Reserves { asset } =>
                to_binary(&query::adapter_reserves(deps, asset)?),
        },
    }
}
