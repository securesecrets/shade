use cosmwasm_std::{from_binary, Addr, Deps, QuerierWrapper, StdError, StdResult};
use serde::de::DeserializeOwned;
use shade_protocol::{
    query_auth::{self, helpers::PermitAuthentication, QueryPermit},
    utils::Query,
    Contract,
};

pub fn authenticate_permit<T: DeserializeOwned>(
    deps: Deps,
    permit: QueryPermit,
    querier: &QuerierWrapper,
    authenticator: Option<Contract>,
) -> StdResult<PermitAuthentication<T>> {
    let sender: Addr;
    let revoked: bool;
    match authenticator {
        Some(a) => {
            let res: query_auth::QueryAnswer = query_auth::QueryMsg::ValidatePermit {
                permit: permit.clone(),
            }
            .query(querier, &a)?;

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
                data: from_binary(&permit.params.data)?,
            })
        }
        None => {
            sender = permit.validate(deps.api, None)?.as_addr(None)?;
            Ok(PermitAuthentication {
                sender,
                revoked: false,
                data: from_binary(&permit.params.data)?,
            })
        }
    }
}
