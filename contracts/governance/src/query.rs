use shade_protocol::math_compat::Uint128;
use shade_protocol::c_std::{Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage};
use shade_protocol::{
    contract_interfaces::governance::{
        assembly::{Assembly, AssemblyMsg},
        contract::AllowedContract,
        profile::Profile,
        proposal::Proposal,
        stored_id::ID,
        Config,
        QueryAnswer,
    },
    utils::storage::default::SingletonStorage,
};
use shade_protocol::contract_interfaces::governance::{Pagination, ResponseWithID};
use shade_protocol::contract_interfaces::governance::stored_id::UserID;

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: Config::load(&deps.storage)?,
    })
}

pub fn total_proposals<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Total {
        total: ID::proposal(&deps.storage)?.checked_add(Uint128::new(1))?,
    })
}

pub fn proposals<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start: Uint128,
    end: Uint128,
) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let mut end = end;
    let total = ID::proposal(&deps.storage)?;

    if start > total {
        return Err(StdError::generic_err("Proposal not found"));
    }

    if end > total {
        end = total;
    }

    for i in start.u128()..=end.u128() {
        items.push(Proposal::load(&deps.storage, &Uint128::new(i))?);
    }

    Ok(QueryAnswer::Proposals { props: items })
}

pub fn total_profiles<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Total {
        total: ID::profile(&deps.storage)?.checked_add(Uint128::new(1))?,
    })
}

pub fn profiles<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start: Uint128,
    end: Uint128,
) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let mut end = end;
    let total = ID::profile(&deps.storage)?;

    if start > total {
        return Err(StdError::generic_err("Profile not found"));
    }

    if end > total {
        end = total;
    }

    for i in start.u128()..=end.u128() {
        items.push(Profile::load(&deps.storage, &Uint128::new(i))?);
    }

    Ok(QueryAnswer::Profiles { profiles: items })
}

pub fn total_assemblies<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Total {
        total: ID::assembly(&deps.storage)?.checked_add(Uint128::new(1))?,
    })
}

pub fn assemblies<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start: Uint128,
    end: Uint128,
) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let mut end = end;
    let total = ID::assembly(&deps.storage)?;

    if start > total {
        return Err(StdError::generic_err("Assembly not found"));
    }

    if end > total {
        end = total;
    }

    for i in start.u128()..=end.u128() {
        items.push(Assembly::load(&deps.storage, &Uint128::new(i))?);
    }

    Ok(QueryAnswer::Assemblies { assemblies: items })
}

pub fn total_assembly_msgs<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Total {
        total: ID::assembly_msg(&deps.storage)?.checked_add(Uint128::new(1))?,
    })
}

pub fn assembly_msgs<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start: Uint128,
    end: Uint128,
) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let mut end = end;
    let total = ID::assembly_msg(&deps.storage)?;

    if start > total {
        return Err(StdError::generic_err("AssemblyMsg not found"));
    }

    if end > total {
        end = total;
    }

    for i in start.u128()..=end.u128() {
        items.push(AssemblyMsg::load(&deps.storage, &Uint128::new(i))?);
    }

    Ok(QueryAnswer::AssemblyMsgs { msgs: items })
}

pub fn total_contracts<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Total {
        total: ID::contract(&deps.storage)?.checked_add(Uint128::new(1))?,
    })
}

pub fn contracts<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    start: Uint128,
    end: Uint128,
) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let mut end = end;
    let total = ID::contract(&deps.storage)?;

    if start > total {
        return Err(StdError::generic_err("Contract not found"));
    }

    if end > total {
        end = total;
    }

    for i in start.u128()..=end.u128() {
        items.push(AllowedContract::load(&deps.storage, &Uint128::new(i))?);
    }

    Ok(QueryAnswer::Contracts { contracts: items })
}

pub fn user_proposals<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    user: HumanAddr,
    pagination: Pagination,
) -> StdResult<QueryAnswer> {
    let total = UserID::total_proposals(&deps.storage, user.clone())?;

    let start = pagination.amount.checked_mul(pagination.page).unwrap();
    let mut props = vec![];

    for i in start..start+pagination.amount {
        let id = match UserID::proposal(&deps.storage, user.clone(), Uint128::new(i as u128)) {
            Ok(id) => id,
            Err(_) => break
        };

        props.push(ResponseWithID {
            prop_id: id,
            data: Proposal::load(&deps.storage, &id)?
        });
    }

    Ok(QueryAnswer::UserProposals{ props , total })
}

pub fn user_assembly_votes<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    user: HumanAddr,
    pagination: Pagination,
) -> StdResult<QueryAnswer> {
    let total = UserID::total_assembly_votes(&deps.storage, user.clone())?;

    let start = pagination.amount.checked_mul(pagination.page).unwrap();
    let mut votes = vec![];

    for i in start..start+pagination.amount {
        let id = match UserID::assembly_vote(&deps.storage, user.clone(), Uint128::new(i as u128)) {
            Ok(id) => id,
            Err(_) => break
        };

        votes.push(ResponseWithID {
            prop_id: id,
            data: Proposal::assembly_vote(&deps.storage, &id, &user)?.unwrap()
        });
    }

    Ok(QueryAnswer::UserAssemblyVotes{ votes, total  })
}

pub fn user_funding<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    user: HumanAddr,
    pagination: Pagination,
) -> StdResult<QueryAnswer> {
    let total = UserID::total_funding(&deps.storage, user.clone())?;

    let start = pagination.amount.checked_mul(pagination.page).unwrap();
    let mut funds = vec![];

    for i in start..start+pagination.amount {
        let id = match UserID::funding(&deps.storage, user.clone(), Uint128::new(i as u128)) {
            Ok(id) => id,
            Err(_) => break
        };

        funds.push(ResponseWithID {
            prop_id: id,
            data: Proposal::funding(&deps.storage, &id, &user)?
        });
    }

    Ok(QueryAnswer::UserFundingVotes{ funds, total  })
}

pub fn user_votes<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    user: HumanAddr,
    pagination: Pagination,
) -> StdResult<QueryAnswer> {
    let total = UserID::total_votes(&deps.storage, user.clone())?;

    let start = pagination.amount.checked_mul(pagination.page).unwrap();
    let mut votes = vec![];

    for i in start..start+pagination.amount {
        let id = match UserID::votes(&deps.storage, user.clone(), Uint128::new(i as u128)) {
            Ok(id) => id,
            Err(_) => break
        };

        votes.push(ResponseWithID {
            prop_id: id,
            data: Proposal::public_vote(&deps.storage, &id, &user)?.unwrap()
        });
    }

    Ok(QueryAnswer::UserVotes{ votes, total  })
}