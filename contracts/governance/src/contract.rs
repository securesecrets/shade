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
            try_receive_funding,
            try_receive_vote,
            try_trigger,
            try_update,
        },
        try_set_config,
        try_set_runtime_state,
    },
    query,
};
use shade_protocol::{
    c_std::{
        shd_entry_point,
        to_binary,
        Addr,
        Binary,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdError,
        StdResult,
        SubMsg,
        Uint128,
    },
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
    governance::{AuthQuery, QueryData},
    query_auth::helpers::{authenticate_permit, authenticate_vk, PermitAuthentication},
    snip20::helpers::register_receive,
    utils::{
        asset::Contract,
        flexible_msg::FlexibleMsg,
        pad_handle_result,
        pad_query_result,
        storage::default::SingletonStorage,
    },
};

// Used to pad up responses for better privacy.
pub const RESPONSE_BLOCK_SIZE: usize = 256;

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    // Setup config
    Config {
        query: msg.query_auth,
        treasury: msg.treasury,
        vote_token: msg.vote_token.clone(),
        funding_token: msg.funding_token.clone(),
    }
    .save(deps.storage)?;

    let mut messages = vec![];
    if let Some(vote_token) = msg.vote_token.clone() {
        messages.push(SubMsg::new(register_receive(
            env.contract.code_hash.clone(),
            None,
            &vote_token,
        )?));
    }
    if let Some(funding_token) = msg.funding_token.clone() {
        messages.push(SubMsg::new(register_receive(
            env.contract.code_hash.clone(),
            None,
            &funding_token,
        )?));
    }

    // Setups IDs
    ID::set_assembly(deps.storage, Uint128::new(1))?;
    ID::set_profile(deps.storage, Uint128::new(1))?;
    ID::set_assembly_msg(deps.storage, Uint128::zero())?;
    ID::set_contract(deps.storage, Uint128::zero())?;

    // Setup public profile
    msg.public_profile.save(deps.storage, &Uint128::zero())?;

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
    .save(deps.storage, &Uint128::zero())?;

    // Setup admin profile
    msg.admin_profile.save(deps.storage, &Uint128::new(1))?;

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
    .save(deps.storage, &Uint128::new(1))?;

    // Setup generic command
    AssemblyMsg {
        name: "blank message".to_string(),
        assemblies: vec![Uint128::zero(), Uint128::new(1)],
        msg: FlexibleMsg {
            msg: MSG_VARIABLE.to_string(),
            arguments: 1,
        },
    }
    .save(deps.storage, &Uint128::zero())?;

    // Setup self contract
    AllowedContract {
        name: "Governance".to_string(),
        metadata: "Current governance contract, this one".to_string(),
        assemblies: None,
        contract: Contract {
            address: env.contract.address,
            code_hash: env.contract.code_hash,
        },
    }
    .save(deps.storage, &Uint128::zero())?;

    Ok(Response::new().add_submessages(messages))
}

