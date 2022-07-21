use shade_protocol::c_std::{MessageInfo, Uint128};
use shade_protocol::c_std::{
    from_binary,
    to_binary,
    Api,
    Binary,
    Coin,
    Env,
    DepsMut,
    Response,
    Addr,
    Querier,
    StdError,
    StdResult,
    Storage,
};
use shade_protocol::{
    contract_interfaces::governance::{
        assembly::{Assembly, AssemblyMsg},
        contract::AllowedContract,
        profile::Profile,
        proposal::{Proposal, ProposalMsg, Status},
        stored_id::{UserID, ID},
        vote::Vote,
        HandleAnswer,
        MSG_VARIABLE,
    },
    utils::{generic_response::ResponseStatus, storage::default::BucketStorage},
};

pub fn try_assembly_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal: Uint128,
    vote: Vote,
) -> StdResult<Response> {
    let sender = info.sender;

    // Check if proposal in assembly voting
    if let Status::AssemblyVote { end, .. } = Proposal::status(deps.storage, &proposal)? {
        if end <= env.block.time.seconds() {
            return Err(StdError::generic_err("Voting time has been reached"));
        }
    } else {
        return Err(StdError::generic_err("Not in assembly vote phase"));
    }
    // Check if user in assembly
    if !Assembly::data(
        deps.storage,
        &Proposal::assembly(deps.storage, &proposal)?,
    )?
    .members
    .contains(&sender)
    {
        return Err(StdError::generic_err("unauthorized"))
    }

    let mut tally = Proposal::assembly_votes(deps.storage, &proposal)?;

    // Assembly votes can only be = 1 uint
    if vote.total_count()? != Uint128::new(1) {
        return Err(StdError::generic_err("Assembly vote can only be one"));
    }

    // Check if user voted
    if let Some(old_vote) = Proposal::assembly_vote(deps.storage, &proposal, &sender)? {
        tally = tally.checked_sub(&old_vote)?;
    }

    Proposal::save_assembly_vote(deps.storage, &proposal, &sender, &vote)?;
    Proposal::save_assembly_votes(deps.storage, &proposal, &tally.checked_add(&vote)?)?;

    // Save data for user queries
    UserID::add_assembly_vote(deps.storage, sender.clone(), proposal.clone())?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::AssemblyVote {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_assembly_proposal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    assembly_id: Uint128,
    title: String,
    metadata: String,
    msgs: Option<Vec<ProposalMsg>>,
) -> StdResult<Response> {
    // Get assembly
    let assembly_data = Assembly::data(deps.storage, &assembly_id)?;

    // Check if public; everyone is allowed
    if assembly_data.profile != Uint128::zero() {
        if !assembly_data.members.contains(&info.sender) {
            return Err(StdError::generic_err("unauthorized"));
        }
    }

    // Get profile
    // Check if assembly is enabled
    let profile = Profile::data(deps.storage, &assembly_data.profile)?;
    if !profile.enabled {
        return Err(StdError::generic_err("Assembly is disabled"));
    }

    let status: Status;

    // Check if assembly voting
    if let Some(vote_settings) = Profile::assembly_voting(deps.storage, &assembly_data.profile)? {
        status = Status::AssemblyVote {
            start: env.block.time.seconds(),
            end: env.block.time.seconds() + vote_settings.deadline,
        }
    }
    // Check if funding
    else if let Some(fund_settings) = Profile::funding(deps.storage, &assembly_data.profile)? {
        status = Status::Funding {
            amount: Uint128::zero(),
            start: env.block.time.seconds(),
            end: env.block.time.seconds() + fund_settings.deadline,
        }
    }
    // Check if token voting
    else if let Some(vote_settings) =
        Profile::public_voting(deps.storage, &assembly_data.profile)?
    {
        status = Status::Voting {
            start: env.block.time.seconds(),
            end: env.block.time.seconds() + vote_settings.deadline,
        }
    }
    // Else push directly to passed
    else {
        status = Status::Passed {
            start: env.block.time.seconds(),
            end: env.block.time.seconds() + profile.cancel_deadline,
        }
    }

    let processed_msgs: Option<Vec<ProposalMsg>>;
    if let Some(msgs) = msgs.clone() {
        let mut new_msgs = vec![];
        for msg in msgs.iter() {
            // Check if msg is allowed in assembly
            let assembly_msg = AssemblyMsg::data(deps.storage, &msg.assembly_msg)?;
            if !assembly_msg.assemblies.contains(&assembly_id) {
                return Err(StdError::generic_err("unauthorized"));
            }

            // Check if msg is allowed in contract
            let contract = AllowedContract::data(deps.storage, &msg.target)?;
            if let Some(assemblies) = contract.assemblies {
                if !assemblies.contains(&msg.target) {
                    return Err(StdError::generic_err("unauthorized"));
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
        proposer: info.sender,
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

    prop.save(deps.storage)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::AssemblyProposal {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_add_assembly(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    name: String,
    metadata: String,
    members: Vec<Addr>,
    profile: Uint128,
) -> StdResult<Response> {
    if info.sender != env.contract.address {
        return Err(StdError::generic_err("unauthorized"));
    }

    let id = ID::add_assembly(deps.storage)?;

    // Check that profile exists
    if profile > ID::profile(deps.storage)? {
        return Err(StdError::generic_err("Profile not found"));
    }

    Assembly {
        name,
        metadata,
        members,
        profile,
    }
    .save(deps.storage, &id)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::AddAssembly {
            status: ResponseStatus::Success,
        })?))
}

pub fn try_set_assembly(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    id: Uint128,
    name: Option<String>,
    metadata: Option<String>,
    members: Option<Vec<Addr>>,
    profile: Option<Uint128>,
) -> StdResult<Response> {
    if info.sender != env.contract.address {
        return Err(StdError::generic_err("unauthorized"));
    }

    let mut assembly = match Assembly::may_load(deps.storage, &id)? {
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
        if profile > ID::profile(deps.storage)? {
            return Err(StdError::generic_err("Profile not found"));
        }
        assembly.profile = profile
    }

    assembly.save(deps.storage, &id)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetAssembly {
            status: ResponseStatus::Success,
        })?))
}
