use crate::{
    proposal_state::{
        proposal_funding_batch_w,
        proposal_funding_deadline_r,
        proposal_funding_deadline_w,
        proposal_funding_r,
        proposal_funding_w,
        proposal_r,
        proposal_run_status_w,
        proposal_status_r,
        proposal_status_w,
        proposal_votes_r,
        proposal_votes_w,
        proposal_voting_deadline_r,
        proposal_voting_deadline_w,
        proposal_w,
        total_proposal_votes_r,
        total_proposal_votes_w,
        total_proposals_w,
    },
    state::{
        admin_commands_list_w,
        admin_commands_r,
        admin_commands_w,
        config_r,
        config_w,
        supported_contract_r,
        supported_contract_w,
        supported_contracts_list_w,
    },
};
use cosmwasm_std::{
    Api,
    Binary,
    CosmosMsg,
    Env,
    Extern,
    from_binary,
    HandleResponse,
    HumanAddr,
    Querier,
    StdError,
    StdResult,
    Storage,
    to_binary,
    Uint128,
    WasmMsg,
};
use secret_toolkit::snip20::{batch::SendAction, batch_send_msg, send_msg};
use shade_protocol::{
    governance::{
        ADMIN_COMMAND_VARIABLE,
        AdminCommand,
        GOVERNANCE_SELF,
        HandleAnswer,
        proposal::{Proposal, ProposalStatus},
        vote::VoteTally,
    },
};
use shade_protocol::utils::asset::Contract;
use shade_protocol::utils::generic_response::{
    ResponseStatus,
    ResponseStatus::{Failure, Success},
};

pub fn create_proposal<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    target_contract: String,
    proposal: Binary,
    description: String,
) -> StdResult<Uint128> {
    // Check that the target contract is neither the governance or a supported contract
    if supported_contract_r(&deps.storage)
        .may_load(target_contract.as_bytes())?
        .is_none()
        && target_contract != *GOVERNANCE_SELF
    {
        return Err(StdError::NotFound {
            kind: "contract is not found".to_string(),
            backtrace: None,
        });
    }

    // Create new proposal ID
    let proposal_id = total_proposals_w(&mut deps.storage).update(|mut id| {
        id += Uint128(1);
        Ok(id)
    })?;

    // Create proposal
    let proposal = Proposal {
        id: proposal_id,
        target: target_contract,
        msg: proposal,
        description,
    };

    let config = config_r(&deps.storage).load()?;

    // Store the proposal
    proposal_w(&mut deps.storage).save(proposal_id.to_string().as_bytes(), &proposal)?;
    // Initialize deadline
    proposal_funding_deadline_w(&mut deps.storage).save(
        proposal_id.to_string().as_bytes(),
        &(env.block.time + config.funding_deadline),
    )?;
    proposal_status_w(&mut deps.storage)
        .save(proposal_id.to_string().as_bytes(), &ProposalStatus::Funding)?;

    // Initialize total funding
    proposal_funding_w(&mut deps.storage)
        .save(proposal_id.to_string().as_bytes(), &Uint128::zero())?;
    // Initialize the funding batch
    proposal_funding_batch_w(&mut deps.storage)
        .save(proposal_id.to_string().as_bytes(), &vec![])?;

    // Create proposal votes
    total_proposal_votes_w(&mut deps.storage).save(
        proposal_id.to_string().as_bytes(),
        &VoteTally {
            yes: Uint128::zero(),
            no: Uint128::zero(),
            abstain: Uint128::zero(),
        },
    )?;

    Ok(proposal_id)
}

