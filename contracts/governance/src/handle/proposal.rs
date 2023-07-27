use crate::handle::assembly_state_valid;
use shade_protocol::{
    c_std::{
        from_binary,
        to_binary,
        Addr,
        Binary,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdResult,
        SubMsg,
        Uint128,
        WasmMsg,
    },
    contract_interfaces::{
        governance::{
            assembly::Assembly,
            contract::AllowedContract,
            profile::{Count, Profile, VoteProfile},
            proposal::{Funding, Proposal, Status},
            stored_id::UserID,
            vote::{ReceiveBalanceMsg, TalliedVotes, Vote},
            Config,
            ExecuteAnswer,
        },
        staking::snip20_staking,
    },
    governance::errors::Error,
    snip20::helpers::send_msg,
    utils::{asset::Contract, generic_response::ResponseStatus, storage::plus::ItemStorage, Query},
};

pub fn try_trigger(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    proposal: u32,
) -> StdResult<Response> {
    let mut messages = vec![];

    let status = Proposal::status(deps.storage, proposal)?;
    if let Status::Passed { .. } = status {
        let mut history = Proposal::status_history(deps.storage, proposal)?;
        history.push(status);
        Proposal::save_status_history(deps.storage, proposal, history)?;
        Proposal::save_status(deps.storage, proposal, Status::Success)?;

        // Trigger the msg
        let proposal_msg = Proposal::msg(deps.storage, proposal)?;
        if let Some(prop_msgs) = proposal_msg {
            for (_i, prop_msg) in prop_msgs.iter().enumerate() {
                let contract = AllowedContract::data(deps.storage, prop_msg.target)?.contract;
                let msg = WasmMsg::Execute {
                    contract_addr: contract.address.into(),
                    code_hash: contract.code_hash,
                    msg: prop_msg.msg.clone(),
                    funds: prop_msg.send.clone(),
                };
                // TODO: set to reply on error where ID is propID + 1
                // TODO: set proposal status to success
                messages.push(SubMsg::new(msg));
            }
        }
    } else {
        return Err(Error::not_passed(vec![]));
    }

    Ok(Response::new()
        .add_submessages(messages)
        .set_data(to_binary(&ExecuteAnswer::Trigger {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_cancel(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    proposal: u32,
) -> StdResult<Response> {
    // Check if passed, and check if current time > cancel time
    let status = Proposal::status(deps.storage, proposal)?;
    if let Status::Passed { start: _, end } = status {
        if env.block.time.seconds() < end {
            return Err(Error::cannot_cancel(vec![&end.to_string()]));
        }
        let mut history = Proposal::status_history(deps.storage, proposal)?;
        history.push(status);
        Proposal::save_status_history(deps.storage, proposal, history)?;
        Proposal::save_status(deps.storage, proposal, Status::Canceled)?;
    } else {
        return Err(Error::cannot_cancel(vec![&(-1).to_string()]));
    }

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Cancel {
        status: ResponseStatus::Success,
    })?))
}

fn validate_votes(votes: Vote, total_power: Uint128, settings: VoteProfile) -> Status {
    let tally = TalliedVotes::tally(votes);

    let threshold = match settings.threshold {
        Count::Percentage { percent } => total_power.multiply_ratio(percent, Uint128::new(10000)),
        Count::LiteralCount { count } => count,
    };

    let yes_threshold = match settings.yes_threshold {
        Count::Percentage { percent } => {
            (tally.yes + tally.no).multiply_ratio(percent, Uint128::new(10000))
        }
        Count::LiteralCount { count } => count,
    };

    let veto_threshold = match settings.veto_threshold {
        Count::Percentage { percent } => {
            (tally.yes + tally.no).multiply_ratio(percent, Uint128::new(10000))
        }
        Count::LiteralCount { count } => count,
    };

    let new_status: Status;

    if tally.total < threshold {
        new_status = Status::Expired;
    } else if tally.veto >= veto_threshold {
        new_status = Status::Vetoed {
            slash_percent: Uint128::zero(),
        };
    } else if tally.yes < yes_threshold {
        new_status = Status::Rejected;
    } else {
        new_status = Status::Success;
    }

    return new_status;
}

pub fn try_update(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    proposal: u32,
) -> StdResult<Response> {
    // TODO: see if this can get cleaned up

    let mut history = Proposal::status_history(deps.storage, proposal)?;
    let status = Proposal::status(deps.storage, proposal)?;
    let mut new_status: Status;

    let assembly = Proposal::assembly(deps.storage, proposal)?;
    let profile = Assembly::data(deps.storage, assembly)?.profile;

    // Halt all proposal updates
    assembly_state_valid(deps.storage, assembly)?;

    let mut messages = vec![];

    match status.clone() {
        Status::AssemblyVote { start: _, end } => {
            if end > env.block.time.seconds() {
                return Err(Error::cannot_update(vec!["AssemblyVote", &end.to_string()]));
            }

            let votes = Proposal::assembly_votes(deps.storage, proposal)?;

            // Total power is equal to the total amount of assembly members
            let total_power =
                Uint128::new(Assembly::data(deps.storage, assembly)?.members.len() as u128);

            // Try to load, if not then assume it was updated after proposal creation but before section end
            let mut vote_conclusion: Status;
            if let Some(settings) = Profile::assembly_voting(deps.storage, profile)? {
                vote_conclusion = validate_votes(votes, total_power, settings);
            } else {
                vote_conclusion = Status::Success
            }

            if let Status::Vetoed { .. } = vote_conclusion {
                // Cant veto an assembly vote
                vote_conclusion = Status::Rejected;
            }

            // Try to load the next steps, if all are none then pass
            if let Status::Success = vote_conclusion {
                if let Some(setting) = Profile::funding(deps.storage, profile)? {
                    vote_conclusion = Status::Funding {
                        amount: Uint128::zero(),
                        start: env.block.time.seconds(),
                        end: env.block.time.seconds() + setting.deadline,
                    }
                } else if let Some(setting) = Profile::public_voting(deps.storage, profile)? {
                    vote_conclusion = Status::Voting {
                        start: env.block.time.seconds(),
                        end: env.block.time.seconds() + setting.deadline,
                    }
                } else {
                    vote_conclusion = Status::Passed {
                        start: env.block.time.seconds(),
                        end: env.block.time.seconds()
                            + Profile::data(deps.storage, profile)?.cancel_deadline,
                    }
                }
            }

            new_status = vote_conclusion;
        }
        Status::Funding { amount, end, .. } => {
            // This helps combat the possibility of the profile changing
            // before another proposal is finished
            if let Some(setting) = Profile::funding(deps.storage, profile)? {
                // Check if deadline or funding limit reached
                if amount >= setting.required {
                    new_status = Status::Passed {
                        start: env.block.time.seconds(),
                        end: env.block.time.seconds()
                            + Profile::data(deps.storage, profile)?.cancel_deadline,
                    }
                } else if end > env.block.time.seconds() {
                    return Err(Error::cannot_update(vec!["Funding", &end.to_string()]));
                } else {
                    new_status = Status::Expired;
                }
            } else {
                new_status = Status::Passed {
                    start: env.block.time.seconds(),
                    end: env.block.time.seconds()
                        + Profile::data(deps.storage, profile)?.cancel_deadline,
                }
            }

            if let Status::Passed { .. } = new_status {
                if let Some(setting) = Profile::public_voting(deps.storage, profile)? {
                    new_status = Status::Voting {
                        start: env.block.time.seconds(),
                        end: env.block.time.seconds() + setting.deadline,
                    }
                }
            }
        }
        Status::Voting { start: _, end } => {
            if end > env.block.time.seconds() {
                return Err(Error::cannot_update(vec!["Voting", &end.to_string()]));
            }

            let config = Config::load(deps.storage)?;
            let votes = Proposal::public_votes(deps.storage, proposal)?;

            let query: snip20_staking::QueryAnswer = snip20_staking::QueryMsg::TotalStaked {}
                .query(&deps.querier, &config.vote_token.unwrap())?;

            // Get total staking power
            let total_power = match query {
                snip20_staking::QueryAnswer::TotalStaked { tokens, .. } => tokens.into(),
                _ => return Err(Error::unexpected_query_response(vec![])),
            };

            let mut vote_conclusion: Status;

            if let Some(settings) = Profile::public_voting(deps.storage, profile)? {
                vote_conclusion = validate_votes(votes, total_power, settings);
            } else {
                vote_conclusion = Status::Success
            }

            if let Status::Vetoed { .. } = vote_conclusion {
                // Send the funding amount to the treasury
                if let Some(profile) = Profile::funding(deps.storage, profile)? {
                    // Look for the history and find funding
                    for s in history.iter() {
                        // Check if it has funding history
                        if let Status::Funding { amount, .. } = s {
                            let loss = profile.veto_deposit_loss.clone();
                            vote_conclusion = Status::Vetoed {
                                slash_percent: loss,
                            };

                            let send_amount = amount.multiply_ratio(100000u128, loss);
                            if send_amount != Uint128::zero() {
                                let config = Config::load(deps.storage)?;
                                // Update slash amount
                                messages.push(send_msg(
                                    config.treasury.into(),
                                    Uint128::new(send_amount.u128()),
                                    None,
                                    None,
                                    None,
                                    &config.funding_token.unwrap(),
                                )?);
                            }
                            break;
                        }
                    }
                }
            } else if let Status::Success = vote_conclusion {
                vote_conclusion = Status::Passed {
                    start: env.block.time.seconds(),
                    end: env.block.time.seconds()
                        + Profile::data(deps.storage, profile)?.cancel_deadline,
                }
            }

            new_status = vote_conclusion;
        }
        _ => return Err(Error::state_update(vec![])),
    }

    // Add old status to history
    history.push(status);
    Proposal::save_status_history(deps.storage, proposal, history)?;
    // Save new status
    Proposal::save_status(deps.storage, proposal, new_status.clone())?;

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Update {
        status: ResponseStatus::Success,
    })?))
}