#[shd_entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    pad_handle_result(
        match msg {
            // State setups
            ExecuteMsg::SetConfig {
                query_auth,
                treasury,
                vote_token,
                funding_token,
                ..
            } => try_set_config(
                deps,
                env,
                info,
                query_auth,
                treasury,
                vote_token,
                funding_token,
            ),

            // TODO: set this, must be discussed with team
            ExecuteMsg::SetRuntimeState { state, .. } => {
                try_set_runtime_state(deps, env, info, state)
            }

            // Proposals
            ExecuteMsg::Proposal {
                title,
                metadata,
                contract,
                msg,
                coins,
                ..
            } => try_proposal(deps, env, info, title, metadata, contract, msg, coins),

            ExecuteMsg::Trigger { proposal, .. } => try_trigger(deps, env, info, proposal),
            ExecuteMsg::Cancel { proposal, .. } => try_cancel(deps, env, info, proposal),
            ExecuteMsg::Update { proposal, .. } => try_update(deps, env, info, proposal),
            ExecuteMsg::Receive {
                sender,
                from,
                amount,
                msg,
                memo,
                ..
            } => try_receive_funding(deps, env, info, sender, from, amount, msg, memo),
            ExecuteMsg::ClaimFunding { id } => try_claim_funding(deps, env, info, id),

            ExecuteMsg::ReceiveBalance {
                sender,
                msg,
                balance,
                memo,
            } => try_receive_vote(deps, env, info, sender, msg, balance, memo),

            // Assemblies
            ExecuteMsg::AssemblyVote { proposal, vote, .. } => {
                try_assembly_vote(deps, env, info, proposal, vote)
            }

            ExecuteMsg::AssemblyProposal {
                assembly,
                title,
                metadata,
                msgs,
                ..
            } => try_assembly_proposal(deps, env, info, assembly, title, metadata, msgs),

            ExecuteMsg::AddAssembly {
                name,
                metadata,
                members,
                profile,
                ..
            } => try_add_assembly(deps, env, info, name, metadata, members, profile),

            ExecuteMsg::SetAssembly {
                id,
                name,
                metadata,
                members,
                profile,
                ..
            } => try_set_assembly(deps, env, info, id, name, metadata, members, profile),

            // Assembly Msgs
            ExecuteMsg::AddAssemblyMsg {
                name,
                msg,
                assemblies,
                ..
            } => try_add_assembly_msg(deps, env, info, name, msg, assemblies),

            ExecuteMsg::SetAssemblyMsg {
                id,
                name,
                msg,
                assemblies,
                ..
            } => try_set_assembly_msg(deps, env, info, id, name, msg, assemblies),

            ExecuteMsg::AddAssemblyMsgAssemblies { id, assemblies } => {
                try_add_assembly_msg_assemblies(deps, env, info, id, assemblies)
            }

            // Profiles
            ExecuteMsg::AddProfile { profile, .. } => try_add_profile(deps, env, info, profile),

            ExecuteMsg::SetProfile { id, profile, .. } => {
                try_set_profile(deps, env, info, id, profile)
            }

            // Contracts
            ExecuteMsg::AddContract {
                name,
                metadata,
                contract,
                assemblies,
                ..
            } => try_add_contract(deps, env, info, name, metadata, contract, assemblies),

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
                info,
                id,
                name,
                metadata,
                contract,
                disable_assemblies,
                assemblies,
            ),

            ExecuteMsg::AddContractAssemblies { id, assemblies } => {
                try_add_contract_assemblies(deps, env, info, id, assemblies)
            }
        },
        RESPONSE_BLOCK_SIZE,
    )
}

#[shd_entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
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

            QueryMsg::WithVK { user, key, query } => {
                // Query VK info
                let authenticator = Config::load(deps.storage)?.query;
                if !authenticate_vk(user.clone(), key, &deps.querier, &authenticator)? {
                    return Err(StdError::generic_err("Unauthorized"));
                }

                auth_queries(deps, query, user)
            }

            QueryMsg::WithPermit { permit, query } => {
                // Query Permit info
                let authenticator = Config::load(deps.storage)?.query;
                let res: PermitAuthentication<QueryData> =
                    authenticate_permit(permit, &deps.querier, authenticator)?;

                if res.revoked {
                    return Err(StdError::generic_err("Unauthorized"));
                }

                auth_queries(deps, query, res.sender)
            }
        },
        RESPONSE_BLOCK_SIZE,
    )
}

pub fn auth_queries(deps: Deps, msg: AuthQuery, user: Addr) -> StdResult<Binary> {
    to_binary(&match msg {
        AuthQuery::Proposals { pagination } => query::user_proposals(deps, user, pagination)?,
        AuthQuery::AssemblyVotes { pagination } => {
            query::user_assembly_votes(deps, user, pagination)?
        }
        AuthQuery::Funding { pagination } => query::user_funding(deps, user, pagination)?,
        AuthQuery::Votes { pagination } => query::user_votes(deps, user, pagination)?,
    })
}
