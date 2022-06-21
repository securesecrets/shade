use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdError, StdResult, Storage};
use shade_protocol::contract_interfaces::query_auth::{Admin, ContractStatus, QueryAnswer, QueryPermit};
use shade_protocol::contract_interfaces::query_auth::auth::{Key, PermitKey};
use shade_protocol::utils::storage::plus::{ItemStorage, MapStorage};

pub fn config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::Config {
        admin: Admin::load(&deps.storage)?.0,
        state: ContractStatus::load(&deps.storage)?
    })
}

pub fn validate_vk<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    user: HumanAddr,
    key: String
) -> StdResult<QueryAnswer> {

    Ok(QueryAnswer::ValidateViewingKey {
        is_valid: Key::verify(&deps.storage, user, key)?
    })
}

pub fn validate_permit<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    permit: QueryPermit
) -> StdResult<QueryAnswer> {

    let user = permit.validate(None)?.as_humanaddr(&deps.api)?;

    Ok(QueryAnswer::ValidatePermit {
        user: user.clone(),
        is_revoked: PermitKey::may_load(
            &deps.storage,
            (user, permit.params.key),
        )?.is_some()
    })
}