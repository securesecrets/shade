use cosmwasm_std::{Api, Extern, Querier, StdError, StdResult, Storage};
use cosmwasm_math_compat::Uint128;
use shade_protocol::governance::assembly::{Assembly, AssemblyMsg};
use shade_protocol::governance::contract::AllowedContract;
use shade_protocol::governance::profile::Profile;
use shade_protocol::governance::proposal::Proposal;
use shade_protocol::governance::{Config, QueryAnswer};
use shade_protocol::governance::stored_id::ID;
use shade_protocol::utils::storage::SingletonStorage;

pub fn config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::Config {
        config: Config::load(&deps.storage)?
    })
}

pub fn total_proposals<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::Total {
        total: ID::proposal(&deps.storage)?.checked_add(Uint128::new(1))?
    })
}

pub fn proposals<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, start: Uint128, end: Uint128) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let mut end = end;
    let total = ID::proposal(&deps.storage)?;

    if start > total {
        return Err(StdError::generic_err("Proposal not found"))
    }

    if end > total {
        end = total;
    }

    for i in start.u128()..=end.u128() {
        items.push(Proposal::load(&deps.storage, &Uint128::new(i))?);
    }

    Ok(QueryAnswer::Proposals {
        props: items
    })
}

pub fn total_profiles<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::Total {
        total: ID::profile(&deps.storage)?.checked_add(Uint128::new(1))?
    })
}

pub fn profiles<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, start: Uint128, end: Uint128) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let mut end = end;
    let total = ID::profile(&deps.storage)?;

    if start > total {
        return Err(StdError::generic_err("Profile not found"))
    }

    if end > total {
        end = total;
    }

    for i in start.u128()..=end.u128() {
        items.push(Profile::load(&deps.storage, &Uint128::new(i))?);
    }

    Ok(QueryAnswer::Profiles {
        profiles: items
    })
}

pub fn total_assemblies<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::Total {
        total: ID::assembly(&deps.storage)?.checked_add(Uint128::new(1))?
    })
}

pub fn assemblies<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, start: Uint128, end: Uint128) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let mut end = end;
    let total = ID::assembly(&deps.storage)?;

    if start > total {
        return Err(StdError::generic_err("Assembly not found"))
    }

    if end > total {
        end = total;
    }

    for i in start.u128()..=end.u128() {
        items.push(Assembly::load(&deps.storage, &Uint128::new(i))?);
    }

    Ok(QueryAnswer::Assemblies {
        assemblies: items
    })
}

pub fn total_assembly_msgs<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::Total {
        total: ID::assembly_msg(&deps.storage)?.checked_add(Uint128::new(1))?
    })
}

pub fn assembly_msgs<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, start: Uint128, end: Uint128) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let mut end = end;
    let total = ID::assembly_msg(&deps.storage)?;

    if start > total {
        return Err(StdError::generic_err("AssemblyMsg not found"))
    }

    if end > total {
        end = total;
    }

    for i in start.u128()..=end.u128() {
        items.push(AssemblyMsg::load(&deps.storage, &Uint128::new(i))?);
    }

    Ok(QueryAnswer::AssemblyMsgs {
        msgs: items,
    })
}

pub fn total_contracts<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::Total {
        total: ID::contract(&deps.storage)?.checked_add(Uint128::new(1))?
    })
}

pub fn contracts<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, start: Uint128, end: Uint128) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let mut end = end;
    let total = ID::contract(&deps.storage)?;

    if start > total {
        return Err(StdError::generic_err("Contract not found"))
    }

    if end > total {
        end = total;
    }

    for i in start.u128()..=end.u128() {
        items.push(AllowedContract::load(&deps.storage, &Uint128::new(i))?);
    }

    Ok(QueryAnswer::Contracts {
        contracts: items,
    })
}