use crate::{
    handle::{
        assembly::{try_add_assembly, try_assembly_proposal, try_assembly_vote, try_set_assembly},
        assembly_msg::{
            try_add_assembly_msg,
            try_add_assembly_msg_assemblies,
            try_set_assembly_msg,
        },
        contract::{try_add_contract, try_add_contract_assemblies, try_set_contract},
        profile::{try_add_profile, try_set_profile},
        proposal::{
            try_cancel,
            try_claim_funding,
            try_proposal,
            try_receive,
            try_receive_balance,
            try_trigger,
            try_update,
        },
        try_set_config,
        try_set_runtime_state,
    },
    query,
};
use shade_protocol::c_std::Uint128;
use shade_protocol::c_std::{
    to_binary,
    Api,
    Binary,
    Env,
    DepsMut,
    Response,
    Querier,
    StdError,
    StdResult,
    Storage,
};
use shade_protocol::secret_toolkit::{
    snip20::register_receive_msg,
    utils::{pad_handle_result, pad_query_result},
};
use shade_protocol::{
    contract_interfaces::governance::{
        assembly::{Assembly, AssemblyMsg},
        contract::AllowedContract,
        stored_id::ID,
        Config,
        ExecuteMsg,
        InstantiateMsg,
        QueryMsg,
        MSG_VARIABLE,
    },
    utils::{
        asset::Contract,
        flexible_msg::FlexibleMsg,
        storage::default::{BucketStorage, SingletonStorage},
    },
};

