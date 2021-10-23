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
use crate::{handle::{try_update_config, try_stake, try_unbond, try_get_staker, try_get_stakers, try_trigger_unbounds},
            state::{unbonding_w}};
use secret_toolkit::utils::HandleCallback;
use crate::state::total_staked_w;
use crate::handle::try_vote;
use shade_protocol::asset::Contract;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    let state = Config {
        admin: match msg.admin {
            None => { Contract { address: env.message.sender.clone(), code_hash: "".to_string() } }
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

    // Initialize total staked
    total_staked_w(&mut deps.storage).save(&Uint128(0))?;

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
        HandleMsg::UpdateConfig { admin, unbond_time
        } => try_update_config(deps, &env, admin, unbond_time),
        HandleMsg::Receive { sender, from, amount
        } => try_stake(deps, &env, sender, from, amount),
        HandleMsg::Unbond { amount
        } => try_unbond(deps, &env, amount),
        HandleMsg::Vote { proposal_id, votes
        } => try_vote(deps, &env, proposal_id, votes),
        HandleMsg::GetStaker { account
        } => try_get_staker(deps, &env, account),
        HandleMsg::GetStakers { accounts
        } => try_get_stakers(deps, &env, accounts),
        HandleMsg::TriggerUnbonds { } => try_trigger_unbounds(deps, &env),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config { } => to_binary(&query::config(deps)?),
        QueryMsg::TotalStaked { } => to_binary(&query::total_staked(deps)?),
    }
}