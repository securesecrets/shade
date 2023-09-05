use crate::{execute, query};
use shade_protocol::{
    c_std::{
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
    },
    contract_interfaces::{
        dao::adapter,
        sky::{Config, Cycles, ExecuteMsg, InstantiateMsg, QueryMsg, SelfAddr, ViewingKeys},
    },
    snip20::helpers::set_viewing_key_msg,
    utils::storage::plus::ItemStorage,
};

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = Config {
        shade_admin: msg.shade_admin,
        shd_token: msg.shd_token.clone(),
        silk_token: msg.silk_token.clone(),
        sscrt_token: msg.sscrt_token.clone(),
        treasury: msg.treasury,
        payback_rate: msg.payback_rate,
    };

    if msg.payback_rate == Decimal::zero() {
        return Err(StdError::generic_err("payback rate cannot be zero"));
    }

    state.save(deps.storage)?;
    SelfAddr(env.contract.address).save(deps.storage)?;
    Cycles(vec![]).save(deps.storage)?;

    deps.api
        .debug(&format!("Contract was initialized by {}", info.sender));

    let messages = vec![
        SubMsg::new(set_viewing_key_msg(
            msg.viewing_key.clone().to_string(),
            None,
            &msg.shd_token.clone(),
        )?),
        SubMsg::new(set_viewing_key_msg(
            msg.viewing_key.clone().to_string(),
            None,
            &msg.silk_token.clone(),
        )?),
        SubMsg::new(set_viewing_key_msg(
            msg.viewing_key.clone().to_string(),
            None,
            &msg.sscrt_token.clone(),
        )?),
    ];

    ViewingKeys(msg.viewing_key).save(deps.storage)?;

    Ok(Response::new().add_submessages(messages))
}

#[shd_entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig {
            shade_admin,
            shd_token,
            silk_token,
            sscrt_token,
            treasury,
            payback_rate,
            ..
        } => execute::try_update_config(
            deps,
            env,
            info,
            shade_admin,
            shd_token,
            silk_token,
            sscrt_token,
            treasury,
            payback_rate,
        ),
        ExecuteMsg::SetCycles { cycles, .. } => execute::try_set_cycles(deps, env, info, cycles),
        ExecuteMsg::AppendCycles { cycle, .. } => execute::try_append_cycle(deps, env, info, cycle),
        ExecuteMsg::UpdateCycle { cycle, index, .. } => {
            execute::try_update_cycle(deps, env, info, cycle, index)
        }
        ExecuteMsg::RemoveCycle { index, .. } => execute::try_remove_cycle(deps, env, info, index),
        ExecuteMsg::ArbCycle { amount, index, .. } => {
            execute::try_arb_cycle(deps, env, info, amount, index)
        }
        ExecuteMsg::ArbAllCycles { amount, .. } => {
            execute::try_arb_all_cycles(deps, env, info, amount)
        }
        ExecuteMsg::Adapter(adapter) => match adapter {
            adapter::SubExecuteMsg::Unbond { asset, amount } => {
                let asset = deps.api.addr_validate(&asset)?;
                execute::try_adapter_unbond(deps, env, info, asset, amount)
            }
            adapter::SubExecuteMsg::Claim { asset } => {
                let asset = deps.api.addr_validate(&asset)?;
                execute::try_adapter_claim(deps, env, asset)
            }
            adapter::SubExecuteMsg::Update { asset } => {
                let asset = deps.api.addr_validate(&asset)?;
                execute::try_adapter_update(deps, env, asset)
            }
        },
    }
}

#[shd_entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig {} => to_binary(&query::config(deps)?),
        QueryMsg::Balance {} => to_binary(&query::get_balances(deps)?),
        QueryMsg::GetCycles {} => to_binary(&query::get_cycles(deps)?),
        QueryMsg::IsCycleProfitable { amount, index } => {
            to_binary(&query::cycle_profitability(deps, amount, index)?)
        }
        QueryMsg::IsAnyCycleProfitable { amount } => {
            to_binary(&query::any_cycles_profitable(deps, amount)?)
        }
        QueryMsg::Adapter(adapter) => match adapter {
            adapter::SubQueryMsg::Balance { asset } => to_binary(&query::adapter_balance(
                deps,
                deps.api.addr_validate(&asset)?,
            )?),
            adapter::SubQueryMsg::Claimable { asset } => to_binary(&query::adapter_claimable(
                deps,
                deps.api.addr_validate(&asset)?,
            )?),
            adapter::SubQueryMsg::Unbonding { asset } => to_binary(&query::adapter_unbonding(
                deps,
                deps.api.addr_validate(&asset)?,
            )?),
            adapter::SubQueryMsg::Unbondable { asset } => to_binary(&query::adapter_unbondable(
                deps,
                deps.api.addr_validate(&asset)?,
            )?),
            adapter::SubQueryMsg::Reserves { asset } => to_binary(&query::adapter_reserves(
                deps,
                deps.api.addr_validate(&asset)?,
            )?),
        },
    }
}