pub fn try_receive_funding(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _sender: Addr,
    from: Addr,
    amount: Uint128,
    msg: Option<Binary>,
    _memo: Option<String>,
) -> StdResult<Response> {
    // Check if sent token is the funding token
    let funding_token: Contract;
    if let Some(token) = Config::load(deps.storage)?.funding_token {
        funding_token = token.clone();
        if info.sender != token.address {
            return Err(Error::missing_funding_token(vec![]));
        }
    } else {
        return Err(Error::missing_funding_token(vec![]));
    }

    // Check if msg contains the proposal information
    let proposal: u32;
    if let Some(msg) = msg {
        proposal = from_binary(&msg)?;
    } else {
        return Err(Error::funding_msg_not_set(vec![]));
    }

    // Check if proposal is in funding stage
    let mut return_amount = Uint128::zero();

    let status = Proposal::status(deps.storage, proposal)?;
    if let Status::Funding {
        amount: funded,
        start,
        end,
    } = status
    {
        // Check if proposal funding stage is set or funding limit already set
        if env.block.time.seconds() >= end {
            return Err(Error::funding_limit_reached(vec![]));
        }

        let mut new_fund = amount + funded;

        let assembly = Proposal::assembly(deps.storage, proposal)?;

        // Validate that this action is possible
        assembly_state_valid(deps.storage, assembly)?;

        let profile = Assembly::data(deps.storage, assembly)?.profile;
        if let Some(funding_profile) = Profile::funding(deps.storage, profile)? {
            if funding_profile.required == funded {
                return Err(Error::completely_funded(vec![]));
            }

            if funding_profile.required < new_fund {
                return_amount = new_fund.checked_sub(funding_profile.required)?;
                new_fund = funding_profile.required;
            }
        } else {
            return Err(Error::no_funding_profile(vec![]));
        }

        // Store the funder information and update the current funding data
        Proposal::save_status(deps.storage, proposal, Status::Funding {
            amount: new_fund,
            start,
            end,
        })?;

        // Either add or update funder
        let mut funder_amount = amount.checked_sub(return_amount)?;
        let mut funders = Proposal::funders(deps.storage, proposal)?;
        if funders.contains(&from) {
            funder_amount += Proposal::funding(deps.storage, proposal, &from)?.amount;
        } else {
            funders.push(from.clone());
            Proposal::save_funders(deps.storage, proposal, funders)?;
        }
        Proposal::save_funding(deps.storage, proposal, &from, Funding {
            amount: funder_amount,
            claimed: false,
        })?;

        // Add funding info to cross search
        UserID::add_funding(deps.storage, from.clone(), proposal.clone())?;
    } else {
        return Err(Error::no_funding_state(vec![]));
    }

    let mut messages = vec![];
    if return_amount != Uint128::zero() {
        messages.push(send_msg(
            from.into(),
            return_amount.into(),
            None,
            None,
            None,
            &funding_token,
        )?);
    }

    Ok(Response::new().set_data(to_binary(&ExecuteAnswer::Receive {
        status: ResponseStatus::Success,
    })?))
}

