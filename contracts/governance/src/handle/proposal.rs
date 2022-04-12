use cosmwasm_std::{Api, Binary, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, to_binary};
use secret_cosmwasm_math_compat::Uint128;
use shade_protocol::governance::assembly::Assembly;
use shade_protocol::governance::HandleAnswer;
use shade_protocol::governance::profile::{Count, Profile};
use shade_protocol::governance::proposal::{Proposal, Status};
use shade_protocol::governance::vote::TalliedVotes;
use shade_protocol::utils::asset::Contract;
use shade_protocol::utils::generic_response::ResponseStatus;
use crate::handle::assembly::try_assembly_proposal;

// Initializes a proposal on the public assembly with the blank command
pub fn try_proposal<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    metadata: String,
    contract: Option<Uint128>,
    msg: Option<String>
) -> StdResult<HandleResponse> {
    try_assembly_proposal(
        deps,
        env,
        Uint128::zero(),
        metadata,
        contract,
        match msg {
            None => None,
            Some(_) => Some(Uint128::zero())
        },
        match msg {
            None => None,
            Some(msg) => Some(vec![msg])
        },
    )?;
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Proposal {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_trigger<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    proposal: Uint128
) -> StdResult<HandleResponse> {
    todo!();
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Trigger {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_cancel<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    proposal: Uint128
) -> StdResult<HandleResponse> {
    // Check if passed, and check if current time > cancel time
    let status = Proposal::status(&deps.storage, &proposal)?;
    if let Status::Passed {start, end} = status {
        if env.block.time < end {
            Err(StdError::unauthorized())
        }
        let mut history = Proposal::status_history(&mut deps.storage, &proposal)?;
        history.push(status);
        Proposal::save_status_history(&mut deps.storage, &proposal, history)?;
        Proposal::save_status(&mut deps.storage, &proposal, Status::Canceled)?;
    }
    else {
        Err(StdError::unauthorized())
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Cancel {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_update<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    proposal: Uint128
) -> StdResult<HandleResponse> {
    let status = Proposal::status(&deps.storage, &proposal)?;
    let new_status: Status;

    let assembly = Proposal::assembly(&deps.storage, &proposal)?;
    let profile = Assembly::data(&deps.storage, &assembly)?.profile;

    match status {
        Status::AssemblyVote { votes, start, end } => {
            if end > env.block.time {
                return Err(StdError::unauthorized())
            }

            let vote_settings = Profile::assembly_voting(&deps.storage, &profile)?.unwrap();
            let tally = TalliedVotes::tally(votes);

            // TODO: get total vote power
            let total_power = Uint128::zero();

            let threshold = match vote_settings.threshold {
                Count::Percentage { percent } => total_power.multiply_ratio(Uint128(10000), percent),
                Count::LiteralCount { count } => count
            };

            let yes_threshold = match vote_settings.yes_threshold {
                Count::Percentage { percent } => (tally.yes + tally.no).multiply_ratio(Uint128(10000), percent),
                Count::LiteralCount { count } => count
            };

            let veto_threshold = match vote_settings.veto_threshold {
                Count::Percentage { percent } => (tally.yes + tally.no).multiply_ratio(Uint128(10000), percent),
                Count::LiteralCount { count } => count
            };

            if tally.total >= threshold && tally.yes >= yes_threshold && tally.veto < veto_threshold {
                // Todo: find next status either funding, voting or passed
            }
            else if tally.total < threshold {
                new_status = Status::Expired;
            }
            else if tally.veto >= veto_threshold {
                new_status = Status::Vetoed;
            }
            else if tally.yes < yes_threshold {
                new_status = Status::Rejected;
            }
        }
        Status::Funding { amount, start, end } => {
            // Check if amount reaches limit or reaches end
            if end > env.block.time {
                return Err(StdError::unauthorized())
            }


        }
        Status::Voting { votes, start, end } => {
            if end > env.block.time {
                return Err(StdError::unauthorized())
            }


        }
        _ => Err(StdError::generic_err("Cants update"))
    }


    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Update {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_receive<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    sender: HumanAddr,
    from: HumanAddr,
    amount: Uint128,
    msg: Option<Binary>,
    memo: Option<String>
) -> StdResult<HandleResponse> {
    todo!();
    // Check if funding was passed and if
    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Receive {
            status: ResponseStatus::Success,
        })?),
    })
}