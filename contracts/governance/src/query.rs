use cosmwasm_std::{Api, Extern, Querier, StdResult, Storage};
use shade_protocol::governance::QueryAnswer;

pub fn <S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<QueryAnswer> {
    todo!()
    // Ok(QueryAnswer:: {
    //
    // })
}