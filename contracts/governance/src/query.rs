use cosmwasm_std::{Api, Extern, Querier, StdError, StdResult, Storage, Uint128};
use shade_protocol::governance::{QueryAnswer, Proposal};
use crate::state::{total_proposals_r, proposal_r, supported_contracts_list_r, admin_commands_list_r, supported_contract_r, admin_commands_r};

pub fn proposals<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    total: Uint128,
    start: Uint128) -> StdResult<QueryAnswer> {

    let mut proposals: Vec<Proposal> = vec![];

    let max = total_proposals_r(&deps.storage).load()?;

    if start > max {
        return Err(StdError::NotFound { kind: "Proposal doesnt exist".to_string(), backtrace: None })
    }

    let clamped_start = start.max(Uint128(1));

    for i in clamped_start.u128()..((total+clamped_start).min(max).u128() + 1) {
        proposals.push(proposal_r(&deps.storage).load(Uint128(i).to_string().as_bytes())?)
    }

    Ok(QueryAnswer::Proposals { proposals })
}

pub fn proposal<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    proposal_id: Uint128) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::Proposal {
        proposal: proposal_r(&deps.storage).load(proposal_id.to_string().as_bytes())?
    })
}

pub fn supported_contracts<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::SupportedContracts {
        contracts: supported_contracts_list_r(&deps.storage).load()? })
}

pub fn supported_contract<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    name: String) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::SupportedContract {
        contract: supported_contract_r(&deps.storage).load(name.as_bytes())? })
}

pub fn admin_commands<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::AdminCommands { commands: admin_commands_list_r(&deps.storage).load()? })
}

pub fn admin_command<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    name: String) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::AdminCommand {
        command: admin_commands_r(&deps.storage).load(name.as_bytes())? })
}