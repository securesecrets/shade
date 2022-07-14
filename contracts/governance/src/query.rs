use shade_protocol::c_std::Uint128;
use shade_protocol::c_std::{Api, DepsMut, Querier, StdError, StdResult, Storage};
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

pub fn config(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        config: Config::load(&deps.storage)?,
    })
}

pub fn total_proposals(
    deps: Deps,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Total {
        total: ID::proposal(&deps.storage)?.checked_add(Uint128::new(1))?,
    })
}

pub fn proposals(
    deps: Deps,
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

pub fn total_profiles(
    deps: Deps,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Total {
        total: ID::profile(&deps.storage)?.checked_add(Uint128::new(1))?,
    })
}

pub fn profiles(
    deps: Deps,
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

pub fn total_assemblies(
    deps: Deps,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Total {
        total: ID::assembly(&deps.storage)?.checked_add(Uint128::new(1))?,
    })
}

pub fn assemblies(
    deps: Deps,
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

pub fn total_assembly_msgs(
    deps: Deps,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Total {
        total: ID::assembly_msg(&deps.storage)?.checked_add(Uint128::new(1))?,
    })
}

pub fn assembly_msgs(
    deps: Deps,
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

pub fn total_contracts(
    deps: Deps,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Total {
        total: ID::contract(&deps.storage)?.checked_add(Uint128::new(1))?,
    })
}

pub fn contracts(
    deps: Deps,
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
