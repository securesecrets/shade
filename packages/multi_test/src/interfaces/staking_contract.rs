use shade_protocol::{
    c_std::{Addr, ContractInfo, StdError, StdResult, Uint256},
    liquidity_book::staking::{ExecuteMsg, Liquidity, QueryAnswer, QueryMsg},
    multi_test::App,
    utils::{ExecuteCallback, Query},
};

pub fn set_viewing_key(
    app: &mut App,
    sender: &str,
    lb_staking: &ContractInfo,
    key: String,
) -> StdResult<()> {
    match (ExecuteMsg::SetViewingKey { key }.test_exec(
        lb_staking,
        app,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    }
}

pub fn unstaking(
    app: &mut App,
    sender: &str,
    lb_staking: &ContractInfo,
    ids: Vec<u32>,
    amounts: Vec<Uint256>,
) -> StdResult<()> {
    match (ExecuteMsg::Unstake {
        token_ids: ids,
        amounts,
    }
    .test_exec(lb_staking, app, Addr::unchecked(sender), &[]))
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    }
}

pub fn query_liquidity(
    app: &App,
    sender: &Addr,
    key: String,
    lb_staking: &ContractInfo,
    ids: Vec<u32>,
    round_index: Option<u64>,
) -> StdResult<Vec<Liquidity>> {
    let res: QueryAnswer = QueryMsg::Liquidity {
        owner: sender.clone(),
        key,
        round_index,
        token_ids: ids,
    }
    .test_query(&lb_staking, app)?;
    match res {
        QueryAnswer::Liquidity(liq) => Ok(liq),
        _ => Err(StdError::generic_err("Query failed")),
    }
}
