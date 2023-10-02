use crate::multi::lb_pair::LbPair;
use shade_protocol::{
    c_std::{Addr, ContractInfo, StdError, StdResult, Uint128},
    contract_interfaces::liquidity_book::lb_pair,
    lb_libraries::{
        tokens::TokenType,
        types::{ContractInstantiationInfo, StaticFeeParameters},
    },
    liquidity_book::lb_pair::{LiquidityParameters, RemoveLiquidity},
    multi_test::App,
    utils::{asset::Contract, ExecuteCallback, InstantiateCallback, MultiTestable, Query},
};

pub fn init(
    app: &mut App,
    sender: &str,
    factory_contract_info: ContractInfo,
    token_x: TokenType,
    token_y: TokenType,
    bin_step: u16,
    pair_parameters: StaticFeeParameters,
    active_id: u32,
    lb_token_implementation: ContractInstantiationInfo,
    viewing_key: String,
    pair_name: String,
    entropy: String,
) -> StdResult<Contract> {
    let lb_pair = Contract::from(
        match (lb_pair::InstantiateMsg {
            factory: factory_contract_info,
            token_x,
            token_y,
            bin_step,
            pair_parameters,
            active_id,
            lb_token_implementation,
            viewing_key,
            pair_name,
            entropy,
        }
        .test_init(
            LbPair::default(),
            app,
            Addr::unchecked(sender),
            "snip20",
            &[],
        )) {
            Ok(contract_info) => contract_info,
            Err(e) => return Err(StdError::generic_err(e.to_string())),
        },
    );
    Ok(lb_pair)
}

pub fn add_liquidity(
    app: &mut App,
    sender: &str,
    lb_pair: &ContractInfo,
    liquidity_parameters: LiquidityParameters,
) -> StdResult<()> {
    match (lb_pair::ExecuteMsg::AddLiquidity {
        liquidity_parameters,
    }
    .test_exec(lb_pair, app, Addr::unchecked(sender), &[]))
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.to_string())),
    }
}

pub fn remove_liquidity(
    app: &mut App,
    sender: &str,
    lb_pair: &ContractInfo,
    liquidity_parameters: RemoveLiquidity,
) -> StdResult<()> {
    match (lb_pair::ExecuteMsg::RemoveLiquidity {
        remove_liquidity_params: liquidity_parameters,
    }
    .test_exec(lb_pair, app, Addr::unchecked(sender), &[]))
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.to_string())),
    }
}

pub fn swap(
    app: &mut App,
    sender: &str,
    lb_pair: &ContractInfo,
    liquidity_parameters: LiquidityParameters,
    swap_for_y: bool,
    to: Addr,
    amount_received: Uint128,
) -> StdResult<()> {
    match (lb_pair::ExecuteMsg::Swap {
        swap_for_y,
        to,
        amount_received,
    }
    .test_exec(lb_pair, app, Addr::unchecked(sender), &[]))
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.to_string())),
    }
}

pub fn lb_token_query(chain: &App, lb_pair: &ContractInfo) -> StdResult<ContractInfo> {
    let res = lb_pair::QueryMsg::GetLbToken {}.test_query(lb_pair, chain)?;
    let lb_pair::LbTokenResponse { lb_token } = res;
    Ok(lb_token)
}

pub fn bin_query(chain: &App, lb_pair: &ContractInfo, id: u32) -> StdResult<(u128, u128)> {
    let res = lb_pair::QueryMsg::GetBin { id }.test_query(lb_pair, chain)?;
    let lb_pair::BinResponse {
        bin_reserve_x,
        bin_reserve_y,
    } = res;
    Ok((bin_reserve_x, bin_reserve_y))
}

pub fn swap_in_query(
    chain: &App,
    lb_pair: &ContractInfo,
    amount_out: Uint128,
    swap_for_y: bool,
) -> StdResult<(Uint128, Uint128, Uint128)> {
    let res = lb_pair::QueryMsg::GetSwapIn {
        amount_out,
        swap_for_y,
    }
    .test_query(lb_pair, chain)?;
    let lb_pair::SwapInResponse {
        amount_in,
        amount_out_left,
        fee,
    } = res;
    Ok((amount_in, amount_out_left, fee))
}

pub fn swap_out_query(
    chain: &App,
    lb_pair: &ContractInfo,
    amount_in: Uint128,
    swap_for_y: bool,
) -> StdResult<(Uint128, Uint128, Uint128)> {
    let res = lb_pair::QueryMsg::GetSwapOut {
        swap_for_y,
        amount_in,
    }
    .test_query(lb_pair, chain)?;
    let lb_pair::SwapOutResponse {
        amount_out,
        amount_in_left,
        fee,
    } = res;
    Ok((amount_out, amount_in_left, fee))
}
