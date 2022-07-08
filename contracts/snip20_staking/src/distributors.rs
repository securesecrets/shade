use crate::{
    contract::check_if_admin,
    msg::{HandleAnswer, QueryAnswer, ResponseStatus::Success},
    state::Config,
    state_staking::{Distributors, DistributorsEnabled},
};
use shade_protocol::c_std::{
    to_binary,
    Api,
    Binary,
    Env,
    Extern,
    HandleResponse,
    HumanAddr,
    Querier,
    StdResult,
    Storage,
};
use shade_protocol::utils::storage::default::SingletonStorage;

pub fn get_distributor<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Option<Vec<HumanAddr>>> {
    Ok(match DistributorsEnabled::load(&deps.storage)?.0 {
        true => Some(Distributors::load(&deps.storage)?.0),
        false => None,
    })
}

pub fn try_set_distributors_status<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    enabled: bool,
) -> StdResult<HandleResponse> {
    let config = Config::from_storage(&mut deps.storage);

    check_if_admin(&config, &env.message.sender)?;

    DistributorsEnabled(enabled).save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetDistributorsStatus {
            status: Success,
        })?),
    })
}

pub fn try_add_distributors<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    new_distributors: Vec<HumanAddr>,
) -> StdResult<HandleResponse> {
    let config = Config::from_storage(&mut deps.storage);

    check_if_admin(&config, &env.message.sender)?;

    let mut distributors = Distributors::load(&deps.storage)?;
    distributors.0.extend(new_distributors);
    distributors.save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::AddDistributors {
            status: Success,
        })?),
    })
}

pub fn try_set_distributors<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    distributors: Vec<HumanAddr>,
) -> StdResult<HandleResponse> {
    let config = Config::from_storage(&mut deps.storage);

    check_if_admin(&config, &env.message.sender)?;

    Distributors(distributors).save(&mut deps.storage)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&HandleAnswer::SetDistributors {
            status: Success,
        })?),
    })
}

pub fn distributors<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    to_binary(&QueryAnswer::Distributors {
        distributors: match DistributorsEnabled::load(&deps.storage)?.0 {
            true => Some(Distributors::load(&deps.storage)?.0),
            false => None,
        },
    })
}