pub fn try_fund_proposal<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    sender: HumanAddr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<HandleResponse> {
    let proposal_id: Uint128 =
        from_binary(&msg.ok_or_else(|| StdError::not_found("Proposal ID in msg"))?)?;

    // Check if proposal is in funding
    let status = proposal_status_r(&deps.storage)
        .may_load(proposal_id.to_string().as_bytes())?
        .ok_or_else(|| StdError::not_found("Proposal"))?;
    if status != ProposalStatus::Funding {
        return Err(StdError::unauthorized());
    }

    let mut total = proposal_funding_r(&deps.storage).load(proposal_id.to_string().as_bytes())?;

    let config = config_r(&deps.storage).load()?;
    let mut messages = vec![];

    // Check if deadline is reached
    if env.block.time
        >= proposal_funding_deadline_r(&deps.storage).load(proposal_id.to_string().as_bytes())?
    {
        proposal_status_w(&mut deps.storage)
            .save(proposal_id.to_string().as_bytes(), &ProposalStatus::Expired)?;

        // Send back amount
        messages.push(send_msg(
            sender,
            amount,
            None,
            None,
            None,
            1,
            config.funding_token.code_hash.clone(),
            config.funding_token.address,
        )?);

        // TODO: send total over to treasury

        return Ok(HandleResponse {
            messages,
            log: vec![],
            data: Some(to_binary(&HandleAnswer::FundProposal {
                status: Failure,
                total_funding: total,
            })?),
        });
    }

    // Sum amount
    total += amount;

    let mut adjusted_amount = amount;

    // return the excess
    if total > config.funding_amount {
        let excess = (total - config.funding_amount)?;
        adjusted_amount = (adjusted_amount - excess)?;
        // Set total to max
        total = config.funding_amount;

        messages.push(send_msg(
            sender.clone(),
            excess,
            None,
            None,
            None,
            1,
            config.funding_token.code_hash.clone(),
            config.funding_token.address.clone(),
        )?);
    }

    // Update list of people that funded
    let amounts = proposal_funding_batch_w(&mut deps.storage).update(
        proposal_id.to_string().as_bytes(),
        |amounts| {
            if let Some(mut amounts) = amounts {
                amounts.push(SendAction {
                    recipient: sender.clone(),
                    recipient_code_hash: None,
                    amount: adjusted_amount,
                    msg: None,
                    memo: None,
                });

                return Ok(amounts);
            }

            Err(StdError::not_found("Funding batch"))
        },
    )?;

    // Update proposal status
    if total == config.funding_amount {
        // Update proposal status
        proposal_status_w(&mut deps.storage)
            .save(proposal_id.to_string().as_bytes(), &ProposalStatus::Voting)?;
        // Set vote deadline
        proposal_voting_deadline_w(&mut deps.storage).save(
            proposal_id.to_string().as_bytes(),
            &(env.block.time + config.voting_deadline),
        )?;

        // Send back all of the invested prop amount
        messages.push(batch_send_msg(
            amounts,
            None,
            1,
            config.funding_token.code_hash,
            config.funding_token.address,
        )?)
    }

    proposal_funding_w(&mut deps.storage).save(proposal_id.to_string().as_bytes(), &total)?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::FundProposal {
            status: Success,
            total_funding: total,
        })?),
    })
}

