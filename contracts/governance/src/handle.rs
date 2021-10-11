use cosmwasm_std::{to_binary, Api, Binary, Env, Extern, HandleResponse, Querier, StdError, StdResult, Storage, CosmosMsg, HumanAddr, Uint128, WasmMsg, Empty};
use crate::state::{supported_contract_r, config_r, total_proposals_w, proposal_w, config_w, supported_contract_w, proposal_r, admin_commands_r, admin_commands_w, admin_commands_list_w, supported_contracts_list_w, total_proposals_r, total_proposal_votes_w, proposal_votes_w, total_proposal_votes_r, proposal_votes_r};
use shade_protocol::{
    governance::{Proposal, ProposalStatus, HandleAnswer, GOVERNANCE_SELF, HandleMsg},
    generic_response::ResponseStatus,
    asset::Contract,
};
use shade_protocol::governance::ProposalStatus::{Accepted, Expired, Rejected};
use shade_protocol::generic_response::ResponseStatus::{Success, Failure};
use shade_protocol::governance::{AdminCommand, ADMIN_COMMAND_VARIABLE, Vote, UserVote, VoteTally};
use secret_toolkit::utils::HandleCallback;

pub fn create_proposal<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    target_contract: String,
    proposal: Binary,
    description: String,
) -> StdResult<Uint128> {

    // Check that the target contract is neither the governance or a supported contract
    if supported_contract_r(&deps.storage).may_load(target_contract.as_bytes())?.is_none() &&
        target_contract != GOVERNANCE_SELF.to_string(){
        return Err(StdError::NotFound {
            kind: "contract is not found".to_string(), backtrace: None })
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
        is_admin_command: false,
        due_date: env.block.time + config_r(&deps.storage).load()?.proposal_deadline,
        vote_status: ProposalStatus::InProgress,
        run_status: None
    };

    // Store the proposal
    proposal_w(&mut deps.storage).save(proposal_id.to_string().as_bytes(), &proposal)?;

    // Create proposal votes
    total_proposal_votes_w(&mut deps.storage).save(
        proposal_id.to_string().as_bytes(), &VoteTally{
        yes: Uint128(0),
        no: Uint128(0),
        abstain: Uint128(0)
    })?;

    Ok(proposal_id)
}

pub fn try_trigger_proposal<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    proposal_id: Uint128
) -> StdResult<HandleResponse> {

    // Get proposal
    let mut proposal = proposal_r(&deps.storage).load(proposal_id.to_string().as_bytes())?;

    // Check if proposal has run
    if proposal.run_status.is_some() {
        return Err(StdError::GenericErr {
            msg: "Proposal has already been executed".to_string(),
            backtrace: None })
    }

    // Check if proposal can be run
    if proposal.due_date > env.block.time {
        return Err(StdError::Unauthorized { backtrace: None })
    }

    let total_votes = total_proposal_votes_r(&deps.storage).load(
        proposal_id.to_string().as_bytes())?;

    let config = config_r(&deps.storage).load()?;
    if total_votes.yes + total_votes.no + total_votes.abstain < config.minimum_votes {
        proposal.vote_status = Expired;
    } else if total_votes.yes > total_votes.no {
        proposal.vote_status = Accepted;
    } else {
        proposal.vote_status = Rejected;
    }

    let mut messages: Vec<CosmosMsg> = vec![];

    let target: Option<Contract>;
    if proposal.target == GOVERNANCE_SELF {
        target = Some(Contract {
            address: env.contract.address.clone(),
            code_hash: env.contract_code_hash.clone(),
        })
    } else {
        target = supported_contract_r(&deps.storage).may_load(
            proposal.target.as_bytes())?;
    }

    // Check if proposal passed or has a valid target contract
    if proposal.vote_status != Accepted || target.is_none() {
        proposal.run_status = Some(Failure);
    }
    else {
        match try_execute_msg(target.unwrap(), proposal.msg.clone()) {
            Ok(msg) => {
                proposal.run_status = Some(Success);
                messages.push(msg);
            }
            Err(_) => proposal.run_status = Some(Failure),
        };
    }

    // Overwrite
    proposal_w(&mut deps.storage).save(proposal_id.to_string().as_bytes(), &proposal)?;

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some( to_binary( &HandleAnswer::TriggerProposal {
            status: proposal.run_status.unwrap(),
        })?),
    })
}

