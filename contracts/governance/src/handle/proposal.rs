use cosmwasm_std::{Api, Binary, Env, Extern, from_binary, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, to_binary};
use cosmwasm_math_compat::Uint128;
use secret_toolkit::snip20::send_msg;
use secret_toolkit::utils::Query;
use shade_protocol::governance::assembly::Assembly;
use shade_protocol::governance::{Config, HandleAnswer};
use shade_protocol::governance::profile::{Count, Profile, VoteProfile};
use shade_protocol::governance::proposal::{Proposal, Status};
use shade_protocol::governance::vote::{TalliedVotes, Vote};
use shade_protocol::shd_staking;
use shade_protocol::utils::asset::Contract;
use shade_protocol::utils::generic_response::ResponseStatus;
use shade_protocol::utils::storage::SingletonStorage;
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

fn validate_votes(votes: Vote, total_power: Uint128, settings: VoteProfile) -> Status {
    let tally = TalliedVotes::tally(votes);

    let threshold = match settings.threshold {
        Count::Percentage { percent } => total_power.multiply_ratio(Uint128(10000), percent),
        Count::LiteralCount { count } => count
    };

    let yes_threshold = match settings.yes_threshold {
        Count::Percentage { percent } => (tally.yes + tally.no).multiply_ratio(Uint128(10000), percent),
        Count::LiteralCount { count } => count
    };

    let veto_threshold = match settings.veto_threshold {
        Count::Percentage { percent } => (tally.yes + tally.no).multiply_ratio(Uint128(10000), percent),
        Count::LiteralCount { count } => count
    };

    let new_status: Status;

    if tally.total < threshold {
        new_status = Status::Expired;
    }
    else if tally.veto >= veto_threshold {
        new_status = Status::Vetoed{slashed_amount: Uint128::zero()};
    }
    else if tally.yes < yes_threshold {
        new_status = Status::Rejected;
    }
    else {
        new_status = Status::Success;
    }

    return new_status;
}

