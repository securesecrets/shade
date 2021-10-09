use cosmwasm_std::{to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdResult, Storage, Uint128, Empty, CosmosMsg, StdError, WasmMsg, from_binary};
use shade_protocol::{
    staking::{
        InitMsg, HandleMsg,
        QueryMsg, Config,
    },
};
use crate::{
    state::{config_w},
    handle,
    query
};
use secret_toolkit::snip20::register_receive_msg;
use binary_heap_plus::{BinaryHeap, MinComparator};
use shade_protocol::{staking::Unbonding, snip20};
use crate::{handle::{try_update_unbond_time, try_stake, try_unbond, try_query_staker, try_query_stakers, try_trigger_unbounds},
            state::{unbonding_w}};
use secret_toolkit::utils::HandleCallback;

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
        unbond_time: msg.unbond_time,
        staked_token: msg.staked_token
    };

    config_w(&mut deps.storage).save(&state)?;

    // Register staked_token
    let cosmos_msg = register_receive_msg(
        env.contract_code_hash, None, 256,
        state.staked_token.code_hash.clone(),
        state.staked_token.address.clone())?;
    let mut messages = vec![cosmos_msg];

    // Initialize binary heap
    let unbonding_heap = BinaryHeap::new_min();
    unbonding_w(&mut deps.storage).save(&unbonding_heap)?;

    Ok(InitResponse {
        messages,
        log: vec![]
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::UpdateUnbondTime { unbond_time
        } => try_update_unbond_time(deps, &env, unbond_time),
        HandleMsg::Receive { sender, from, amount
        } => try_stake(deps, &env, sender, from, amount),
        HandleMsg::Unbond { amount
        } => try_unbond(deps, &env, amount),
        HandleMsg::QueryStaker { account
        } => try_query_staker(deps, &env, account),
        HandleMsg::QueryStakers { accounts
        } => try_query_stakers(deps, &env, accounts),
        HandleMsg::TriggerUnbonds { } => try_trigger_unbounds(deps, &env),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config { } => to_binary(&query::config(deps)),
        QueryMsg::TotalStaked { } => to_binary(&query::total_staked(deps)),
    }
}