pub fn try_execute_msg(
    contract: Contract,
    msg: Binary,
) -> StdResult<CosmosMsg> {
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

    // Check that sender is staking contract
    let config = config_r(&deps.storage).load()?;
    if config.staker.address != env.message.sender {
        return Err(StdError::Unauthorized { backtrace: None })
    }

    // Check that proposal exists
    if proposal_id > total_proposals_r(&deps.storage).load()? {
        return Err(StdError::NotFound { kind: "Proposal".to_string(), backtrace: None })
    }

    // Check that proposal is still votable
    let proposal = proposal_r(&deps.storage).load(proposal_id.to_string().as_bytes())?;

    if proposal.vote_status != ProposalStatus::InProgress || proposal.due_date <= env.block.time {
        return Err(StdError::Unauthorized { backtrace: None })
    }

    // Get proposal voting state
    let mut proposal_voting_state = total_proposal_votes_r(&deps.storage).load(
        proposal_id.to_string().as_bytes())?;

    // Check if user has already voted
    match proposal_votes_r(&deps.storage, proposal_id).may_load(
        voter.to_string().as_bytes())? {
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
    total_proposal_votes_w(&mut deps.storage).save(proposal_id.to_string().as_bytes(),
                                                   &proposal_voting_state)?;
    proposal_votes_w(&mut deps.storage, proposal_id).save(voter.to_string().as_bytes(), &votes)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::Vote {
            status: Success,
        })?),
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
        return Err(StdError::Unauthorized { backtrace: None })
    }

    // First validate that the contract exists
    let target_contract = match supported_contract_r(&deps.storage).may_load(target.as_bytes())? {
        None => return Err(StdError::NotFound { kind: "Contract not found".to_string(), backtrace: None }),
        Some(contract) => contract,
    };

    // Check that command exists
    let admin_command = match admin_commands_r(&deps.storage).may_load(command.as_bytes())? {
        None => return Err(StdError::NotFound { kind: "Command not found".to_string(), backtrace: None }),
        Some(admin_c) => admin_c,
    };

    // With command validate that number of variables is equal
    if admin_command.total_arguments != variables.len() as u16 {
        return Err(StdError::GenericErr {
            msg: "Variable number doesnt match up".to_string(), backtrace: None })
    }

    // Replace variable spaces
    let mut finished_command = admin_command.msg;
    for item in variables.iter() {
        finished_command = finished_command.replacen(ADMIN_COMMAND_VARIABLE, item, 1);
    }

    let mut messages= vec![];

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
        due_date: 0,
        is_admin_command: true,
        vote_status: ProposalStatus::AdminRequested,
        run_status: match try_execute_msg(target_contract, Binary::from(finished_command.as_bytes())) {
            Ok(executed_msg) => {
                messages.push(executed_msg);
                Some(Success)
            },
            Err(_) => Some(Failure)
        },
    };

    // Store the proposal
    proposal_w(&mut deps.storage).save(proposal_id.to_string().as_bytes(), &proposal)?;


    Ok(HandleResponse {
        messages,
        log: vec![],
        data: Some( to_binary( &HandleAnswer::TriggerAdminCommand {
            status: proposal.run_status.unwrap(),
            proposal_id
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

    let proposal_id = create_proposal(deps, &env, target_contract, proposal, description)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::CreateProposal {
            status: ResponseStatus::Success,
            proposal_id
        })?),
    })
}

pub fn try_update_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    admin: Option<HumanAddr>,
    staker: Option<Contract>,
    proposal_deadline: Option<u64>,
    minimum_votes: Option<Uint128>,
) -> StdResult<HandleResponse> {

    // It has to be self
    if env.contract.address != env.message.sender {
        return Err(StdError::Unauthorized { backtrace: None })
    }

    config_w(&mut deps.storage).update(|mut state| {
        if let Some(admin) = admin {
            state.admin = admin;
        }
        if let Some(staker) = staker {
            state.staker = staker;
        }
        if let Some(proposal_deadline) = proposal_deadline {
            state.proposal_deadline = proposal_deadline;
        }
        if let Some(minimum_votes) = minimum_votes {
            state.minimum_votes = minimum_votes;
        }

        Ok(state)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success
        })?),
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
        return Err(StdError::Unauthorized { backtrace: None })
    }

    // Cannot be the same name as governance default
    if name == GOVERNANCE_SELF.to_string() {
        return Err(StdError::Unauthorized { backtrace: None })
    }

    // Supported contract cannot exist
    if supported_contract_r(&deps.storage).may_load(name.as_bytes())?.is_some() {
        return Err(StdError::Unauthorized { backtrace: None })
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
        data: Some( to_binary( &HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success
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
        return Err(StdError::Unauthorized { backtrace: None })
    }

    // Cannot be the same name as governance default
    if name == GOVERNANCE_SELF.to_string() {
        return Err(StdError::Unauthorized { backtrace: None })
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
        data: Some( to_binary( &HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success
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
    if env.contract.address != env.message.sender || name == GOVERNANCE_SELF.to_string() {
        return Err(StdError::Unauthorized { backtrace: None })
    }

    // Replace contract
    supported_contract_w(&mut deps.storage).update(name.as_bytes(), |_state| {
        Ok(contract)
    })?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some( to_binary( &HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success
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
        return Err(StdError::Unauthorized { backtrace: None })
    }

    // Admin command cannot exist
    if admin_commands_r(&deps.storage).may_load(name.as_bytes())?.is_some() {
        return Err(StdError::Unauthorized { backtrace: None })
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
        data: Some( to_binary( &HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success
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
        return Err(StdError::Unauthorized { backtrace: None })
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
        data: Some( to_binary( &HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success
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
        return Err(StdError::Unauthorized { backtrace: None })
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
        data: Some( to_binary( &HandleAnswer::UpdateConfig {
            status: ResponseStatus::Success
        })?),
    })
}