pub fn try_trigger_proposal<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    proposal_id: Uint128,
) -> StdResult<HandleResponse> {
    // Get proposal
    let proposal = proposal_r(&deps.storage).load(proposal_id.to_string().as_bytes())?;
    let run_status: ResponseStatus;
    let mut vote_status =
        proposal_status_r(&deps.storage).load(proposal_id.to_string().as_bytes())?;

    // Check if proposal has run
    // TODO: This might not be needed
    // if proposal_run_status_r(&deps.storage).may_load(proposal_id.to_string().as_bytes())?.is_some() {
    //     return Err(StdError::generic_err("Proposal has already been executed"))
    // }

    // Change proposal behavior according to stake availability
    let config = config_r(&deps.storage).load()?;
    vote_status = match config.staker {
        Some(_) => {
            // When staking is enabled funding is required
            if vote_status != ProposalStatus::Voting {
                return Err(StdError::unauthorized());
            }

            let total_votes =
                total_proposal_votes_r(&deps.storage).load(proposal_id.to_string().as_bytes())?;

            // Check if proposal can be run
            let voting_deadline = proposal_voting_deadline_r(&deps.storage)
                .may_load(proposal_id.to_string().as_bytes())?
                .ok_or_else(|| StdError::generic_err("No deadline set"))?;
            if voting_deadline > env.block.time {
                Err(StdError::unauthorized())
            } else if total_votes.yes + total_votes.no + total_votes.abstain < config.minimum_votes
            {
                Ok(ProposalStatus::Expired)
            } else if total_votes.yes > total_votes.no {
                Ok(ProposalStatus::Passed)
            } else {
                Ok(ProposalStatus::Rejected)
            }
        }
        None => {
            // Check if user is an admin in order to trigger the proposal
            if config.admin == env.message.sender {
                Ok(ProposalStatus::Passed)
            } else {
                Err(StdError::unauthorized())
            }
        }
    }?;

    let mut messages: Vec<CosmosMsg> = vec![];

    let target: Option<Contract>;
    if proposal.target == GOVERNANCE_SELF {
        target = Some(Contract {
            address: env.contract.address.clone(),
            code_hash: env.contract_code_hash.clone(),
        })
    } else {
        target = supported_contract_r(&deps.storage).may_load(proposal.target.as_bytes())?;
    }

    // Check if proposal passed or has a valid target contract
    if vote_status != ProposalStatus::Passed {
        run_status = Failure;
    } else if let Some(target) = target {
        run_status = match try_execute_msg(target, proposal.msg) {
            Ok(msg) => {
                messages.push(msg);
                Success
            }
            Err(_) => Failure,
        };
    } else {
        run_status = Failure;
    }

    // Overwrite
    proposal_run_status_w(&mut deps.storage)
        .save(proposal_id.to_string().as_bytes(), &run_status)?;
    proposal_status_w(&mut deps.storage).save(proposal_id.to_string().as_bytes(), &vote_status)?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::TriggerProposal {
            status: run_status,
        })?),
    })
}

pub fn try_execute_msg(contract: Contract, msg: Binary) -> StdResult<CosmosMsg> {
    let execute = WasmMsg::Execute {
        msg,
        contract_addr: contract.address,
        callback_code_hash: contract.code_hash,
        send: vec![],
    };
    Ok(execute.into())
}

pub fn try_vote<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    voter: HumanAddr,
    proposal_id: Uint128,
    votes: VoteTally,
) -> StdResult<HandleResponse> {
    // Check that sender is staking contract and staking is enabled
    let config = config_r(&deps.storage).load()?;
    if config.staker.is_none() || config.staker.unwrap().address != env.message.sender {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Check that proposal is votable
    let vote_status = proposal_status_r(&deps.storage)
        .may_load(proposal_id.to_string().as_bytes())?
        .ok_or_else(|| StdError::not_found("Proposal"))?;
    let voting_deadline = proposal_voting_deadline_r(&deps.storage)
        .may_load(proposal_id.to_string().as_bytes())?
        .ok_or_else(|| StdError::generic_err("No deadline set"))?;

    if vote_status != ProposalStatus::Voting || voting_deadline <= env.block.time {
        return Err(StdError::unauthorized());
    }

    // Get proposal voting state
    let mut proposal_voting_state =
        total_proposal_votes_r(&deps.storage).load(proposal_id.to_string().as_bytes())?;

    // Check if user has already voted
    match proposal_votes_r(&deps.storage, proposal_id).may_load(voter.to_string().as_bytes())? {
        None => {}
        Some(old_votes) => {
            // Remove those votes from state
            proposal_voting_state.yes = (proposal_voting_state.yes - old_votes.yes)?;
            proposal_voting_state.no = (proposal_voting_state.no - old_votes.no)?;
            proposal_voting_state.abstain = (proposal_voting_state.abstain - old_votes.abstain)?;
        }
    }

    // Update state
    proposal_voting_state.yes += votes.yes;
    proposal_voting_state.no += votes.no;
    proposal_voting_state.abstain += votes.abstain;

    // Save staker info
    total_proposal_votes_w(&mut deps.storage)
        .save(proposal_id.to_string().as_bytes(), &proposal_voting_state)?;
    proposal_votes_w(&mut deps.storage, proposal_id).save(voter.to_string().as_bytes(), &votes)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::MakeVote { status: Success })?),
    })
}

