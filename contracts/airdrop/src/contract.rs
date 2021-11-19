use cosmwasm_std::{to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdResult, Storage, Uint128, StdError};
use shade_protocol::{
    airdrop::{
        InitMsg, HandleMsg,
        QueryMsg, Config
    }
};
use crate::{state::{config_w, reward_w, claim_status_w, user_total_claimed_w, total_claimed_w},
            handle::{try_update_config, try_add_tasks, try_complete_task, try_claim},
            query };
use shade_protocol::airdrop::RequiredTask;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    // Setup task claim
    let mut task_claim= vec![RequiredTask {
        address: env.contract.address,
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

    let config = Config{
        admin: match msg.admin {
            None => { env.message.sender.clone() }
            Some(admin) => { admin }
        },
        airdrop_snip20: msg.airdrop_token.clone(),
        task_claim,
        start_date: match msg.start_time {
            None => env.block.time,
            Some(date) => date
        },
        end_date: msg.end_time
    };

    config_w(&mut deps.storage).save(&config)?;

    // Store the delegators list
    for reward in msg.rewards {
        let key = reward.address.to_string();

        reward_w(&mut deps.storage).save(key.as_bytes(), &reward)?;
        user_total_claimed_w(&mut deps.storage).save(key.as_bytes(), &Uint128::zero())?;
        // Save the initial claim
        claim_status_w(&mut deps.storage, 0).save(key.as_bytes(), &false)?;
    }

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
            admin, start_date, end_date
        } => try_update_config(deps, env, admin, start_date, end_date),
        HandleMsg::AddTasks { tasks
        } => try_add_tasks(deps, &env, tasks),
        HandleMsg::CompleteTask { address
        } => try_complete_task(deps, &env, address),
        HandleMsg::Claim { } => try_claim(deps, &env),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetConfig { } => to_binary(&query::config(&deps)?),
        QueryMsg::GetDates { } => to_binary(&query::dates(&deps)?),
        QueryMsg::GetEligibility { address } => to_binary(
            &query::airdrop_amount(&deps, address)?),
    }
}