pub fn try_claim_funding(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    id: u32,
) -> StdResult<Response> {
    let reduction = match Proposal::status(deps.storage, id)? {
        Status::AssemblyVote { .. } | Status::Funding { .. } | Status::Voting { .. } => {
            return Err(Error::funding_not_claimable(vec![]));
        }
        Status::Vetoed { slash_percent } => slash_percent,
        _ => Uint128::zero(),
    };

    let funding = Proposal::funding(deps.storage, id, &info.sender)?;

    if funding.claimed {
        return Err(Error::funding_claimed(vec![]));
    }

    let return_amount = funding.amount.checked_sub(
        funding
            .amount
            .multiply_ratio(reduction, Uint128::new(10000)),
    )?;

    if return_amount == Uint128::zero() {
        return Err(Error::funding_nothing(vec![]));
    }

    let funding_token = match Config::load(deps.storage)?.funding_token {
        None => return Err(Error::missing_funding_token(vec![])),
        Some(token) => token,
    };

    Ok(Response::new()
        .add_message(send_msg(
            info.sender.into(),
            return_amount.into(),
            None,
            None,
            None,
            &funding_token,
        )?)
        .set_data(to_binary(&ExecuteAnswer::ClaimFunding {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_receive_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Addr,
    msg: Option<Binary>,
    balance: Uint128,
    _memo: Option<String>,
) -> StdResult<Response> {
    if let Some(token) = Config::load(deps.storage)?.vote_token {
        if info.sender != token.address {
            return Err(Error::sender_funding(vec![]));
        }
    } else {
        return Err(Error::missing_funding_token(vec![]));
    }

    let vote: Vote;
    let proposal: u32;
    if let Some(msg) = msg {
        let decoded_msg: ReceiveBalanceMsg = from_binary(&msg)?;
        vote = decoded_msg.vote;
        proposal = decoded_msg.proposal;

        // Verify that total does not exceed balance
        let total_votes = vote.yes.checked_add(
            vote.no
                .checked_add(vote.abstain.checked_add(vote.no_with_veto)?)?,
        )?;

        if total_votes > balance {
            return Err(Error::voting_balance(vec![]));
        }
    } else {
        return Err(Error::voting_msg(vec![]));
    }

    // Check if proposal in assembly voting
    if let Status::Voting { end, .. } = Proposal::status(deps.storage, proposal)? {
        if end <= env.block.time.seconds() {
            return Err(Error::voting_time(vec![&end.to_string()]));
        }
    } else {
        return Err(Error::voting_not_state(vec![]));
    }

    let mut tally = Proposal::public_votes(deps.storage, proposal)?;

    // Check if user voted
    if let Some(old_vote) = Proposal::public_vote(deps.storage, proposal, &sender)? {
        tally = tally.checked_sub(&old_vote)?;
    }

    Proposal::save_public_vote(deps.storage, proposal, &sender, &vote)?;
    Proposal::save_public_votes(deps.storage, proposal, &tally.checked_add(&vote)?)?;
    UserID::add_vote(deps.storage, sender.clone(), proposal)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::ReceiveBalance {
            status: ResponseStatus::Success,
        })?),
    )
}