pub fn try_trigger_admin_command<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    target: String,
    command: String,
    variables: Vec<String>,
    description: String,
) -> StdResult<HandleResponse> {
    // Check that user is admin
    if config_r(&deps.storage).load()?.admin != env.message.sender {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // First validate that the contract exists
    let target_contract = match supported_contract_r(&deps.storage).may_load(target.as_bytes())? {
        None => {
            return Err(StdError::NotFound {
                kind: "Contract not found".to_string(),
                backtrace: None,
            });
        }
        Some(contract) => contract,
    };

    // Check that command exists
    let admin_command = match admin_commands_r(&deps.storage).may_load(command.as_bytes())? {
        None => {
            return Err(StdError::NotFound {
                kind: "Command not found".to_string(),
                backtrace: None,
            });
        }
        Some(admin_c) => admin_c,
    };

    // With command validate that number of variables is equal
    if admin_command.total_arguments != variables.len() as u16 {
        return Err(StdError::GenericErr {
            msg: "Variable number doesnt match up".to_string(),
            backtrace: None,
        });
    }

    // Replace variable spaces
    let mut finished_command = admin_command.msg;
    for item in variables.iter() {
        finished_command = finished_command.replacen(ADMIN_COMMAND_VARIABLE, item, 1);
    }

    let mut messages = vec![];

    // Create new proposal ID
    let proposal_id = total_proposals_w(&mut deps.storage).update(|mut id| {
        id += Uint128(1);
        Ok(id)
    })?;

    // Try to run
    let proposal = Proposal {
        id: proposal_id,
        target,
        msg: Binary::from(finished_command.as_bytes()),
        description,
    };

    // Store the proposal
    proposal_w(&mut deps.storage).save(proposal_id.to_string().as_bytes(), &proposal)?;
    proposal_funding_deadline_w(&mut deps.storage)
        .save(proposal_id.to_string().as_bytes(), &env.block.time)?;
    proposal_voting_deadline_w(&mut deps.storage)
        .save(proposal_id.to_string().as_bytes(), &env.block.time)?;
    proposal_status_w(&mut deps.storage).save(
        proposal_id.to_string().as_bytes(),
        &ProposalStatus::AdminRequested,
    )?;
    let run_status =
        match try_execute_msg(target_contract, Binary::from(finished_command.as_bytes())) {
            Ok(executed_msg) => {
                messages.push(executed_msg);
                Success
            }
            Err(_) => Failure,
        };
    proposal_run_status_w(&mut deps.storage)
        .save(proposal_id.to_string().as_bytes(), &run_status)?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some(to_binary(&HandleAnswer::TriggerAdminCommand {
            status: run_status,
            proposal_id,
        })?),
    })
}

/// SELF only interactions

pub fn try_create_proposal<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    target_contract: String,
    proposal: Binary,
    description: String,
) -> StdResult<HandleResponse> {
    let proposal_id = create_proposal(deps, env, target_contract, proposal, description)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::CreateProposal {
            status: Success,
            proposal_id,
        })?),
    })
}

#[allow(clippy::too_many_arguments)]
pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    admin: Option<HumanAddr>,
    staker: Option<Contract>,
    proposal_deadline: Option<u64>,
    funding_amount: Option<Uint128>,
    funding_deadline: Option<u64>,
    minimum_votes: Option<Uint128>,
) -> StdResult<HandleResponse> {
    // It has to be self
    if env.contract.address != env.message.sender {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    config_w(&mut deps.storage).update(|mut state| {
        if let Some(admin) = admin {
            state.admin = admin;
        }
        if staker.is_some() {
            state.staker = staker;
        }
        if let Some(proposal_deadline) = proposal_deadline {
            state.voting_deadline = proposal_deadline;
        }
        if let Some(funding_amount) = funding_amount {
            state.funding_amount = funding_amount;
        }
        if let Some(funding_deadline) = funding_deadline {
            state.funding_deadline = funding_deadline;
        }
        if let Some(minimum_votes) = minimum_votes {
            state.minimum_votes = minimum_votes;
        }

        Ok(state)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateConfig { status: Success })?),
    })
}

