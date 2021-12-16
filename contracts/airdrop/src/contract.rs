use cosmwasm_std::{to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdResult, Storage, Uint128, StdError};
use shade_protocol::{
    airdrop::{
        InitMsg, HandleMsg,
        QueryMsg, Config, claim_info::RequiredTask
    }
};
use crate::{state::{config_w, airdrop_address_w, total_claimed_w, address_in_account_w},
            handle::{try_update_config, try_add_tasks, try_complete_task, try_create_account,
                     try_update_account, try_disable_permit_key, try_claim, try_decay},
            query };
use crate::handle::try_add_reward_chunk;
use crate::state::airdrop_total_w;

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

    let config = Config{
        admin: match msg.admin {
            None => { env.message.sender.clone() }
            Some(admin) => { admin }
        },
        contract: env.contract.address.clone(),
        dump_address: msg.dump_address,
        airdrop_snip20: msg.airdrop_token.clone(),
        task_claim,
        start_date: match msg.start_time {
            None => env.block.time,
            Some(date) => date
        },
        end_date: msg.end_time
    };

    config_w(&mut deps.storage).save(&config)?;

    // Initialize claim amount
    total_claimed_w(&mut deps.storage).save(&Uint128::zero())?;

    airdrop_total_w(&mut deps.storage).save(&Uint128::zero())?;

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
            start_date, end_date
        } => try_update_config(deps, env, admin, dump_address,
                               start_date, end_date),
        HandleMsg::AddRewardChunk { reward_chunk
        } => try_add_reward_chunk(deps, env, reward_chunk),
        HandleMsg::AddTasks { tasks
        } => try_add_tasks(deps, &env, tasks),
        HandleMsg::CompleteTask { address
        } => try_complete_task(deps, &env, address),
        HandleMsg::CreateAccount { addresses
        } => try_create_account(deps, &env, addresses),
        HandleMsg::UpdateAccount { addresses
        } => try_update_account(deps, &env, addresses),
        HandleMsg::DisablePermitKey { key
        } => try_disable_permit_key(deps, &env, key),
        HandleMsg::Claim { } => try_claim(deps, &env),
        HandleMsg::Decay { } => try_decay(deps, &env),
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
        QueryMsg::GetAccount { address, permit } => to_binary(
            &query::account(&deps, address, permit)?),
    }
}