use crate::{
    handle::{
        try_claim_rewards, try_claim_unbond, try_set_viewing_key, try_stake, try_unbond,
        try_update_config, try_vote,
    },
    query,
    state::{config_w, stake_state_w, unbonding_w},
};
use binary_heap_plus::BinaryHeap;
use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdResult, Storage,
};
use secret_toolkit::snip20::register_receive_msg;
use shade_protocol::staking::{stake::Stake, Config, HandleMsg, InitMsg, QueryMsg};
use shade_protocol::utils::asset::Contract;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = Config {
        admin: match msg.admin {
            None => Contract {
                address: env.message.sender.clone(),
                code_hash: "".to_string(),
            },
            Some(admin) => admin,
        },
        unbond_time: msg.unbond_time,
        staked_token: msg.staked_token,
    };

    config_w(&mut deps.storage).save(&state)?;

    // Register staked_token
    let cosmos_msg = register_receive_msg(
        env.contract_code_hash,
        None,
        256,
        state.staked_token.code_hash.clone(),
        state.staked_token.address,
    )?;

    // Initialize binary heap
    let unbonding_heap = BinaryHeap::new_min();
    unbonding_w(&mut deps.storage).save(&unbonding_heap)?;

    // Initialize stake state
    stake_state_w(&mut deps.storage).save(&Stake {
        total_shares: Uint128::zero(),
        total_tokens: Uint128::zero(),
    })?;

    Ok(InitResponse {
        messages: vec![cosmos_msg],
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::UpdateConfig { admin, unbond_time } => {
            try_update_config(deps, &env, admin, unbond_time)
        }
        HandleMsg::Receive {
            sender,
            from,
            amount,
        } => try_stake(deps, &env, sender, from, amount),
        HandleMsg::Unbond { amount } => try_unbond(deps, &env, amount),
        HandleMsg::Vote { proposal_id, votes } => try_vote(deps, &env, proposal_id, votes),
        HandleMsg::ClaimUnbond {} => try_claim_unbond(deps, &env),
        HandleMsg::ClaimRewards {} => try_claim_rewards(deps, &env),
        HandleMsg::SetViewingKey { key } => try_set_viewing_key(deps, &env, key),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query::config(deps)?),
        QueryMsg::TotalStaked {} => to_binary(&query::total_staked(deps)?),
        QueryMsg::TotalUnbonding { start, end } => {
            to_binary(&query::total_unbonding(deps, start, end)?)
        }
        QueryMsg::UserStake { address, key, time } => {
            to_binary(&query::user_stake(deps, address, key, time)?)
        }
    }
}
