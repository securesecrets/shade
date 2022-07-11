use crate::{handle, query};
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
    contract_interfaces::{
        dao::adapter,
        sky::{Config, Cycles, HandleMsg, InitMsg, QueryMsg, SelfAddr, ViewingKeys},
    },
    math_compat::Uint128,
    secret_toolkit::snip20::set_viewing_key_msg,
    utils::storage::plus::ItemStorage,
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = Config {
        shade_admin: msg.shade_admin,
        shd_token_contract: msg.shd_token_contract.clone(),
        silk_token_contract: msg.silk_token_contract.clone(),
        sscrt_token_contract: msg.sscrt_token_contract.clone(),
        treasury: msg.treasury,
        payback_rate: msg.payback_rate,
    };

    state.save(&mut deps.storage)?;
    SelfAddr(env.contract.address).save(&mut deps.storage)?;
    Cycles(vec![]).save(&mut deps.storage)?;

    let messages = vec![
        set_viewing_key_msg(
            msg.viewing_key.clone(),
            None,
            1,
            msg.shd_token_contract.code_hash.clone(),
            msg.shd_token_contract.address.clone(),
        )?,
        set_viewing_key_msg(
            msg.viewing_key.clone(),
            None,
            1,
            msg.silk_token_contract.code_hash.clone(),
            msg.silk_token_contract.address.clone(),
        )?,
        set_viewing_key_msg(
            msg.viewing_key.clone(),
            None,
            1,
            msg.sscrt_token_contract.code_hash.clone(),
            msg.sscrt_token_contract.address.clone(),
        )?,
    ];

    ViewingKeys(msg.viewing_key).save(&mut deps.storage)?;

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
    match msg {
        HandleMsg::UpdateConfig {
            shade_admin,
            shd_token_contract,
            silk_token_contract,
            sscrt_token_contract,
            treasury,
            payback_rate,
            ..
        } => handle::try_update_config(
            deps,
            env,
            shade_admin,
            shd_token_contract,
            silk_token_contract,
            sscrt_token_contract,
            treasury,
            payback_rate,
        ),
        HandleMsg::SetCycles { cycles, .. } => handle::try_set_cycles(deps, env, cycles),
        HandleMsg::AppendCycles { cycle, .. } => handle::try_append_cycle(deps, env, cycle),
        HandleMsg::RemoveCycle { index, .. } => handle::try_remove_cycle(deps, env, index),
        HandleMsg::ArbCycle { amount, index, .. } => {
            handle::try_arb_cycle(deps, env, amount, index)
        }
        HandleMsg::Adapter(adapter) => match adapter {
            adapter::SubHandleMsg::Unbond { asset, amount } => {
                handle::try_adapter_unbond(deps, env, asset, Uint128::from(amount.u128()))
            }
            adapter::SubHandleMsg::Claim { asset } => handle::try_adapter_claim(deps, env, asset),
            adapter::SubHandleMsg::Update { asset } => handle::try_adapter_update(deps, env, asset),
        },
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
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
