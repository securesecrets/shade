use cosmwasm_std::{to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdResult, Storage, Uint128, StdError};
use shade_protocol::{
    airdrop::{
        InitMsg, HandleMsg,
        QueryMsg, Config, claim_info::RequiredTask
    }
};
use crate::{state::{config_w, total_claimed_w},
            handle::{try_update_config, try_add_tasks, try_complete_task, try_create_account,
                     try_update_account, try_disable_permit_key, try_claim, try_claim_decay},
            query };

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    // Setup task claim
    let mut task_claim= vec![RequiredTask {
        address: env.contract.address.clone(),
        percent: msg.default_claim
    }];
    let mut claim = msg.task_claim;
    task_claim.append(&mut claim);

    // Validate claim percentage
    let mut count = Uint128::zero();
    for claim in task_claim.iter() {
        count += claim.percent;
    }

    if count > Uint128(100) {
        return Err(StdError::GenericErr { msg: "tasks above 100%".to_string(), backtrace: None })
    }

    let start_date = match msg.start_date {
        None => env.block.time,
        Some(date) => date
    };

    if let Some(end_date) = msg.end_date {
        if end_date < start_date {
            return Err(StdError::generic_err("Start date must come before end date"))
        }
    }

    // Avoid decay collisions
    if let Some(start_decay) = msg.decay_start {
        if let Some(end_date) = msg.end_date {
            if start_decay > end_date {
                return Err(StdError::generic_err("Decay cannot start after the end date"))
            }
        }
        else {
            return Err(StdError::generic_err("Decay must have an end date"))
        }
    }

    let config = Config{
        admin: match msg.admin {
            None => { env.message.sender.clone() }
            Some(admin) => { admin }
        },
        contract: env.contract.address.clone(),
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
    };

    config_w(&mut deps.storage).save(&config)?;

    // Initialize claim amount
    total_claimed_w(&mut deps.storage).save(&Uint128::zero())?;

    Ok(InitResponse {
        messages: vec![],
        log: vec![]
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::UpdateConfig {
            admin, dump_address,
            start_date, end_date, decay_start: start_decay
        } => try_update_config(deps, env, admin, dump_address,
                               start_date, end_date, start_decay),
        HandleMsg::AddTasks { tasks
        } => try_add_tasks(deps, &env, tasks),
        HandleMsg::CompleteTask { address
        } => try_complete_task(deps, &env, address),
        HandleMsg::CreateAccount { addresses, partial_tree
        } => try_create_account(deps, &env, addresses, partial_tree),
        HandleMsg::UpdateAccount { addresses, partial_tree
        } => try_update_account(deps, &env, addresses, partial_tree),
        HandleMsg::DisablePermitKey { key
        } => try_disable_permit_key(deps, &env, key),
        HandleMsg::Claim { } => try_claim(deps, &env),
        HandleMsg::ClaimDecay { } => try_claim_decay(deps, &env),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig { } => to_binary(&query::config(&deps)?),
        QueryMsg::GetDates { current_date } => to_binary(&query::dates(&deps, current_date)?),
        QueryMsg::GetAccount { permit, current_date } => to_binary(
            &query::account(&deps, permit, current_date)?),
    }
}