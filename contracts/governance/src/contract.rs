use cosmwasm_std::{to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdResult, Storage, Uint128};
use shade_protocol::{
    governance::{
        InitMsg, HandleMsg,
        QueryMsg, Config,
    },
};
use crate::{
    state::{config_w, total_proposals_w},
    handle,
    query
};
use crate::state::{admin_commands_list_w, supported_contracts_list_w};
use crate::handle::try_disable_staker;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {

    let state = Config {
        admin: match msg.admin {
            None => { env.message.sender.clone() }
            Some(admin) => { admin }
        },
        staker: msg.staker,
        proposal_deadline: msg.proposal_deadline,
        minimum_votes: msg.quorum
    };

    config_w(&mut deps.storage).save(&state)?;

    // Initialize total proposal counter
    total_proposals_w(&mut deps.storage).save(&Uint128(0))?;

    // Initialize lists
    admin_commands_list_w(&mut deps.storage).save(&vec![])?;
    supported_contracts_list_w(&mut deps.storage).save(&vec![])?;

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
        // Proposals
        HandleMsg::CreateProposal { target_contract, proposal, description
        } => handle::try_create_proposal(deps, &env, target_contract,
                                         Binary::from(proposal.as_bytes()), description),

        // Self interactions
        // Config
        HandleMsg::UpdateConfig { admin, staker, proposal_deadline,
            minimum_votes } =>
            handle::try_update_config(deps, &env, admin, staker, proposal_deadline, minimum_votes),

        HandleMsg::DisableStaker {} => try_disable_staker(deps, &env),

        // Supported contract
        HandleMsg::AddSupportedContract { name, contract
        } => handle::try_add_supported_contract(deps, &env, name, contract),

        HandleMsg::RemoveSupportedContract { name
        } => handle::try_remove_supported_contract(deps, &env, name),

        HandleMsg::UpdateSupportedContract { name, contract
        } => handle::try_update_supported_contract(deps, &env, name, contract),

        // Admin command
        HandleMsg::AddAdminCommand { name, proposal
        } => handle::try_add_admin_command(deps, &env, name, proposal),

        HandleMsg::RemoveAdminCommand { name
        } => handle::try_remove_admin_command(deps, &env, name),

        HandleMsg::UpdateAdminCommand { name, proposal
        } => handle::try_update_admin_command(deps, &env, name, proposal),

        // User interaction
        HandleMsg::MakeVote { voter, proposal_id, votes
        } => handle::try_vote(deps, &env, voter, proposal_id, votes),

        HandleMsg::TriggerProposal { proposal_id
        } => handle::try_trigger_proposal(deps, &env, proposal_id),

        // Admin interactions
        HandleMsg::TriggerAdminCommand { target, command,
            variables, description
        } => handle::try_trigger_admin_command(deps, &env, target, command, variables, description),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetProposals { total, start
        } => to_binary(&query::proposals(deps, total, start)?),

        QueryMsg::GetProposal { proposal_id } => to_binary(
            &query::proposal(deps, proposal_id)?),

        QueryMsg::GetTotalProposals {} => to_binary(
            &query::total_proposals(deps)?),

        QueryMsg::GetProposalVotes { proposal_id } => to_binary(
            &query::proposal_votes(deps, proposal_id)?),

        QueryMsg::GetSupportedContracts { } => to_binary(&query::supported_contracts(deps)?),

        QueryMsg::GetSupportedContract { name } => to_binary(
            &query::supported_contract(deps, name)?),

        QueryMsg::GetAdminCommands {} => to_binary(&query::admin_commands(deps)?),

        QueryMsg::GetAdminCommand { name
        } => to_binary(&query::admin_command(deps, name)?),
    }
}
