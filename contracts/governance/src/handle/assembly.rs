use cosmwasm_math_compat::Uint128;
use cosmwasm_std::{
    from_binary,
    to_binary,
    Api,
    Binary,
    Coin,
    Env,
    Extern,
    HandleResponse,
    HumanAddr,
    Querier,
    StdError,
    StdResult,
    Storage,
};
use shade_protocol::{
    contract_interfaces::governance::{
        assembly::{Assembly, AssemblyMsg},
        contract::AllowedContract,
        profile::{Profile, VoteProfile},
        proposal::{Proposal, ProposalMsg, Status},
        stored_id::ID,
        vote::Vote,
        HandleAnswer,
        MSG_VARIABLE,
    },
    utils::{generic_response::ResponseStatus, storage::default::BucketStorage},
};
use std::convert::TryInto;

pub fn try_assembly_vote<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    proposal: Uint128,
    vote: Vote,
) -> StdResult<HandleResponse> {
    let sender = env.message.sender;

    // Check if proposal in assembly voting
    if let Status::AssemblyVote { end, .. } = Proposal::status(&deps.storage, &proposal)? {
        if end <= env.block.time {
            return Err(StdError::generic_err("Voting time has been reached"));
        }
    } else {
        return Err(StdError::generic_err("Not in assembly vote phase"));
    }
    // Check if user in assembly
    if !Assembly::data(
        &deps.storage,
        &Proposal::assembly(&deps.storage, &proposal)?,
    )?
    .members
    .contains(&sender)
    {
        return Err(StdError::unauthorized());
    }

    let mut tally = Proposal::assembly_votes(&deps.storage, &proposal)?;

    // Assembly votes can only be = 1 uint
    if vote.total_count()? != Uint128::new(1) {
        return Err(StdError::generic_err("Assembly vote can only be one"));
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
    title: String,
    metadata: String,
    msgs: Option<Vec<ProposalMsg>>,
) -> StdResult<HandleResponse> {
    // Get assembly
    let assembly_data = Assembly::data(&deps.storage, &assembly_id)?;

    // Check if public; everyone is allowed
    if assembly_data.profile != Uint128::zero() {
        if !assembly_data.members.contains(&env.message.sender) {
            return Err(StdError::unauthorized());
        }
    }

    // Get profile
    // Check if assembly is enabled
    let profile = Profile::data(&deps.storage, &assembly_data.profile)?;
    if !profile.enabled {
        return Err(StdError::generic_err("Assembly is disabled"));
    }

    let status: Status;

    // Check if assembly voting
    if let Some(vote_settings) = Profile::assembly_voting(&deps.storage, &assembly_data.profile)? {
        status = Status::AssemblyVote {
            start: env.block.time,
            end: env.block.time + vote_settings.deadline,
        }
    }
    // Check if funding
    else if let Some(fund_settings) = Profile::funding(&deps.storage, &assembly_data.profile)? {
        status = Status::Funding {
            amount: Uint128::zero(),
            start: env.block.time,
            end: env.block.time + fund_settings.deadline,
        }
    }
    // Check if token voting
    else if let Some(vote_settings) =
        Profile::public_voting(&deps.storage, &assembly_data.profile)?
    {
        status = Status::Voting {
            start: env.block.time,
            end: env.block.time + vote_settings.deadline,
        }
    }
    // Else push directly to passed
    else {
        status = Status::Passed {
            start: env.block.time,
            end: env.block.time + profile.cancel_deadline,
        }
    }

    let processed_msgs: Option<Vec<ProposalMsg>>;
    if let Some(msgs) = msgs.clone() {
        let mut new_msgs = vec![];
        for msg in msgs.iter() {
            // Check if msg is allowed in assembly
            let assembly_msg = AssemblyMsg::data(&deps.storage, &msg.assembly_msg)?;
            if !assembly_msg.assemblies.contains(&assembly_id) {
                return Err(StdError::unauthorized());
            }

            // Check if msg is allowed in contract
            let contract = AllowedContract::data(&deps.storage, &msg.target)?;
            if let Some(assemblies) = contract.assemblies {
                if !assemblies.contains(&msg.target) {
                    return Err(StdError::unauthorized());
                }
            }

            let vars: Vec<String> = from_binary(&msg.msg)?;
            let binary_msg =
                Binary::from(assembly_msg.msg.create_msg(vars, MSG_VARIABLE)?.as_bytes());

            new_msgs.push(ProposalMsg {
                target: msg.target,
                assembly_msg: msg.assembly_msg,
                msg: binary_msg,
                send: msg.send.clone(),
            });
        }
        processed_msgs = Some(new_msgs);
    } else {
        processed_msgs = None;
    }

    let prop = Proposal {
        proposer: env.message.sender,
        title,
        metadata,
        msgs: processed_msgs,
        assembly: assembly_id,
        assembly_vote_tally: None,
        public_vote_tally: None,
        status,
        status_history: vec![],
        funders: None,
    };

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
    profile: Uint128,
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized());
    }

    let id = ID::add_assembly(&mut deps.storage)?;

    // Check that profile exists
    if profile > ID::profile(&deps.storage)? {
        return Err(StdError::generic_err("Profile not found"));
    }

    Assembly {
        name,
        metadata,
        members,
        profile,
    }
    .save(&mut deps.storage, &id)?;

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
    profile: Option<Uint128>,
) -> StdResult<HandleResponse> {
    if env.message.sender != env.contract.address {
        return Err(StdError::unauthorized());
    }

    let mut assembly = match Assembly::may_load(&mut deps.storage, &id)? {
        None => return Err(StdError::generic_err("Assembly not found")),
        Some(c) => c,
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
            return Err(StdError::generic_err("Profile not found"));
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