pub fn try_disable_staker<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: &Env,
) -> StdResult<HandleResponse> {
    config_w(&mut deps.storage).update(|mut state| {
        state.staker = None;
        Ok(state)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::DisableStaker { status: Success })?),
    })
}

pub fn try_add_supported_contract<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    name: String,
    contract: Contract,
) -> StdResult<HandleResponse> {
    // It has to be self
    if env.contract.address != env.message.sender {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Cannot be the same name as governance default
    if name == *GOVERNANCE_SELF {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Supported contract cannot exist
    if supported_contract_r(&deps.storage)
        .may_load(name.as_bytes())?
        .is_some()
    {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Save contract
    supported_contract_w(&mut deps.storage).save(name.as_bytes(), &contract)?;

    // Update command list
    supported_contracts_list_w(&mut deps.storage).update(|mut arr| {
        arr.push(name);
        Ok(arr)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddSupportedContract {
            status: Success,
        })?),
    })
}

pub fn try_remove_supported_contract<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    name: String,
) -> StdResult<HandleResponse> {
    // It has to be self
    if env.contract.address != env.message.sender {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Cannot be the same name as governance default
    if name == *GOVERNANCE_SELF {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Remove contract
    supported_contract_w(&mut deps.storage).remove(name.as_bytes());

    // Remove from array
    supported_contracts_list_w(&mut deps.storage).update(|mut arr| {
        arr.retain(|value| *value != name);
        Ok(arr)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RemoveSupportedContract {
            status: Success,
        })?),
    })
}

pub fn try_update_supported_contract<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    name: String,
    contract: Contract,
) -> StdResult<HandleResponse> {
    // It has to be self and cannot be the same name as governance default
    if env.contract.address != env.message.sender || name == *GOVERNANCE_SELF {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Replace contract
    supported_contract_w(&mut deps.storage).update(name.as_bytes(), |_state| Ok(contract))?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateSupportedContract {
            status: Success,
        })?),
    })
}

pub fn try_add_admin_command<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    name: String,
    proposal: String,
) -> StdResult<HandleResponse> {
    // It has to be self
    if env.contract.address != env.message.sender {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Admin command cannot exist
    if admin_commands_r(&deps.storage)
        .may_load(name.as_bytes())?
        .is_some()
    {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Save command
    admin_commands_w(&mut deps.storage).save(name.as_bytes(), &AdminCommand {
        msg: proposal.clone(),
        total_arguments: proposal.matches(ADMIN_COMMAND_VARIABLE).count() as u16,
    })?;

    // Update command list
    admin_commands_list_w(&mut deps.storage).update(|mut arr| {
        arr.push(name);
        Ok(arr)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddAdminCommand {
            status: Success,
        })?),
    })
}

pub fn try_remove_admin_command<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    name: String,
) -> StdResult<HandleResponse> {
    // It has to be self
    if env.contract.address != env.message.sender {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Remove command
    admin_commands_w(&mut deps.storage).remove(name.as_bytes());

    // Remove from array
    admin_commands_list_w(&mut deps.storage).update(|mut arr| {
        arr.retain(|value| *value != name);
        Ok(arr)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::RemoveAdminCommand {
            status: Success,
        })?),
    })
}

pub fn try_update_admin_command<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    name: String,
    proposal: String,
) -> StdResult<HandleResponse> {
    // It has to be self
    if env.contract.address != env.message.sender {
        return Err(StdError::Unauthorized { backtrace: None });
    }

    // Replace contract
    admin_commands_w(&mut deps.storage).update(name.as_bytes(), |_state| {
        Ok(AdminCommand {
            msg: proposal.clone(),
            total_arguments: proposal.matches(ADMIN_COMMAND_VARIABLE).count() as u16,
        })
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::UpdateAdminCommand {
            status: Success,
        })?),
    })
}
