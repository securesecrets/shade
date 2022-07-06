use crate::{handle, query};
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{
    self,
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
};
use secret_toolkit::snip20::set_viewing_key_msg;

use shade_protocol::{
    contract_interfaces::{
        dao::adapter,
        sky::{Config, Cycles, HandleMsg, InitMsg, QueryMsg, SelfAddr, ViewingKeys},
    },
    utils::storage::plus::ItemStorage,
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = Config {
        shade_admin: msg.shade_admin,
        mint_contract_shd: msg.mint_contract_shd,
        mint_contract_silk: msg.mint_contract_silk,
        market_swap_contract: msg.market_swap_contract,
        shd_token_contract: msg.shd_token_contract.clone(),
        silk_token_contract: msg.silk_token_contract.clone(),
        treasury: msg.treasury,
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
            mint_contract_silk,
            mint_contract_shd,
            market_swap_contract,
            shd_token_contract,
            silk_token_contract,
            treasury,
            ..
        } => handle::try_update_config(
            deps,
            env,
            shade_admin,
            mint_contract_shd,
            mint_contract_silk,
            market_swap_contract,
            shd_token_contract,
            silk_token_contract,
            treasury,
        ),
        HandleMsg::ArbPeg { amount, .. } => handle::try_execute(deps, env, amount),
        HandleMsg::SetCycles { cycles, .. } => handle::try_set_cycles(deps, env, cycles),
        HandleMsg::AppendCycles { cycle, .. } => handle::try_append_cycle(deps, env, cycle),
        HandleMsg::RemoveCycle { index, .. } => handle::try_remove_cycle(deps, env, index),
        HandleMsg::ArbCycle { amount, index, .. } => {
            handle::try_arb_cycle(deps, env, amount, index)
        }
        //HandleMsg::ArbAllCycles{ amount } => handle::try_arb_all_cycles(deps, env, amount ),
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
        QueryMsg::IsArbPegProfitable { amount } => {
            to_binary(&query::conversion_mint_profitability(deps, amount)?)
        }
        QueryMsg::Balance {} => to_binary(&query::get_balances(deps)?),
        QueryMsg::GetCycles {} => to_binary(&query::get_cycles(deps)?),
        QueryMsg::IsCycleProfitable { amount, index } => {
            to_binary(&query::cycle_profitability(deps, amount, index)?)
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
