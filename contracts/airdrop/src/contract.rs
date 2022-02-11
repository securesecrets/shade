use crate::{
    handle::{
        try_add_tasks, try_claim, try_claim_decay, try_complete_task, try_create_account,
        try_disable_permit_key, try_update_account, try_update_config,
    },
    query,
    state::{config_w, decay_claimed_w, total_claimed_w},
};
use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError,
    StdResult, Storage, Uint128,
};
use secret_toolkit::utils::{pad_handle_result, pad_query_result};
use shade_protocol::airdrop::{claim_info::RequiredTask, Config, HandleMsg, InitMsg, QueryMsg};
use shade_protocol::airdrop::errors::{invalid_dates, invalid_task_percentage};
use crate::handle::try_account;

// Used to pad up responses for better privacy.
pub const RESPONSE_BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    // Setup task claim
    let mut task_claim = vec![RequiredTask {
        address: env.contract.address.clone(),
        percent: msg.default_claim,
    }];
    let mut claim = msg.task_claim;
    task_claim.append(&mut claim);

    // Validate claim percentage
    let mut count = Uint128::zero();
    for claim in task_claim.iter() {
        count += claim.percent;
    }

    if count > Uint128(100) {
        return Err(invalid_task_percentage(count.to_string().as_str()));
    }

    let start_date = match msg.start_date {
        None => env.block.time,
        Some(date) => date,
    };

    if let Some(end_date) = msg.end_date {
        if end_date < start_date {
            return Err(invalid_dates(
                "EndDate",
                end_date.to_string().as_str(),
                "before",
                "StartDate",
                start_date.to_string().as_str()
            ));
        }
    }

    // Avoid decay collisions
    if let Some(start_decay) = msg.decay_start {
        if start_decay < start_date {
            return Err(invalid_dates(
                "Decay",
                start_decay.to_string().as_str(),
                "before",
                "StartDate",
                start_date.to_string().as_str()
            ));
        }
        if let Some(end_date) = msg.end_date {
            if start_decay > end_date {
                return Err(invalid_dates(
                    "EndDate",
                    end_date.to_string().as_str(),
                    "before",
                    "Decay",
                    start_decay.to_string().as_str()
                ));
            }
        } else {
            return Err(StdError::generic_err("Decay must have an end date"));
        }
    }

    let config = Config {
        admin: msg.admin.unwrap_or(env.message.sender),
        contract: env.contract.address,
        dump_address: msg.dump_address,
        airdrop_snip20: msg.airdrop_token.clone(),
        airdrop_amount: msg.airdrop_amount,
        task_claim,
        start_date,
        end_date: msg.end_date,
        decay_start: msg.decay_start,
        merkle_root: msg.merkle_root,
        total_accounts: msg.total_accounts,
        max_amount: msg.max_amount,
        query_rounding: msg.query_rounding,
    };

    config_w(&mut deps.storage).save(&config)?;

    // Initialize claim amount
    total_claimed_w(&mut deps.storage).save(&Uint128::zero())?;

    decay_claimed_w(&mut deps.storage).save(&false)?;

    Ok(InitResponse {
        messages: vec![],
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
            HandleMsg::UpdateConfig {
                admin,
                dump_address,
                query_rounding: redeem_step_size,
                start_date,
                end_date,
                decay_start: start_decay,
                ..
            } => try_update_config(
                deps,
                env,
                admin,
                dump_address,
                redeem_step_size,
                start_date,
                end_date,
                start_decay,
            ),
            HandleMsg::AddTasks { tasks, .. } => try_add_tasks(deps, &env, tasks),
            HandleMsg::CompleteTask { address, .. } => try_complete_task(deps, &env, address),
            HandleMsg::Account {
                addresses,
                partial_tree,
                ..
            } => try_account(deps, &env, addresses, partial_tree),
            HandleMsg::DisablePermitKey { key, .. } => try_disable_permit_key(deps, &env, key),
            HandleMsg::Claim { .. } => try_claim(deps, &env),
            HandleMsg::ClaimDecay { .. } => try_claim_decay(deps, &env),
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
            QueryMsg::Dates { current_date } => to_binary(&query::dates(deps, current_date)?),
            QueryMsg::TotalClaimed {} => to_binary(&query::total_claimed(deps)?),
            QueryMsg::Account {
                permit,
                current_date,
            } => to_binary(&query::account(deps, permit, current_date)?),
        },
        RESPONSE_BLOCK_SIZE,
    )
}
