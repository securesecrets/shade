use shade_protocol::{
    c_std::{Addr, ContractInfo, StdError, StdResult, Uint256},
    liquidity_book::staking::ExecuteMsg,
    multi_test::App,
    utils::ExecuteCallback,
};

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
