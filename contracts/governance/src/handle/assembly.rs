use cosmwasm_std::{Api, Coin, Env, Extern, HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, to_binary};
use cosmwasm_math_compat::Uint128;
use shade_protocol::governance::assembly::{Assembly, AssemblyMsg};
use shade_protocol::governance::{HandleAnswer, MSG_VARIABLE};
use shade_protocol::governance::profile::{Profile, VoteProfile};
use shade_protocol::governance::proposal::{Proposal, Status};
use shade_protocol::governance::stored_id::ID;
use shade_protocol::governance::vote::Vote;
use shade_protocol::utils::generic_response::ResponseStatus;
use shade_protocol::utils::storage::BucketStorage;

pub fn try_assembly_vote<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    proposal: Uint128,
    vote: Vote
) -> StdResult<HandleResponse> {
    let sender = env.message.sender;

    // Check if proposal in assembly voting
    if let Status::AssemblyVote {end, ..} = Proposal::status(&deps.storage, &proposal)? {
        if end <= env.block.time {
            return Err(StdError::generic_err("Voting time has been reached"))
        }
    }
    else {
        return Err(StdError::generic_err("Not in assembly vote phase"))
    }
    // Check if user in assembly
    if !Assembly::data(&deps.storage, &Proposal::assembly(&deps.storage, &proposal)?)?.members.contains(&sender) {
        return Err(StdError::unauthorized())
    }

    let mut tally = Proposal::assembly_votes(&deps.storage, &proposal)?;

    // Assembly votes can only be = 1 uint
    if vote.total_count()? != Uint128::new(1) {
        return Err(StdError::generic_err("Assembly vote can only be one"))
    }

    // Check if user voted
    if let Some(old_vote) = Proposal::assembly_vote(&deps.storage, &proposal, &sender)? {
        tally = tally.checked_sub(&old_vote)?;
    }
    
    Proposal::save_assembly_vote(&mut deps.storage, &proposal, &sender, &vote)?;
    Proposal::save_assembly_votes(&mut deps.storage, &proposal, &tally.checked_add(&vote)?)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AssemblyVote {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_assembly_proposal<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    assembly_id: Uint128,
    metadata: String,
    contract_id: Option<Uint128>,
    assembly_msg_id: Option<Uint128>,
    variables: Option<Vec<String>>,
    coins: Option<Vec<Coin>>
) -> StdResult<HandleResponse> {

    // Get assembly
    let assembly_data = Assembly::data(&deps.storage, &assembly_id)?;

    // Check if public; everyone is allowed
    if assembly_data.profile != Uint128::zero() {
        if !assembly_data.members.contains(&env.message.sender) {
            return Err(StdError::unauthorized())
        }
    }

    // Get profile
    // Check if assembly is enabled
    let profile = Profile::data(&deps.storage, &assembly_data.profile)?;
    if !profile.enabled {
        return Err(StdError::generic_err("Assembly is disabled"))
    }

    let status: Status;

    // Check if assembly voting
    if let Some(vote_settings) = Profile::assembly_voting(&deps.storage, &assembly_data.profile)? {
        status = Status::AssemblyVote { 
            votes: Vote::default(),
            start: env.block.time, 
            end: env.block.time + vote_settings.deadline 
        }
    }
    // Check if funding
    else if let Some(fund_settings) = Profile::funding(&deps.storage, &assembly_data.profile)? {
        status = Status::Funding {
            amount: Uint128::zero(),
            start: env.block.time,
            end: env.block.time + fund_settings.deadline
        }
    }
    // Check if token voting
    else if let Some(vote_settings) = Profile::public_voting(&deps.storage, &assembly_data.profile)? {
        status = Status::Voting {
            votes: Vote::default(),
            start: env.block.time,
            end: env.block.time + vote_settings.deadline
        }
    }
    // Else push directly to passed
    else {
        status = Status::Passed {
            start: env.block.time,
            end: env.block.time + profile.cancel_deadline
        }
    }
    
    let mut prop = Proposal {
        proposer: env.message.sender,
        metadata,
        target: None,
        assembly_msg: None,
        msg: None,
        send: None,
        assembly: assembly_id,
        assembly_vote_tally: None,
        public_vote_tally: None,
        status,
        status_history: vec![],
        funders: None
    };
    
    if let Some(msg_id) = assembly_msg_id {
        // Check if msg is allowed in assembly
        let assembly_msg = AssemblyMsg::data(&deps.storage, &msg_id)?;
        if !assembly_msg.assemblies.contains(&assembly_id) {
            return Err(StdError::unauthorized())
        }

        prop.assembly_msg = assembly_msg_id;
        
        if let Some(id) = contract_id {
            if id > ID::contract(&deps.storage)? {
                return Err(StdError::generic_err("Contract ID does not exist"))
            }
            prop.target = contract_id;
        }
        else {
            return Err(StdError::generic_err("Contract ID was not specified"))
        }

        // Try to replace variables in msg
        if let Some(vars) = variables {
            prop.msg = Some(to_binary(&assembly_msg.msg.create_msg(vars, MSG_VARIABLE))?);
        }
        else {
            return Err(StdError::generic_err("Variables were not specified"))
        }

        prop.send = coins;
    }

    let prop_id = ID::add_proposal(&mut deps.storage)?;
    prop.save(&mut deps.storage, &prop_id)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AssemblyProposal {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_add_assembly<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    name: String,
    metadata: String,
    members: Vec<HumanAddr>,
    profile: Uint128
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized())
    }

    let id = ID::add_assembly(&mut deps.storage)?;

    // Check that profile exists
    if profile > ID::profile(&deps.storage)? {
        return Err(StdError::generic_err("Profile not found"))
    }

    Assembly {
        name,
        metadata,
        members,
        profile
    }.save(&mut deps.storage, &id)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddAssembly {
            status: ResponseStatus::Success,
        })?),
    })
}

pub fn try_set_assembly<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    id: Uint128,
    name: Option<String>,
    metadata: Option<String>,
    members: Option<Vec<HumanAddr>>,
    profile: Option<Uint128>
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized())
    }

    let mut assembly = match Assembly::may_load(&mut deps.storage, &id)? {
        None => return Err(StdError::generic_err("Assembly not found")),
        Some(c) => c
    };

    if let Some(name) = name {
        assembly.name = name;
    }

    if let Some(metadata) = metadata {
        assembly.metadata = metadata
    }

    if let Some(members) = members {
        assembly.members = members
    }

    if let Some(profile) = profile {
        // Check that profile exists
        if profile > ID::profile(&deps.storage)? {
            return Err(StdError::generic_err("Profile not found"))
        }
        assembly.profile = profile
    }

    assembly.save(&mut deps.storage, &id)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetAssembly {
            status: ResponseStatus::Success,
        })?),
    })
}