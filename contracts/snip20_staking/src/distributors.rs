use crate::{
    contract::check_if_admin,
    msg::{HandleAnswer, QueryAnswer, ResponseStatus::Success},
    state::Config,
    state_staking::{Distributors, DistributorsEnabled},
};
use cosmwasm_std::{Deps, MessageInfo};
use shade_protocol::c_std::{
    to_binary,
    Api,
    Binary,
    Env,
    DepsMut,
    Response,
    Addr,
    Querier,
    StdResult,
    Storage,
};
use shade_protocol::utils::storage::default::SingletonStorage;

pub fn get_distributor(
    deps: Deps,
) -> StdResult<Option<Vec<Addr>>> {
    Ok(match DistributorsEnabled::load(deps.storage)?.0 {
        true => Some(Distributors::load(deps.storage)?.0),
        false => None,
    })
}

pub fn try_set_distributors_status(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    enabled: bool,
) -> StdResult<Response> {
    let config = Config::from_storage(deps.storage);

    check_if_admin(&config, &info.sender)?;

    DistributorsEnabled(enabled).save(deps.storage)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetDistributorsStatus {
            status: Success,
        })?))
}

pub fn try_add_distributors(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_distributors: Vec<Addr>,
) -> StdResult<Response> {
    let config = Config::from_storage(deps.storage);

    check_if_admin(&config, &info.sender)?;

    let mut distributors = Distributors::load(deps.storage)?;
    distributors.0.extend(new_distributors);
    distributors.save(deps.storage)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::AddDistributors {
            status: Success,
        })?))
}

pub fn try_set_distributors(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    distributors: Vec<Addr>,
) -> StdResult<Response> {
    let config = Config::from_storage(deps.storage);

    check_if_admin(&config, &info.sender)?;

    Distributors(distributors).save(deps.storage)?;

    Ok(Response::new().set_data(to_binary(&HandleAnswer::SetDistributors {
            status: Success,
        })?))
}

pub fn distributors(deps: Deps) -> StdResult<Binary> {
    to_binary(&QueryAnswer::Distributors {
        distributors: match DistributorsEnabled::load(deps.storage)?.0 {
            true => Some(Distributors::load(deps.storage)?.0),
            false => None,
        },
    })
}
