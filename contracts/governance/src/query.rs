use shade_protocol::{
    c_std::{Addr, Deps, StdResult},
    contract_interfaces::governance::{
        assembly::{Assembly, AssemblyMsg},
        contract::AllowedContract,
        profile::Profile,
        proposal::Proposal,
        stored_id::ID,
        Config,
        QueryAnswer,
    },
    governance::{errors::Error, stored_id::UserID, Pagination, ResponseWithID},
    utils::storage::plus::ItemStorage,
};
use std::cmp::min;

pub fn config(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: Config::load(deps.storage)?,
    })
}

pub fn total_proposals(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Total {
        total: ID::proposal(deps.storage)?.checked_add(1).unwrap(),
    })
}

pub fn proposals(deps: Deps, start: u32, end: u32) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let total = ID::proposal(deps.storage)?;

    if start > total {
        return Err(Error::item_not_found(vec![&start.to_string(), "Proposal"]));
    }

    for i in start..=min(end, total) {
        items.push(Proposal::load(deps.storage, i)?);
    }

    Ok(QueryAnswer::Proposals { props: items })
}

pub fn total_profiles(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Total {
        total: ID::profile(deps.storage)?.checked_add(1).unwrap() as u32,
    })
}

pub fn profiles(deps: Deps, start: u16, end: u16) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let total = ID::profile(deps.storage)?;

    if start > total {
        return Err(Error::item_not_found(vec![&start.to_string(), "Profile"]));
    }

    for i in start..=min(end, total) {
        items.push(Profile::load(deps.storage, i)?);
    }

    Ok(QueryAnswer::Profiles { profiles: items })
}

pub fn total_assemblies(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Total {
        total: ID::assembly(deps.storage)?.checked_add(1).unwrap() as u32,
    })
}

pub fn assemblies(deps: Deps, start: u16, end: u16) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let total = ID::assembly(deps.storage)?;

    if start > total {
        return Err(Error::item_not_found(vec![&start.to_string(), "Assembly"]));
    }

    for i in start..=min(end, total) {
        items.push(Assembly::load(deps.storage, i)?);
    }

    Ok(QueryAnswer::Assemblies { assemblies: items })
}

pub fn total_assembly_msgs(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Total {
        total: ID::assembly_msg(deps.storage)?.checked_add(1).unwrap() as u32,
    })
}

pub fn assembly_msgs(deps: Deps, start: u16, end: u16) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let total = ID::assembly_msg(deps.storage)?;

    if start > total {
        return Err(Error::item_not_found(vec![
            &start.to_string(),
            "AssemblyMsg",
        ]));
    }

    for i in start..=min(end, total) {
        items.push(AssemblyMsg::load(deps.storage, i)?);
    }

    Ok(QueryAnswer::AssemblyMsgs { msgs: items })
}

pub fn total_contracts(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Total {
        total: ID::contract(deps.storage)?.checked_add(1).unwrap() as u32,
    })
}

pub fn contracts(deps: Deps, start: u16, end: u16) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let total = ID::contract(deps.storage)?;

    if start > total {
        return Err(Error::item_not_found(vec![&start.to_string(), "Contract"]));
    }

    for i in start..=min(end, total) {
        items.push(AllowedContract::load(deps.storage, i)?);
    }

    Ok(QueryAnswer::Contracts { contracts: items })
}

pub fn user_proposals(deps: Deps, user: Addr, pagination: Pagination) -> StdResult<QueryAnswer> {
    let total = UserID::total_proposals(deps.storage, user.clone())?;

    let start = pagination
        .amount
        .checked_mul(pagination.page as u32)
        .unwrap();
    let mut props = vec![];

    for i in start..start + pagination.amount {
        let id = match UserID::proposal(deps.storage, user.clone(), i) {
            Ok(id) => id,
            Err(_) => break,
        };

        props.push(ResponseWithID {
            prop_id: id,
            data: Proposal::load(deps.storage, id)?,
        });
    }

    Ok(QueryAnswer::UserProposals { props, total })
}

pub fn user_assembly_votes(
    deps: Deps,
    user: Addr,
    pagination: Pagination,
) -> StdResult<QueryAnswer> {
    let total = UserID::total_assembly_votes(deps.storage, user.clone())?;

    let start = pagination
        .amount
        .checked_mul(pagination.page as u32)
        .unwrap();
    let mut votes = vec![];

    for i in start..start + pagination.amount {
        let id = match UserID::assembly_vote(deps.storage, user.clone(), i) {
            Ok(id) => id,
            Err(_) => break,
        };

        votes.push(ResponseWithID {
            prop_id: id,
            data: Proposal::assembly_vote(deps.storage, id, &user)?.unwrap(),
        });
    }

    Ok(QueryAnswer::UserAssemblyVotes { votes, total })
}

pub fn user_funding(deps: Deps, user: Addr, pagination: Pagination) -> StdResult<QueryAnswer> {
    let total = UserID::total_funding(deps.storage, user.clone())?;

    let start = pagination
        .amount
        .checked_mul(pagination.page as u32)
        .unwrap();
    let mut funds = vec![];

    for i in start..start + pagination.amount {
        let id = match UserID::funding(deps.storage, user.clone(), i as u32) {
            Ok(id) => id,
            Err(_) => break,
        };

        funds.push(ResponseWithID {
            prop_id: id,
            data: Proposal::funding(deps.storage, id, &user)?,
        });
    }

    Ok(QueryAnswer::UserFunding { funds, total })
}

pub fn user_votes(deps: Deps, user: Addr, pagination: Pagination) -> StdResult<QueryAnswer> {
    let total = UserID::total_votes(deps.storage, user.clone())?;

    let start = pagination
        .amount
        .checked_mul(pagination.page as u32)
        .unwrap();
    let mut votes = vec![];

    for i in start..start + pagination.amount {
        let id = match UserID::votes(deps.storage, user.clone(), i) {
            Ok(id) => id,
            Err(_) => break,
        };

        votes.push(ResponseWithID {
            prop_id: id,
            data: Proposal::public_vote(deps.storage, id, &user)?.unwrap(),
        });
    }

    Ok(QueryAnswer::UserVotes { votes, total })
}