pub fn try_update<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    proposal: Uint128
) -> StdResult<HandleResponse> {
    let mut history = Proposal::status_history(&deps.storage, &proposal)?;
    let status = Proposal::status(&deps.storage, &proposal)?;
    let new_status: Status;

    let assembly = Proposal::assembly(&deps.storage, &proposal)?;
    let profile = Assembly::data(&deps.storage, &assembly)?.profile;

    let mut messages = vec![];

    match status {
        Status::AssemblyVote { votes, start, end } => {
            if end > env.block.time {
                return Err(StdError::unauthorized())
            }

            // Total power is equal to the total amount of assembly members
            let total_power = Uint128(Assembly::data(&deps.storage, &assembly)?.members.len().into());

            // Try to load, if not then assume it was updated after proposal creation but before section end
            let mut vote_conclusion: Status;
            if let Some(settings) = Profile::assembly_voting(&deps.storage, &profile)? {
                vote_conclusion = validate_votes(votes, total_power, settings);
            }
            else {
                vote_conclusion = Status::Success
            }

            if let Status::Vetoed{..} = vote_conclusion {
                // Cant veto an assembly vote
                vote_conclusion = Status::Rejected;
            }

            // Try to load the next steps, if all are none then pass
            if let Status::Success = vote_conclusion {
                if let Some(setting) = Profile::funding(&deps.storage, &profile)? {
                    vote_conclusion = Status::Funding {
                        amount: Uint128::zero(),
                        start: env.block.time,
                        end: env.block.time + setting.deadline
                    }
                }
                else if let Some(setting) = Profile::public_voting(&deps.storage, &profile)? {
                    vote_conclusion = Status::Voting {
                        votes: Vote::default(),
                        start: env.block.time,
                        end: env.block.time + setting.deadline
                    }
                }
                else {
                    vote_conclusion = Status::Passed {
                        start: env.block.time,
                        end: env.block.time + Profile::data(&deps.storage, &profile)?.cancel_deadline
                    }
                }
            }

            new_status = vote_conclusion;
        }
        Status::Funding { amount, start, end } => {
            // This helps combat the possibility of the profile changing
            // before another proposal is finished
            if let Some(setting) = Profile::funding(&deps.storage, &profile)? {
                // Check if deadline or funding limit reached
                if amount < setting.required && end > env.block.time {
                    return Err(StdError::unauthorized())
                }
                else if amount < setting.required {
                    new_status = Status::Expired
                }
            }

            if new_status != Status::Expired {
                if let Some(setting) = Profile::public_voting(&deps.storage, &profile)? {
                    new_status = Status::Voting {
                        votes: Vote::default(),
                        start: env.block.time,
                        end: env.block.time + setting.deadline
                    }
                }
                else {
                    new_status = Status::Passed {
                        start: env.block.time,
                        end: env.block.time + Profile::data(&deps.storage, &profile)?.cancel_deadline
                    }
                }
            }

        }
        Status::Voting { votes, start, end } => {
            if end > env.block.time {
                return Err(StdError::unauthorized())
            }

            let config = Config::load(&deps.storage)?;
            let query: shd_staking::QueryAnswer = shd_staking::QueryMsg::TotalStaked {}
                .query(
                    &deps.querier,
                    config.vote_token.unwrap().code_hash,
                    config.vote_token.unwrap().address
                )?.into();

            // Get total staking power
            let total_power = match query {
                // TODO: fix when uint update is merged
                shd_staking::QueryAnswer::TotalStaked { shares, tokens } => tokens.into(),
                _ => return Err(StdError::generic_err("Wrong query returned"))
            };

            let mut vote_conclusion: Status;

            if let Some(settings) = Profile::public_voting(&deps.storage, &profile)? {
                vote_conclusion = validate_votes(votes, total_power, settings);
            }
            else {
                vote_conclusion = Status::Success
            }

            if let Status::Vetoed {..} = vote_conclusion {
                // Send the funding amount to the treasury
                if let Some(profile) = Profile::funding(&deps.storage, &profile)? {
                    // Look for the history and find funding
                    for s in history {
                        // Check if it has funding history
                        if let Status::Funding{ amount, ..} = s {
                            let send_amount = amount.multiply_ratio(Uint128(100000), profile.veto_deposit_loss.clone());
                            if send_amount != Uint128::zero() {
                                let config = Config::load(&deps.storage)?;
                                // Update slash amount
                                vote_conclusion = Status::Vetoed { slashed_amount: send_amount };
                                messages.push(send_msg(
                                    config.treasury,
                                    cosmwasm_std::Uint128(send_amount.u128()),
                                    None, None, None, 1,
                                    config.funding_token.unwrap().code_hash,
                                    config.funding_token.unwrap().address
                                )?);
                            }
                            break;
                        }
                    }
                }
            }
            else if let Status::Success = new_status {
                new_status = Status::Passed {
                    start: env.block.time,
                    end: env.block.time + Profile::data(&deps.storage, &profile)?.cancel_deadline
                }
            }

            new_status = vote_conclusion;
        }
        _ => Err(StdError::generic_err("Cants update"))
    }

    // Add old status to history
    history.push(status.clone());
    Proposal::save_status_history(&mut deps.storage, &proposal, history)?;
    // Save new status
    Proposal::save_status(&mut deps.storage, &proposal, new_status.clone())?;

    Ok(HandleResponse {
        messages,
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
    // Check if sent token is the funding token
    let funding_token: Contract;
    if let Some(token) = Config::load(&deps.storage)?.funding_token {
        funding_token = token.clone();
        if env.message.sender != token.address {
            return Err(StdError::generic_err("Must be the set funding token"))
        }
    }
    else {
        return Err(StdError::generic_err("Funding token not set"))
    }

    // Check if msg contains the proposal information
    let proposal: Uint128;
    if let Some(msg) = msg {
        proposal = from_binary(&msg)?;
    }
    else {
        return Err(StdError::generic_err("Msg must be set"))
    }

    // Check if proposal is in funding stage
    // TODO: return overspent amount
    let mut new_fund = amount;
    let mut return_amount = Uint128::zero();
    let status = Proposal::status(&deps.storage, &proposal)?;
    if let Status::Funding{ amount, start, end } = status {
        // Check if proposal funding stage is set or funding limit already set
        if env.block.time >= end {
            return Err(StdError::generic_err("Funding time limit reached"))
        }


        let assembly = &Proposal::assembly(&deps.storage, &proposal)?;
        let profile = &Assembly::data(&deps.storage, assembly)?.profile;
        if let Some(funding_profile) = Profile::funding(&deps.storage, &profile)? {
            if funding_profile.required == amount {
                return Err(StdError::generic_err("Already funded"))
            }

            new_fund += amount;

            if funding_profile.required < new_fund {
                return_amount = new_fund.checked_sub(funding_profile.required)?;
                new_fund = funding_profile.required;
            }
        }
        else {
            return Err(StdError::generic_err("Funding profile setting was removed"))
        }

        // Store the funder information and update the current funding data
        Proposal::save_status(&mut deps.storage, &proposal, Status::Funding {
            amount: new_fund,
            start,
            end
        })?;
    }
    else {
        return Err(StdError::generic_err("Not in funding status"))
    }

    let mut messages = vec![];
    if return_amount != Uint128::zero() {
        // TODO: should the recipient be sender or from
        messages.push(send_msg(
            from,
            return_amount.into(),
            None,
            None,
            None,
            256,
            funding_token.code_hash,
            funding_token.address
        )?);
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::Receive {
            status: ResponseStatus::Success,
        })?),
    })
}