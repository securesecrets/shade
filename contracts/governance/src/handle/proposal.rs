use crate::handle::assembly::try_assembly_proposal;
use shade_protocol::c_std::{MessageInfo, Uint128};
use shade_protocol::c_std::{
    from_binary,
    to_binary,
    Api,
    Binary,
    Coin,
    Env,
    DepsMut,
    Response,
    Addr,
    Querier,
    StdError,
    StdResult,
    Storage,
    WasmMsg,
};
use shade_protocol::snip20::helpers::send_msg;
use shade_protocol::{
    contract_interfaces::{
        governance::{
            assembly::Assembly,
            contract::AllowedContract,
            profile::{Count, Profile, VoteProfile},
            proposal::{Funding, Proposal, ProposalMsg, Status},
            vote::{ReceiveBalanceMsg, TalliedVotes, Vote},
            Config,
            HandleAnswer,
            ExecuteMsg::Receive,
        },
        staking::snip20_staking,
    },
    utils::{
        asset::Contract,
        generic_response::ResponseStatus,
        storage::default::SingletonStorage,
    },
};
use shade_protocol::utils::Query;

// Initializes a proposal on the public assembly with the blank command
pub fn try_proposal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    title: String,
    metadata: String,
    contract: Option<Uint128>,
    msg: Option<String>,
    coins: Option<Vec<Coin>>,
) -> StdResult<Response> {
    let msgs: Option<Vec<ProposalMsg>>;

    if contract.is_some() && msg.is_some() {
        msgs = Some(vec![ProposalMsg {
            target: contract.unwrap(),
            assembly_msg: Uint128::zero(),
            msg: to_binary(&msg.unwrap())?,
            send: match coins {
                None => vec![],
                Some(c) => c,
            },
        }]);
    } else {
        msgs = None;
    }

    try_assembly_proposal(deps, env, info, Uint128::zero(), title, metadata, msgs)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::Proposal {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_trigger(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal: Uint128,
) -> StdResult<Response> {
    let mut messages = vec![];
    let status = Proposal::status(deps.storage, &proposal)?;
    if let Status::Passed { .. } = status {
        let mut history = Proposal::status_history(deps.storage, &proposal)?;
        history.push(status);
        Proposal::save_status_history(deps.storage, &proposal, history)?;
        Proposal::save_status(deps.storage, &proposal, Status::Success)?;

        // Trigger the msg
        let proposal_msg = Proposal::msg(deps.storage, &proposal)?;
        if let Some(prop_msgs) = proposal_msg {
            for prop_msg in prop_msgs.iter() {
                let contract = AllowedContract::data(deps.storage, &prop_msg.target)?.contract;
                messages.push(
                    WasmMsg::Execute {
                        contract_addr: contract.address.into(),
                        code_hash: contract.code_hash,
                        msg: prop_msg.msg.clone(),
                        funds: prop_msg.send.clone()
                    }
                    .into(),
                );
            }
        }
    } else {
        return Err(StdError::generic_err("unauthorized"));
    }

    Ok(Response::new().set_data(to_binary(&HandleAnswer::Trigger {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_cancel(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal: Uint128,
) -> StdResult<Response> {
    // Check if passed, and check if current time > cancel time
    let status = Proposal::status(deps.storage, &proposal)?;
    if let Status::Passed { start, end } = status {
        if env.block.time.seconds() < end {
            return Err(StdError::generic_err("unauthorized"));
        }
        let mut history = Proposal::status_history(deps.storage, &proposal)?;
        history.push(status);
        Proposal::save_status_history(deps.storage, &proposal, history)?;
        Proposal::save_status(deps.storage, &proposal, Status::Canceled)?;
    } else {
        return Err(StdError::generic_err("unauthorized"));
    }

    Ok(Response::new().set_data(to_binary(&HandleAnswer::Cancel {
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
    info: MessageInfo,
    proposal: Uint128,
) -> StdResult<Response> {
    let mut history = Proposal::status_history(deps.storage, &proposal)?;
    let status = Proposal::status(deps.storage, &proposal)?;
    let mut new_status: Status;

    let assembly = Proposal::assembly(deps.storage, &proposal)?;
    let profile = Assembly::data(deps.storage, &assembly)?.profile;

    let mut messages = vec![];

    match status.clone() {
        Status::AssemblyVote { start, end } => {
            if end > env.block.time.seconds() {
                return Err(StdError::generic_err("unauthorized"));
            }

            let votes = Proposal::assembly_votes(deps.storage, &proposal)?;

            // Total power is equal to the total amount of assembly members
            let total_power =
                Uint128::new(Assembly::data(deps.storage, &assembly)?.members.len() as u128);

            // Try to load, if not then assume it was updated after proposal creation but before section end
            let mut vote_conclusion: Status;
            if let Some(settings) = Profile::assembly_voting(deps.storage, &profile)? {
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
                if let Some(setting) = Profile::funding(deps.storage, &profile)? {
                    vote_conclusion = Status::Funding {
                        amount: Uint128::zero(),
                        start: env.block.time.seconds(),
                        end: env.block.time.seconds() + setting.deadline,
                    }
                } else if let Some(setting) = Profile::public_voting(deps.storage, &profile)? {
                    vote_conclusion = Status::Voting {
                        start: env.block.time.seconds(),
                        end: env.block.time.seconds() + setting.deadline,
                    }
                } else {
                    vote_conclusion = Status::Passed {
                        start: env.block.time.seconds(),
                        end: env.block.time.seconds()
                            + Profile::data(deps.storage, &profile)?.cancel_deadline,
                    }
                }
            }

            new_status = vote_conclusion;
        }
        Status::Funding { amount, start, end } => {
            // This helps combat the possibility of the profile changing
            // before another proposal is finished
            if let Some(setting) = Profile::funding(deps.storage, &profile)? {
                // Check if deadline or funding limit reached
                if amount >= setting.required {
                    new_status = Status::Passed {
                        start: env.block.time.seconds(),
                        end: env.block.time.seconds()
                            + Profile::data(deps.storage, &profile)?.cancel_deadline,
                    }
                } else if end > env.block.time.seconds() {
                    return Err(StdError::generic_err("unauthorized"));
                } else {
                    new_status = Status::Expired;
                }
            } else {
                new_status = Status::Passed {
                    start: env.block.time.seconds(),
                    end: env.block.time.seconds() + Profile::data(deps.storage, &profile)?.cancel_deadline,
                }
            }

            if let Status::Passed { .. } = new_status {
                if let Some(setting) = Profile::public_voting(deps.storage, &profile)? {
                    new_status = Status::Voting {
                        start: env.block.time.seconds(),
                        end: env.block.time.seconds() + setting.deadline,
                    }
                }
            }
        }
        Status::Voting { start, end } => {
            if end > env.block.time.seconds() {
                return Err(StdError::generic_err("unauthorized"));
            }

            let config = Config::load(deps.storage)?;
            let votes = Proposal::public_votes(deps.storage, &proposal)?;

            let query: snip20_staking::QueryAnswer = snip20_staking::QueryMsg::TotalStaked {}
                .query(
                    &deps.querier,
                    &config.vote_token.unwrap(),
                )?;

            // Get total staking power
            let total_power = match query {
                // TODO: fix when uint update is merged
                snip20_staking::QueryAnswer::TotalStaked { shares, tokens } => tokens.into(),
                _ => return Err(StdError::generic_err("Wrong query returned")),
            };

            let mut vote_conclusion: Status;

            if let Some(settings) = Profile::public_voting(deps.storage, &profile)? {
                vote_conclusion = validate_votes(votes, total_power, settings);
            } else {
                vote_conclusion = Status::Success
            }

            if let Status::Vetoed { .. } = vote_conclusion {
                // Send the funding amount to the treasury
                if let Some(profile) = Profile::funding(deps.storage, &profile)? {
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
                                    config.treasury,
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
                    end: env.block.time.seconds() + Profile::data(deps.storage, &profile)?.cancel_deadline,
                }
            }

            new_status = vote_conclusion;
        }
        _ => return Err(StdError::generic_err("Cant update")),
    }

    // Add old status to history
    history.push(status);
    Proposal::save_status_history(deps.storage, &proposal, history)?;
    // Save new status
    Proposal::save_status(deps.storage, &proposal, new_status.clone())?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::Update {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_receive(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Addr,
    from: Addr,
    amount: Uint128,
    msg: Option<Binary>,
    memo: Option<String>,
) -> StdResult<Response> {
    // Check if sent token is the funding token
    let funding_token: Contract;
    if let Some(token) = Config::load(deps.storage)?.funding_token {
        funding_token = token.clone();
        if info.sender != token.address {
            return Err(StdError::generic_err("Must be the set funding token"));
        }
    } else {
        return Err(StdError::generic_err("Funding token not set"));
    }

    // Check if msg contains the proposal information
    let proposal: Uint128;
    if let Some(msg) = msg {
        proposal = from_binary(&msg)?;
    } else {
        return Err(StdError::generic_err("Msg must be set"));
    }

    // Check if proposal is in funding stage
    let mut return_amount = Uint128::zero();

    let status = Proposal::status(deps.storage, &proposal)?;
    if let Status::Funding {
        amount: funded,
        start,
        end,
    } = status
    {
        // Check if proposal funding stage is set or funding limit already set
        if env.block.time.seconds() >= end {
            return Err(StdError::generic_err("Funding time limit reached"));
        }

        let mut new_fund = amount + funded;

        let assembly = &Proposal::assembly(deps.storage, &proposal)?;
        let profile = &Assembly::data(deps.storage, assembly)?.profile;
        if let Some(funding_profile) = Profile::funding(deps.storage, &profile)? {
            if funding_profile.required == funded {
                return Err(StdError::generic_err("Already funded"));
            }

            if funding_profile.required < new_fund {
                return_amount = new_fund.checked_sub(funding_profile.required)?;
                new_fund = funding_profile.required;
            }
        } else {
            return Err(StdError::generic_err("Funding profile setting was removed"));
        }

        // Store the funder information and update the current funding data
        Proposal::save_status(deps.storage, &proposal, Status::Funding {
            amount: new_fund,
            start,
            end,
        })?;

        // Either add or update funder
        let mut funder_amount = amount.checked_sub(return_amount)?;
        let mut funders = Proposal::funders(deps.storage, &proposal)?;
        if funders.contains(&from) {
            funder_amount += Proposal::funding(deps.storage, &proposal, &from)?.amount;
        } else {
            funders.push(from.clone());
            Proposal::save_funders(deps.storage, &proposal, funders)?;
        }
        Proposal::save_funding(deps.storage, &proposal, &from, Funding {
            amount: funder_amount,
            claimed: false,
        })?;
    } else {
        return Err(StdError::generic_err("Not in funding status"));
    }

    let mut messages = vec![];
    if return_amount != Uint128::zero() {
        messages.push(send_msg(
            from,
            return_amount.into(),
            None,
            None,
            None,
            &funding_token
        )?);
    }

    Ok(Response::new().set_data(to_binary(&HandleAnswer::Receive {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_claim_funding(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: Uint128,
) -> StdResult<Response> {
    let reduction = match Proposal::status(deps.storage, &id)? {
        Status::AssemblyVote { .. } | Status::Funding { .. } | Status::Voting { .. } => {
            return Err(StdError::generic_err("Cannot claim funding"));
        }
        Status::Vetoed { slash_percent } => slash_percent,
        _ => Uint128::zero(),
    };

    let funding = Proposal::funding(deps.storage, &id, &info.sender)?;

    if funding.claimed {
        return Err(StdError::generic_err("Funding already claimed"));
    }

    let return_amount = funding.amount.checked_sub(
        funding
            .amount
            .multiply_ratio(reduction, Uint128::new(10000)),
    )?;

    if return_amount == Uint128::zero() {
        return Err(StdError::generic_err("Nothing to claim"));
    }

    let funding_token = match Config::load(deps.storage)?.funding_token {
        None => return Err(StdError::generic_err("No funding token set")),
        Some(token) => token,
    };

    Ok(Response::new()
        .add_message(send_msg(
            info.sender,
            return_amount.into(),
            None,
            None,
            None,
            &funding_token
        )?)
        .set_data(to_binary(&HandleAnswer::ClaimFunding {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_receive_balance(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    sender: Addr,
    msg: Option<Binary>,
    balance: Uint128,
    memo: Option<String>,
) -> StdResult<Response> {
    if let Some(token) = Config::load(deps.storage)?.vote_token {
        if info.sender != token.address {
            return Err(StdError::generic_err("Must be the set voting token"));
        }
    } else {
        return Err(StdError::generic_err("Voting token not set"));
    }

    let vote: Vote;
    let proposal: Uint128;
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
            return Err(StdError::generic_err(
                "Total voting is greater than available balance",
            ));
        }
    } else {
        return Err(StdError::generic_err("Msg not set"));
    }

    // Check if proposal in assembly voting
    if let Status::Voting { end, .. } = Proposal::status(deps.storage, &proposal)? {
        if end <= env.block.time.seconds() {
            return Err(StdError::generic_err("Voting time has been reached"));
        }
    } else {
        return Err(StdError::generic_err("Not in public vote phase"));
    }

    let mut tally = Proposal::public_votes(deps.storage, &proposal)?;

    // Check if user voted
    if let Some(old_vote) = Proposal::public_vote(deps.storage, &proposal, &sender)? {
        tally = tally.checked_sub(&old_vote)?;
    }

    Proposal::save_public_vote(deps.storage, &proposal, &sender, &vote)?;
    Proposal::save_public_votes(deps.storage, &proposal, &tally.checked_add(&vote)?)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::ReceiveBalance {
            status: ResponseStatus::Success,
        })?))
}
