use shade_protocol::c_std::{
    shd_entry_point,
    to_binary,
    Binary,
    Deps,
    DepsMut,
    Env,
    MessageInfo,
    Response,
    StdResult,
    SubMsg,
    Uint128,
};

use shade_protocol::{
    contract_interfaces::{
        dao::adapter,
        sky::sky_derivatives::{
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
    SelfAddr(env.contract.address).save(deps.storage)?;
    ViewingKey(msg.viewing_key).save(deps.storage)?;
    Rollover(Uint128::zero()).save(deps.storage)?;

    // Use handle's functions for data validation
    execute::try_update_config(
        deps,
        env,
        info,
        Some(msg.shade_admin_addr),
        Some(msg.derivative.clone()),
        Some(msg.trading_fees),
        Some(msg.max_arb_amount),
        Some(msg.arb_period),
    )?;
    execute::try_set_dex_pairs(deps, env, info, msg.dex_pairs)?;

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
            env, 
            info,
            shade_admin_addr, 
            derivative, 
            trading_fees,
            max_arb_amount,
            arb_period,
        ),
        ExecuteMsg::SetDexPairs { pairs } => execute::try_set_dex_pairs(deps, env, info, pairs),
        ExecuteMsg::SetPair { pair, index } => execute::try_set_pair(deps, env, info, pair, index),
        ExecuteMsg::AddPair { pair } => execute::try_add_pair(deps, env, info, pair),
        ExecuteMsg::RemovePair { index } => execute::try_remove_pair(deps, env, info, index),
        ExecuteMsg::Arbitrage { index } => execute::try_arb_pair(deps, index),
        ExecuteMsg::ArbAllPairs {} => execute::try_arb_all_pairs(deps),
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
        QueryMsg::IsProfitable { 
            index,
            max_swap,
        } => {
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
