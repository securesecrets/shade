use cosmwasm_std::{Addr, from_binary, QuerierWrapper, StdError, StdResult};
use serde::de::DeserializeOwned;
use crate::{Contract, query_auth};
use crate::query_auth::QueryPermit;
use crate::utils::Query;

pub struct PermitAuthentication<T: DeserializeOwned> {
    pub sender: Addr,
    pub revoked: bool,
    pub data: T
}

pub fn authenticate_permit<T: DeserializeOwned>(
    permit: QueryPermit,
    querier: &QuerierWrapper,
    authenticator: Contract
) -> StdResult<PermitAuthentication<T>> {
    let res: query_auth::QueryAnswer = query_auth::QueryMsg::ValidatePermit { permit: permit.clone() }
        .query(querier, &authenticator)?;

    let sender: Addr;
    let revoked: bool;

    match res {
        query_auth::QueryAnswer::ValidatePermit { user, is_revoked } => {
            sender = user;
            revoked = is_revoked;
        }
        _ => return Err(StdError::generic_err("Wrong query response")),
    }

    Ok(PermitAuthentication {
        sender,
        revoked,
        data: from_binary(&permit.params.data)?
    })
}

pub fn authenticate_vk(
    address: Addr,
    key: String,
    querier: &QuerierWrapper,
    authenticator: &Contract
) -> StdResult<bool> {
    let res: query_auth::QueryAnswer = query_auth::QueryMsg::ValidateViewingKey {
        user: address,
        key,
    }.query(querier, authenticator)?;

    match res {
        query_auth::QueryAnswer::ValidateViewingKey { is_valid } => {
            Ok(is_valid)
        }
        _ => Err(StdError::generic_err("Unauthorized")),
    }
}