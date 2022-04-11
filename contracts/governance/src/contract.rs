use crate::{
    handle,
    proposal_state::total_proposals_w,
    query,
    state::{admin_commands_list_w, config_w, supported_contracts_list_w},
};
use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdResult, Storage,
};
use secret_cosmwasm_math_compat::Uint128;
use secret_toolkit::snip20::register_receive_msg;
use secret_toolkit::utils::{pad_handle_result, pad_query_result};
use shade_protocol::governance::{MSG_VARIABLE, Config, HandleMsg, InitMsg, QueryMsg};
use shade_protocol::governance::assembly::{Assembly, AssemblyMsg};
use shade_protocol::governance::contract::AllowedContract;
use shade_protocol::utils::asset::Contract;
use shade_protocol::utils::flexible_msg::FlexibleMsg;
use shade_protocol::utils::storage::{BucketStorage, SingletonStorage};
use crate::handle::{try_set_config, try_set_runtime_state};
use crate::handle::assembly::{try_add_assembly, try_assembly_proposal, try_assembly_vote, try_set_assembly};
use crate::handle::assembly_msg::{try_add_assembly_msg, try_set_assembly_msg};
use crate::handle::contract::{try_add_contract, try_set_contract};
use crate::handle::profile::{try_add_profile, try_set_profile};
use crate::handle::proposal::{try_cancel, try_proposal, try_receive, try_trigger, try_update};
use crate::state::ID;

// Used to pad up responses for better privacy.
pub const RESPONSE_BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    // Setup config
    Config {
        treasury: msg.treasury,
        vote_token: msg.vote_token,
        funding_token: msg.funding_token
    }.save(&mut deps.storage)?;

    // Setups IDs
    ID::set_assembly(&mut deps.storage, Uint128(1))?;
    ID::set_profile(&mut deps.storage, Uint128(1))?;
    ID::set_assembly_msg(&mut deps.storage, Uint128::zero())?;
    ID::set_contract(&mut deps.storage, Uint128::zero())?;

    // Setup public profile
    msg.public_profile.save(&mut deps.storage, &Uint128::zero())?;
    // Setup public assembly
    Assembly {
        name: "public".to_string(),
        metadata: "All inclusive assembly, acts like traditional governance".to_string(),
        members: vec![],
        profile: Uint128::zero()
    }.save(&mut deps.storage, &Uint128::zero())?;

    // Setup admin profile
    msg.admin_profile.save(&mut deps.storage, &Uint128(1))?;
    // Setup admin assembly
    Assembly {
        name: "admin".to_string(),
        metadata: "Assembly of DAO admins.".to_string(),
        members: msg.admin_members,
        profile: Uint128::zero()
    }.save(&mut deps.storage, &Uint128(1))?;

    // Setup generic command
    AssemblyMsg {
        name: "blank message".to_string(),
        assemblys: vec![Uint128::zero(), Uint128(1)],
        msg: FlexibleMsg { msg: MSG_VARIABLE.to_string(), arguments: 1 }
    }.save(&mut deps.storage, &Uint128::zero())?;

    // Setup self contract
    AllowedContract {
        name: "Governance".to_string(),
        metadata: "Current governance contract, this one".to_string(),
        contract: Contract { address: env.contract.address, code_hash: env.contract_code_hash }
    }.save(&mut deps.storage, &Uint128::zero())?;

    Ok(InitResponse {
        messages: vec![],
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    pad_handle_result(
        match msg {
            // State setups
            HandleMsg::SetConfig { treasury, vote_token, funding_token, ..
            } => try_set_config(deps, env, treasury, vote_token, funding_token),

            HandleMsg::SetRuntimeState { state, .. } => try_set_runtime_state(deps, env, state),

            // Proposals
            HandleMsg::Proposal { metadata, contract, msg, ..
            } => try_proposal(deps, env, metadata, contract, msg),

            HandleMsg::Trigger { proposal, .. } => try_trigger(deps, env, proposal),
            HandleMsg::Cancel { proposal, .. } => try_cancel(deps, env, proposal),
            HandleMsg::Update { proposal, .. } => try_update(deps, env, proposal),
            HandleMsg::Receive { sender, from, amount, msg, memo, ..
            } => try_receive(deps, env, sender, from, amount, msg, memo),

            // Assemblys
            HandleMsg::AssemblyVote { proposal, vote, ..
            } => try_assembly_vote(deps, env, proposal, vote),

            HandleMsg::AssemblyProposal { assembly, metadata, contract, assembly_msg, variables, ..
            } => try_assembly_proposal(deps, env, assembly, metadata, contract, assembly_msg, variables),

            HandleMsg::AddAssembly { name, metadata, members, profile, ..
            } => try_add_assembly(deps, env, name, metadata, members, profile),

            HandleMsg::SetAssembly { id, name, metadata, members, profile, ..
            } => try_set_assembly(deps, env, id, name, metadata, members, profile),

            // Assembly Msgs
            HandleMsg::AddAssemblyMsg { name, msg, assemblys, ..
            } => try_add_assembly_msg(deps, env, name, msg, assemblys),

            HandleMsg::SetAssemblyMsg { id, name, msg, assemblys, ..
            } => try_set_assembly_msg(deps, env, id, name, msg, assemblys),

            // Profiles
            HandleMsg::AddProfile { profile, .. } => try_add_profile(deps, env, profile),
            HandleMsg::SetProfile { id, profile, .. } => try_set_profile(deps, env, id, profile),

            // Contracts
            HandleMsg::AddContract { name, metadata, contract, .. } => try_add_contract(deps, env, name, metadata, contract),
            HandleMsg::SetContract { id, name, metadata, contract, .. } => try_set_contract(deps, env, id, name, metadata, contract),
        },
        RESPONSE_BLOCK_SIZE
    )
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    pad_query_result(
        match msg {
            QueryMsg::Proposals { start, end
            } => to_binary(&query::proposals(deps, start, end)?),

            QueryMsg::Assemblys { start, end
            } => to_binary(&query::assemblys(deps, start, end)?),

            QueryMsg::AssemblyMsgs { start, end
            } => to_binary(&query::assemblymsgs(deps, start, end)?),

            QueryMsg::Profiles { start, end
            } => to_binary(&query::profiles(deps, start, end)?),
        },
        RESPONSE_BLOCK_SIZE
    )
}
