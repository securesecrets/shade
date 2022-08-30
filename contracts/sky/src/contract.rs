use crate::{handle, query};
use shade_protocol::{
    c_std::{
        entry_point,
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

pub fn init(
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

#[entry_point]
pub fn handle(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::UpdateConfig {
            shade_admin,
            shd_token,
            silk_token,
            sscrt_token,
            treasury,
            payback_rate,
            ..
        } => handle::try_update_config(
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
        ExecuteMsg::SetCycles { cycles, .. } => handle::try_set_cycles(deps, env, info, cycles),
        ExecuteMsg::AppendCycles { cycle, .. } => handle::try_append_cycle(deps, env, info, cycle),
        ExecuteMsg::UpdateCycle { cycle, index, .. } => {
            handle::try_update_cycle(deps, env, info, cycle, index)
        }
        ExecuteMsg::RemoveCycle { index, .. } => handle::try_remove_cycle(deps, env, info, index),
        ExecuteMsg::ArbCycle { amount, index, .. } => {
            handle::try_arb_cycle(deps, env, info, amount, index)
        }
        ExecuteMsg::ArbAllCycles { amount, .. } => {
            handle::try_arb_all_cycles(deps, env, info, amount)
        }
        ExecuteMsg::Adapter(adapter) => match adapter {
            adapter::SubExecuteMsg::Unbond { asset, amount } => {
                handle::try_adapter_unbond(deps, env, info, asset, amount)
            }
            adapter::SubExecuteMsg::Claim { asset } => handle::try_adapter_claim(deps, env, asset),
            adapter::SubExecuteMsg::Update { asset } => {
                handle::try_adapter_update(deps, env, asset)
            }
        },
    }
}

#[entry_point]
pub fn query(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
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
            adapter::SubQueryMsg::Balance { asset } => {
                to_binary(&query::adapter_balance(deps, asset)?)
            }
            adapter::SubQueryMsg::Claimable { asset } => {
                to_binary(&query::adapter_claimable(deps, asset)?)
            }
            adapter::SubQueryMsg::Unbonding { asset } => {
                to_binary(&query::adapter_unbonding(deps, asset)?)
            }
            adapter::SubQueryMsg::Unbondable { asset } => {
                to_binary(&query::adapter_unbondable(deps, asset)?)
            }
            adapter::SubQueryMsg::Reserves { asset } => {
                to_binary(&query::adapter_reserves(deps, asset)?)
            }
        },
    }
}