// Used to pad up responses for better privacy.
pub const RESPONSE_BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // Setup config
    Config {
        treasury: msg.treasury.clone(),
        vote_token: msg.vote_token.clone(),
        funding_token: msg.funding_token.clone(),
    }
    .save(&mut deps.storage)?;

    let mut messages = vec![];
    if let Some(vote_token) = msg.vote_token.clone() {
        messages.push(register_receive_msg(
            env.contract_code_hash.clone(),
            None,
            255,
            vote_token.code_hash,
            vote_token.address,
        )?);
    }
    if let Some(funding_token) = msg.funding_token.clone() {
        messages.push(register_receive_msg(
            env.contract_code_hash.clone(),
            None,
            255,
            funding_token.code_hash,
            funding_token.address,
        )?);
    }

    // Setups IDs
    ID::set_assembly(&mut deps.storage, Uint128::new(1))?;
    ID::set_profile(&mut deps.storage, Uint128::new(1))?;
    ID::set_assembly_msg(&mut deps.storage, Uint128::zero())?;
    ID::set_contract(&mut deps.storage, Uint128::zero())?;

    // Setup public profile
    msg.public_profile
        .save(&mut deps.storage, &Uint128::zero())?;

    if msg.public_profile.funding.is_some() {
        if msg.funding_token.is_none() {
            return Err(StdError::generic_err("Funding token must be set"));
        }
    }

    if msg.public_profile.token.is_some() {
        if msg.vote_token.is_none() {
            return Err(StdError::generic_err("Voting token must be set"));
        }
    }

    // Setup public assembly
    Assembly {
        name: "public".to_string(),
        metadata: "All inclusive assembly, acts like traditional governance".to_string(),
        members: vec![],
        profile: Uint128::zero(),
    }
    .save(&mut deps.storage, &Uint128::zero())?;

    // Setup admin profile
    msg.admin_profile
        .save(&mut deps.storage, &Uint128::new(1))?;

    if msg.admin_profile.funding.is_some() {
        if msg.funding_token.is_none() {
            return Err(StdError::generic_err("Funding token must be set"));
        }
    }

    if msg.admin_profile.token.is_some() {
        if msg.vote_token.is_none() {
            return Err(StdError::generic_err("Voting token must be set"));
        }
    }

    // Setup admin assembly
    Assembly {
        name: "admin".to_string(),
        metadata: "Assembly of DAO admins.".to_string(),
        members: msg.admin_members,
        profile: Uint128::new(1),
    }
    .save(&mut deps.storage, &Uint128::new(1))?;

    // Setup generic command
    AssemblyMsg {
        name: "blank message".to_string(),
        assemblies: vec![Uint128::zero(), Uint128::new(1)],
        msg: FlexibleMsg {
            msg: MSG_VARIABLE.to_string(),
            arguments: 1,
        },
    }
    .save(&mut deps.storage, &Uint128::zero())?;

    // Setup self contract
    AllowedContract {
        name: "Governance".to_string(),
        metadata: "Current governance contract, this one".to_string(),
        assemblies: None,
        contract: Contract {
            address: env.contract.address,
            code_hash: env.contract_code_hash,
        },
    }
    .save(&mut deps.storage, &Uint128::zero())?;

    Ok(Response {
        messages,
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: DepsMut,
    env: Env,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    pad_handle_result(
        match msg {
            // State setups
            ExecuteMsg::SetConfig {
                treasury,
                vote_token,
                funding_token,
                ..
            } => try_set_config(deps, env, treasury, vote_token, funding_token),

            // TODO: set this, must be discussed with team
            ExecuteMsg::SetRuntimeState { state, .. } => try_set_runtime_state(deps, env, state),

            // Proposals
            ExecuteMsg::Proposal {
                title,
                metadata,
                contract,
                msg,
                coins,
                ..
            } => try_proposal(deps, env, title, metadata, contract, msg, coins),

            ExecuteMsg::Trigger { proposal, .. } => try_trigger(deps, env, proposal),
            ExecuteMsg::Cancel { proposal, .. } => try_cancel(deps, env, proposal),
            ExecuteMsg::Update { proposal, .. } => try_update(deps, env, proposal),
            ExecuteMsg::Receive {
                sender,
                from,
                amount,
                msg,
                memo,
                ..
            } => try_receive(deps, env, sender, from, amount, msg, memo),
            ExecuteMsg::ClaimFunding { id } => try_claim_funding(deps, env, id),

            ExecuteMsg::ReceiveBalance {
                sender,
                msg,
                balance,
                memo,
            } => try_receive_balance(deps, env, sender, msg, balance, memo),

            // Assemblies
            ExecuteMsg::AssemblyVote { proposal, vote, .. } => {
                try_assembly_vote(deps, env, proposal, vote)
            }

            ExecuteMsg::AssemblyProposal {
                assembly,
                title,
                metadata,
                msgs,
                ..
            } => try_assembly_proposal(deps, env, assembly, title, metadata, msgs),

            ExecuteMsg::AddAssembly {
                name,
                metadata,
                members,
                profile,
                ..
            } => try_add_assembly(deps, env, name, metadata, members, profile),

            ExecuteMsg::SetAssembly {
                id,
                name,
                metadata,
                members,
                profile,
                ..
            } => try_set_assembly(deps, env, id, name, metadata, members, profile),

            // Assembly Msgs
            ExecuteMsg::AddAssemblyMsg {
                name,
                msg,
                assemblies,
                ..
            } => try_add_assembly_msg(deps, env, name, msg, assemblies),

            ExecuteMsg::SetAssemblyMsg {
                id,
                name,
                msg,
                assemblies,
                ..
            } => try_set_assembly_msg(deps, env, id, name, msg, assemblies),

            ExecuteMsg::AddAssemblyMsgAssemblies { id, assemblies } => {
                try_add_assembly_msg_assemblies(deps, env, id, assemblies)
            }

            // Profiles
            ExecuteMsg::AddProfile { profile, .. } => try_add_profile(deps, env, profile),

            ExecuteMsg::SetProfile { id, profile, .. } => try_set_profile(deps, env, id, profile),

            // Contracts
            ExecuteMsg::AddContract {
                name,
                metadata,
                contract,
                assemblies,
                ..
            } => try_add_contract(deps, env, name, metadata, contract, assemblies),

            ExecuteMsg::SetContract {
                id,
                name,
                metadata,
                contract,
                disable_assemblies,
                assemblies,
                ..
            } => try_set_contract(
                deps,
                env,
                id,
                name,
                metadata,
                contract,
                disable_assemblies,
                assemblies,
            ),

            ExecuteMsg::AddContractAssemblies { id, assemblies } => {
                try_add_contract_assemblies(deps, env, id, assemblies)
            }
        },
        RESPONSE_BLOCK_SIZE,
    )
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    pad_query_result(
        match msg {
            QueryMsg::TotalProposals {} => to_binary(&query::total_proposals(deps)?),

            QueryMsg::Proposals { start, end } => to_binary(&query::proposals(deps, start, end)?),

            QueryMsg::TotalAssemblies {} => to_binary(&query::total_assemblies(deps)?),

            QueryMsg::Assemblies { start, end } => to_binary(&query::assemblies(deps, start, end)?),

            QueryMsg::TotalAssemblyMsgs {} => to_binary(&query::total_assembly_msgs(deps)?),

            QueryMsg::AssemblyMsgs { start, end } => {
                to_binary(&query::assembly_msgs(deps, start, end)?)
            }

            QueryMsg::TotalProfiles {} => to_binary(&query::total_profiles(deps)?),

            QueryMsg::Profiles { start, end } => to_binary(&query::profiles(deps, start, end)?),

            QueryMsg::TotalContracts {} => to_binary(&query::total_contracts(deps)?),

            QueryMsg::Contracts { start, end } => to_binary(&query::contracts(deps, start, end)?),

            QueryMsg::Config {} => to_binary(&query::config(deps)?),
        },
        RESPONSE_BLOCK_SIZE,
    )
}
