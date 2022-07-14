use shade_protocol::c_std::{Api, DepsMut, Addr, Querier, StdResult, Storage};
use shade_protocol::{
    contract_interfaces::query_auth::{
        auth::{Key, PermitKey},
        Admin,
        ContractStatus,
        QueryAnswer,
        QueryPermit,
    },
    utils::storage::plus::{ItemStorage, MapStorage},
};

pub fn config<S: Storage, A: Api, Q: Querier>(deps: Deps) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::Config {
        admin: Admin::load(&deps.storage)?.0,
        state: ContractStatus::load(&deps.storage)?,
    })
}

pub fn validate_vk<S: Storage, A: Api, Q: Querier>(
    deps: Deps,
    user: Addr,
    key: String,
) -> StdResult<QueryAnswer> {
    Ok(QueryAnswer::ValidateViewingKey {
        is_valid: Key::verify(&deps.storage, user, key)?,
    })
}

pub fn validate_permit<S: Storage, A: Api, Q: Querier>(
    deps: Deps,
    permit: QueryPermit,
) -> StdResult<QueryAnswer> {
    let user = permit.validate(&deps.api, None)?.as_Addr(None)?;

    Ok(QueryAnswer::ValidatePermit {
        user: user.clone(),
        is_revoked: PermitKey::may_load(&deps.storage, (user, permit.params.key))?.is_some(),
    })
}