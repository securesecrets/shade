use crate::handle::authorize_assembly;
use shade_protocol::{
    c_std::{
        from_binary,
        to_binary,
        Addr,
        Binary,
        DepsMut,
        Env,
        MessageInfo,
        Response,
        StdResult,
        Uint128,
    },
    contract_interfaces::governance::{
        assembly::{Assembly, AssemblyMsg},
        contract::AllowedContract,
        profile::Profile,
        proposal::{Proposal, ProposalMsg, Status},
        stored_id::{UserID, ID},
        vote::Vote,
        ExecuteAnswer,
        MSG_VARIABLE,
    },
    governance::errors::Error,
    utils::generic_response::ResponseStatus,
};

pub fn try_assembly_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    proposal: u32,
    vote: Vote,
) -> StdResult<Response> {
    authorize_assembly(
        deps.storage,
        &info,
        Proposal::assembly(deps.storage, proposal)?,
    )?;

    let sender = info.sender;

    // Check if proposal in assembly voting
    if let Status::AssemblyVote { end, .. } = Proposal::status(deps.storage, proposal)? {
        if end <= env.block.time.seconds() {
            return Err(Error::voting_ended(vec![&end.to_string()]));
        }
    } else {
        return Err(Error::not_assembly_voting(vec![]));
    }

    let mut tally = Proposal::assembly_votes(deps.storage, proposal)?;

    // Assembly votes can only be = 1 uint
    if vote.total_count()? != Uint128::new(1) {
        return Err(Error::assembly_vote_qty(vec![]));
    }

    // Check if user voted
    if let Some(old_vote) = Proposal::assembly_vote(deps.storage, proposal, &sender)? {
        tally = tally.checked_sub(&old_vote)?;
    }

    Proposal::save_assembly_vote(deps.storage, proposal, &sender, &vote)?;
    Proposal::save_assembly_votes(deps.storage, proposal, &tally.checked_add(&vote)?)?;

    // Save data for user queries
    UserID::add_assembly_vote(deps.storage, sender.clone(), proposal.clone())?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::AssemblyVote {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn try_assembly_proposal(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    assembly_id: u16,
    title: String,
    metadata: String,
    msgs: Option<Vec<ProposalMsg>>,
) -> StdResult<Response> {
    // Get assembly
    let assembly_data = authorize_assembly(deps.storage, &info, assembly_id)?;

    // Get profile
    // Check if assembly is enabled
    let profile = Profile::data(deps.storage, assembly_data.profile)?;

    let status: Status;

    // Check if assembly voting
    if let Some(vote_settings) = Profile::assembly_voting(deps.storage, assembly_data.profile)? {
        status = Status::AssemblyVote {
            start: env.block.time.seconds(),
            end: env.block.time.seconds() + vote_settings.deadline,
        }
    }
    // Check if funding
    else if let Some(fund_settings) = Profile::funding(deps.storage, assembly_data.profile)? {
        status = Status::Funding {
            amount: Uint128::zero(),
            start: env.block.time.seconds(),
            end: env.block.time.seconds() + fund_settings.deadline,
        }
    }
    // Check if token voting
    else if let Some(vote_settings) = Profile::public_voting(deps.storage, assembly_data.profile)?
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
            let assembly_msg = AssemblyMsg::data(deps.storage, msg.assembly_msg)?;
            if !assembly_msg.assemblies.contains(&assembly_id) {
                return Err(Error::msg_not_in_assembly(vec![]));
            }

            // Check if msg is allowed in contract
            let contract = AllowedContract::data(deps.storage, msg.target)?;
            if let Some(assemblies) = contract.assemblies {
                if !assemblies.contains(&msg.target) {
                    return Err(Error::msg_not_in_contract(vec![]));
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

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::AssemblyProposal {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn try_add_assembly(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    name: String,
    metadata: String,
    members: Vec<Addr>,
    profile: u16,
) -> StdResult<Response> {
    let id = ID::add_assembly(deps.storage)?;

    // Check that profile exists
    if profile > ID::profile(deps.storage)? {
        return Err(Error::item_not_found(vec![&profile.to_string(), "Profile"]));
    }

    Assembly {
        name,
        metadata,
        members,
        profile,
    }
    .save(deps.storage, id)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::AddAssembly {
            status: ResponseStatus::Success,
        })?),
    )
}

pub fn try_set_assembly(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    id: u16,
    name: Option<String>,
    metadata: Option<String>,
    members: Option<Vec<Addr>>,
    profile: Option<u16>,
) -> StdResult<Response> {
    let mut assembly = match Assembly::may_load(deps.storage, id)? {
        None => return Err(Error::item_not_found(vec![&id.to_string(), "Assembly"])),
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
            return Err(Error::item_not_found(vec![&profile.to_string(), "Profile"]));
        }
        assembly.profile = profile
    }

    assembly.save(deps.storage, id)?;

    Ok(
        Response::new().set_data(to_binary(&ExecuteAnswer::SetAssembly {
            status: ResponseStatus::Success,
        })?),
    )
}
