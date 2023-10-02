use shade_protocol::{
    c_std::{ContractInfo, StdError, StdResult},
    contract_interfaces::liquidity_book::lb_token::*,
    multi_test::App,
    utils::Query,
};

pub fn contract_info_query(chain: &App, info: &ContractInfo) -> StdResult<QueryAnswer> {
    let res: QueryAnswer = QueryMsg::TokenContractInfo {}.test_query(&info, chain)?;
    match res {
        QueryAnswer::TokenContractInfo { .. } => Ok(res),
        _ => Err(StdError::generic_err("Query failed")),
    }
}

pub fn id_balance_query(chain: &App, info: &ContractInfo, id: String) -> StdResult<QueryAnswer> {
    let res: QueryAnswer = QueryMsg::IdTotalBalance { id }.test_query(&info, chain)?;
    match res {
        QueryAnswer::IdTotalBalance { .. } => Ok(res),
        _ => Err(StdError::generic_err("Query failed")),
    }
}
