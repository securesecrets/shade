use crate::{
    handle::{
        assembly::{try_add_assembly, try_assembly_proposal, try_assembly_vote, try_set_assembly},
        assembly_msg::{
            try_add_assembly_msg,
            try_add_assembly_msg_assemblies,
            try_set_assembly_msg,
        },
        authorized,
        contract::{try_add_contract, try_add_contract_assemblies, try_set_contract},
        migration::{try_migrate, try_migrate_data, try_receive_migration_data},
        profile::{try_add_profile, try_set_profile},
        proposal::{
            try_cancel,
            try_claim_funding,
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
        from_binary,
        shd_entry_point,
        to_binary,
        Addr,
        Api,
        Binary,
        Deps,
        DepsMut,
        Env,
        MessageInfo,
        Querier,
        Reply,
        Response,
        StdError,
        StdResult,
        Storage,
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
    governance::{AuthQuery, InstantiateMsgResponse, QueryData, RuntimeState},
    query_auth,
    query_auth::helpers::{authenticate_permit, authenticate_vk, PermitAuthentication},
    snip20::helpers::register_receive,
    utils::{
        asset::Contract,
        flexible_msg::FlexibleMsg,
        pad_handle_result,
        pad_query_result,
        storage::{
            default::{BucketStorage, SingletonStorage},
            plus::ItemStorage,
        },
        Query,
    },
};

// Used to pad up responses for better privacy.
pub const RESPONSE_BLOCK_SIZE: usize = 256;

#[shd_entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let migrated_from: Option<Contract>;

    if let Some(migrator) = msg.migrator {
        ID::set_assembly(deps.storage, migrator.assembly)?;
        ID::set_profile(deps.storage, migrator.profile)?;
        ID::set_assembly_msg(deps.storage, migrator.assemblyMsg)?;
        ID::set_contract(deps.storage, migrator.contract)?;
        migrated_from = Some(migrator.source);
    } else {
        // Setups IDs
        ID::set_assembly(deps.storage, Uint128::new(1))?;
        ID::set_profile(deps.storage, Uint128::new(1))?;
        ID::set_assembly_msg(deps.storage, Uint128::zero())?;
        ID::set_contract(deps.storage, Uint128::zero())?;
        migrated_from = None;
    }

    // Setup config
    Config {
        query: msg.query_auth,
        treasury: msg.treasury,
        vote_token: msg.vote_token.clone(),
        funding_token: msg.funding_token.clone(),
        migrated_from,
        migrated_to: None,
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

    // Only initialize the data if not migrating
    if let Some(assemblies) = msg.assemblies {
        // Setup public profile
        assemblies
            .public_profile
            .save(deps.storage, &Uint128::zero())?;

        if assemblies.public_profile.funding.is_some() {
            if msg.funding_token.is_none() {
                return Err(StdError::generic_err("Funding token must be set"));
            }
        }

        if assemblies.public_profile.token.is_some() {
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
        assemblies
            .admin_profile
            .save(deps.storage, &Uint128::new(1))?;

        if assemblies.admin_profile.funding.is_some() {
            if msg.funding_token.is_none() {
                return Err(StdError::generic_err("Funding token must be set"));
            }
        }

        if assemblies.admin_profile.token.is_some() {
            if msg.vote_token.is_none() {
                return Err(StdError::generic_err("Voting token must be set"));
            }
        }

        // Setup admin assembly
        Assembly {
            name: "admin".to_string(),
            metadata: "Assembly of DAO admins.".to_string(),
            members: assemblies.admin_members,
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
    }

    // Set runtime
    RuntimeState::Normal.save(deps.storage)?;

    Ok(Response::new()
        .add_submessages(messages)
        .set_data(to_binary(&InstantiateMsgResponse {
            contract: Contract {
                address: env.contract.address.clone(),
                code_hash: env.contract.code_hash.clone(),
            },
        })?))
}

#[shd_entry_point]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Trigger { .. } // Will be deprecated
        | ExecuteMsg::Cancel { .. } // Will also be deprecated
        | ExecuteMsg::Update { .. } // Gets halted 
        | ExecuteMsg::Receive { .. } // Gets halted
        | ExecuteMsg::ClaimFunding { .. } // Gets halted
        | ExecuteMsg::AssemblyVote { .. } // Gets halted
        | ExecuteMsg::ReceiveBalance { .. } // Gets halted
        | ExecuteMsg::AssemblyProposal { .. } // Gets halted with special permissions
        | ExecuteMsg::MigrateData { .. }
        | ExecuteMsg::ReceiveMigrationData { .. } => {}
        // Only callable by itself
        _ => authorized(deps.storage, &env, &info)?,
    }

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

            ExecuteMsg::SetRuntimeState { state, .. } => {
                try_set_runtime_state(deps, env, info, state)
            }

            // Proposals
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

            // Migration
            ExecuteMsg::Migrate {
                id,
                label,
                code_hash,
            } => try_migrate(deps, env, info, id, label, code_hash),

            ExecuteMsg::MigrateData { data, total } => {
                try_migrate_data(deps, env, info, data, total)
            }

            ExecuteMsg::ReceiveMigrationData { data } => {
                try_receive_migration_data(deps, env, info, data)
            }
        },
        RESPONSE_BLOCK_SIZE,
    )
}

#[shd_entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
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

const MIGRATION_REPLY: u64 = 0;
const PROPOSAL_REPLY: u64 = 1;
#[shd_entry_point]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> StdResult<Response> {
    match msg.id {
        MIGRATION_REPLY => {
            // Get the returned address and code_hash
            let res: InstantiateMsgResponse = from_binary(&msg.result.unwrap().data.unwrap())?;

            let mut config = Config::load(deps.storage)?;
            config.migrated_to = Some(res.contract);
            config.save(deps.storage)?;
        }
        _ => return Err(StdError::generic_err("Reply ID not recognized")),
    }

    Ok(Response::new())
}
