use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{Api, Extern, Querier, StdError, StdResult, Storage};
use shade_protocol::governance::{
    proposal::{ProposalStatus, QueriedProposal},
    QueryAnswer,
};

use crate::{
    proposal_state::{
        proposal_funding_deadline_r, proposal_funding_r, proposal_r, proposal_run_status_r,
        proposal_status_r, proposal_voting_deadline_r, total_proposal_votes_r, total_proposals_r,
    },
    state::{
        admin_commands_list_r, admin_commands_r, supported_contract_r, supported_contracts_list_r,
    },
};

fn build_proposal<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    proposal_id: Uint128,
) -> StdResult<QueriedProposal> {
    let proposal = proposal_r(&deps.storage).load(proposal_id.to_string().as_bytes())?;

    Ok(QueriedProposal {
        id: proposal.id,
        target: proposal.target,
        msg: proposal.msg,
        description: proposal.description,
        funding_deadline: proposal_funding_deadline_r(&deps.storage)
            .load(proposal_id.to_string().as_bytes())?,
        voting_deadline: proposal_voting_deadline_r(&deps.storage)
            .may_load(proposal_id.to_string().as_bytes())?,
        total_funding: proposal_funding_r(&deps.storage)
            .load(proposal_id.to_string().as_bytes())?,
        status: proposal_status_r(&deps.storage).load(proposal_id.to_string().as_bytes())?,
        run_status: proposal_run_status_r(&deps.storage)
            .may_load(proposal_id.to_string().as_bytes())?,
    })
}

pub fn proposals<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start: Uint128,
    end: Uint128,
    status: Option<ProposalStatus>,
) -> StdResult<QueryAnswer> {
    let mut proposals: Vec<QueriedProposal> = vec![];

    let max = total_proposals_r(&deps.storage).load()?;

    if start > max {
        return Err(StdError::NotFound {
            kind: "Proposal doesnt exist".to_string(),
            backtrace: None,
        });
    }

    let clamped_start = start.max(Uint128::new(1u128));

    for i in clamped_start.u128()..((end + clamped_start).min(max).u128() + 1) {
        let proposal = build_proposal(deps, Uint128::new(i))?;

        // Filter proposal by status if it was specified in fn params.
        if let Some(s) = &status {
            if s != &proposal.status {
                continue;
            }
        }
        proposals.push(proposal)
    }

    Ok(QueryAnswer::Proposals { proposals })
}

pub fn proposal<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    proposal_id: Uint128,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Proposal {
        proposal: build_proposal(deps, proposal_id)?,
    })
}

pub fn total_proposals<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::TotalProposals {
        total: total_proposals_r(&deps.storage).load()?,
    })
}

pub fn proposal_votes<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    proposal_id: Uint128,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::ProposalVotes {
        status: total_proposal_votes_r(&deps.storage).load(proposal_id.to_string().as_bytes())?,
    })
}

pub fn supported_contracts<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::SupportedContracts {
        contracts: supported_contracts_list_r(&deps.storage).load()?,
    })
}

pub fn supported_contract<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    name: String,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::SupportedContract {
        contract: supported_contract_r(&deps.storage).load(name.as_bytes())?,
    })
}

pub fn admin_commands<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::AdminCommands {
        commands: admin_commands_list_r(&deps.storage).load()?,
    })
}

pub fn admin_command<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    name: String,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::AdminCommand {
        command: admin_commands_r(&deps.storage).load(name.as_bytes())?,
    })
}
