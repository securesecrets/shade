use cosmwasm_std::{Api, Extern, Querier, StdError, StdResult, Storage};
use cosmwasm_math_compat::Uint128;
use shade_protocol::governance::assembly::{Assembly, AssemblyMsg};
use shade_protocol::governance::contract::AllowedContract;
use shade_protocol::governance::profile::Profile;
use shade_protocol::governance::proposal::Proposal;
use shade_protocol::governance::QueryAnswer;
use shade_protocol::governance::stored_id::ID;

pub fn proposals<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, start: Uint128, end: Uint128) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let mut end = end;
    let total = ID::proposal(&deps.storage)?;

    if start > total {
        return Err(StdError::not_found(Proposal))
    }

    if end > total {
        end = total;
    }

    for i in start.u128()..end.u128() {
        items.push(Proposal::load(&deps.storage, &Uint128::new(i))?);
    }

    Ok(QueryAnswer::Proposals {
        props: items
    })
}

pub fn profiles<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, start: Uint128, end: Uint128) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let mut end = end;
    let total = ID::profile(&deps.storage)?;

    if start > total {
        return Err(StdError::not_found(Proposal))
    }

    if end > total {
        end = total;
    }

    for i in start.u128()..end.u128() {
        items.push(Profile::load(&deps.storage, &Uint128::new(i))?);
    }

    Ok(QueryAnswer::Profiles {
        profiles: items
    })
}

pub fn assemblies<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, start: Uint128, end: Uint128) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let mut end = end;
    let total = ID::assembly(&deps.storage)?;

    if start > total {
        return Err(StdError::not_found(Proposal))
    }

    if end > total {
        end = total;
    }

    for i in start.u128()..end.u128() {
        items.push(Assembly::load(&deps.storage, &Uint128::new(i))?);
    }

    Ok(QueryAnswer::Assemblies {
        assemblies: items
    })
}

pub fn assembly_msgs<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, start: Uint128, end: Uint128) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let mut end = end;
    let total = ID::assembly_msg(&deps.storage)?;

    if start > total {
        return Err(StdError::not_found(Proposal))
    }

    if end > total {
        end = total;
    }

    for i in start.u128()..end.u128() {
        items.push(AssemblyMsg::load(&deps.storage, &Uint128::new(i))?);
    }

    Ok(QueryAnswer::AssemblyMsgs {
        msgs: items,
    })
}

pub fn contracts<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, start: Uint128, end: Uint128) -> StdResult<QueryAnswer> {
    let mut items = vec![];
    let mut end = end;
    let total = ID::contract(&deps.storage)?;

    if start > total {
        return Err(StdError::not_found(Proposal))
    }

    if end > total {
        end = total;
    }

    for i in start.u128()..end.u128() {
        items.push(AllowedContract::load(&deps.storage, &Uint128::new(i))?);
    }

    Ok(QueryAnswer::Contracts {
        contracts: items,
    })
}