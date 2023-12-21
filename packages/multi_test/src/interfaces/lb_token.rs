use shade_protocol::{
    c_std::{Addr, ContractInfo, StdError, StdResult, Uint256},
    contract_interfaces::liquidity_book::{lb_token, lb_token::*},
    lb_libraries::lb_token::state_structs::OwnerBalance,
    multi_test::App,
    utils::{ExecuteCallback, Query},
};

pub fn set_viewing_key(
    app: &mut App,
    sender: &str,
    lb_token: &ContractInfo,
    key: String,
) -> StdResult<()> {
    match (lb_token::ExecuteMsg::SetViewingKey { key, padding: None }.test_exec(
        lb_token,
        app,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    }
}

pub fn batch_send(
    app: &mut App,
    sender: &str,
    lb_token: &ContractInfo,
    actions: Vec<SendAction>,
) -> StdResult<()> {
    match (lb_token::ExecuteMsg::BatchSend {
        actions,
        padding: None,
    }
    .test_exec(lb_token, app, Addr::unchecked(sender), &[]))
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    }
}

pub fn query_contract_info(app: &App, info: &ContractInfo) -> StdResult<QueryAnswer> {
    let res: QueryAnswer = QueryMsg::TokenContractInfo {}.test_query(&info, app)?;
    match res {
        QueryAnswer::TokenContractInfo { .. } => Ok(res),
        _ => Err(StdError::generic_err("Query failed")),
    }
}

pub fn query_id_balance(app: &App, info: &ContractInfo, id: String) -> StdResult<QueryAnswer> {
    let res: QueryAnswer = QueryMsg::IdTotalBalance { id }.test_query(&info, app)?;
    match res {
        QueryAnswer::IdTotalBalance { .. } => Ok(res),
        _ => Err(StdError::generic_err("Query failed")),
    }
}

pub fn query_balance(
    app: &App,
    info: &ContractInfo,
    owner: Addr,
    viewer: Addr,
    key: String,
    token_id: String,
) -> StdResult<Uint256> {
    let res: QueryAnswer = QueryMsg::Balance {
        owner,
        viewer,
        key,
        token_id,
    }
    .test_query(&info, app)?;
    match res {
        QueryAnswer::Balance { amount } => Ok(amount),
        _ => Err(StdError::generic_err("Query failed")),
    }
}

pub fn query_all_balances(
    app: &App,
    info: &ContractInfo,
    owner: Addr,
    key: String,
) -> StdResult<Vec<OwnerBalance>> {
    let res: QueryAnswer = QueryMsg::AllBalances {
        owner,
        key,
        tx_history_page: None,
        tx_history_page_size: None,
    }
    .test_query(&info, app)?;
    match res {
        QueryAnswer::AllBalances(owner_balance) => Ok(owner_balance),
        _ => Err(StdError::generic_err("Query failed")),
    }
}
