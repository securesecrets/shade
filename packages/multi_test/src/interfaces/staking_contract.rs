use shade_protocol::{
    c_std::{Addr, ContractInfo, StdError, StdResult, Uint256},
    cosmwasm_schema::cw_serde,
    liquidity_book::staking::{ExecuteMsg, Liquidity, OwnerBalance, QueryAnswer, QueryMsg},
    multi_test::App,
    utils::{asset::RawContract, ExecuteCallback, Query},
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

pub fn update_config(
    app: &mut App,
    sender: &str,
    lb_staking: &ContractInfo,
    admin_auth: Option<RawContract>,
    query_auth: Option<RawContract>,
    epoch_duration: Option<u64>,
    expiry_duration: Option<u64>,
) -> StdResult<()> {
    match (ExecuteMsg::UpdateConfig {
        admin_auth,
        query_auth,
        epoch_duration,
        expiry_duration,
    }
    .test_exec(lb_staking, app, Addr::unchecked(sender), &[]))
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    }
}

pub fn claim_rewards(app: &mut App, sender: &str, lb_staking: &ContractInfo) -> StdResult<()> {
    match (ExecuteMsg::ClaimRewards {}.test_exec(lb_staking, app, Addr::unchecked(sender), &[])) {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    }
}

pub fn register_reward_tokens(
    app: &mut App,
    sender: &str,
    lb_staking: &ContractInfo,
    tokens: Vec<ContractInfo>,
) -> StdResult<()> {
    match ExecuteMsg::RegisterRewardTokens(tokens).test_exec(
        lb_staking,
        app,
        Addr::unchecked(sender),
        &[],
    ) {
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

pub fn query_registered_tokens(
    app: &App,
    lb_staking: &ContractInfo,
) -> StdResult<Vec<ContractInfo>> {
    let res: QueryAnswer = QueryMsg::RegisteredTokens {}.test_query(&lb_staking, app)?;
    match res {
        QueryAnswer::RegisteredTokens(liq) => Ok(liq),
        _ => Err(StdError::generic_err("Query failed")),
    }
}

pub fn query_id_total_balance(app: &App, lb_staking: &ContractInfo, id: u32) -> StdResult<Uint256> {
    let res: QueryAnswer =
        QueryMsg::IdTotalBalance { id: id.to_string() }.test_query(&lb_staking, app)?;
    match res {
        QueryAnswer::IdTotalBalance { amount } => Ok(amount),
        _ => Err(StdError::generic_err("Query failed")),
    }
}

pub fn query_balance(
    app: &App,
    lb_staking: &ContractInfo,
    staker: Addr,
    key: String,
    id: u32,
) -> StdResult<Uint256> {
    let res: QueryAnswer = QueryMsg::Balance {
        owner: staker,
        key,
        token_id: id.to_string(),
    }
    .test_query(&lb_staking, app)?;
    match res {
        QueryAnswer::Balance { amount } => Ok(amount),
        _ => Err(StdError::generic_err("Query failed")),
    }
}

pub fn query_all_balances(
    app: &App,
    lb_staking: &ContractInfo,
    staker: Addr,
    key: String,
    page: Option<u32>,
    page_size: Option<u32>,
) -> StdResult<Vec<OwnerBalance>> {
    let res: QueryAnswer = QueryMsg::AllBalances {
        owner: staker,
        key,
        page,
        page_size,
    }
    .test_query(&lb_staking, app)?;
    match res {
        QueryAnswer::AllBalances(balances) => Ok(balances),
        _ => Err(StdError::generic_err("Query failed")),
    }
}

pub fn query_config(app: &App, lb_staking: &ContractInfo) -> StdResult<Config> {
    let res: QueryAnswer = QueryMsg::ContractInfo {}.test_query(&lb_staking, app)?;
    match res {
        QueryAnswer::ContractInfo {
            lb_token,
            lb_pair,
            admin_auth,
            query_auth,
            epoch_index,
            epoch_durations,
            expiry_durations,
        } => Ok((Config {
            lb_token,
            lb_pair,
            admin_auth,
            query_auth,
            epoch_index,
            epoch_durations,
            expiry_durations,
        })),
        _ => Err(StdError::generic_err("Query failed")),
    }
}

pub struct Config {
    pub lb_token: ContractInfo,
    pub lb_pair: Addr,
    pub admin_auth: ContractInfo,
    pub query_auth: Option<ContractInfo>,
    pub epoch_index: u64,
    pub epoch_durations: u64,
    pub expiry_durations: Option<u64>,
}
