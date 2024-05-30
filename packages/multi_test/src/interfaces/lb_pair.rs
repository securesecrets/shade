use std::str::FromStr;

use crate::multi::lb_pair::LbPair;
use lb_libraries::types::{ContractImplementation, StaticFeeParameters};
use shade_protocol::{
    c_std::{to_binary, Addr, Coin, ContractInfo, StdError, StdResult, Uint128, Uint256},
    contract_interfaces::{liquidity_book::lb_pair, snip20},
    liquidity_book::lb_pair::{
        BinResponse, ContractStatus, LiquidityParameters, OracleSampleResponse,
        OracleSamplesAtResponse, RemoveLiquidity, RewardsDistribution,
        RewardsDistributionAlgorithm,
    },
    multi_test::App,
    swap::core::{TokenAmount, TokenType},
    utils::{
        asset::{Contract, RawContract},
        ExecuteCallback, InstantiateCallback, MultiTestable, Query,
    },
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
    lb_token_implementation: ContractImplementation,
    staking_contract_implementation: ContractImplementation,
    viewing_key: String,
    _pair_name: String,
    entropy: String,
    protocol_fee_recipient: Addr,
    admin_auth: RawContract,
    query_auth: RawContract,

    total_reward_bins: u32,
    rewards_distribution_algorithm: Option<RewardsDistributionAlgorithm>,
    epoch_staking_index: u64,
    epoch_staking_duration: u64,
    expiry_staking_duration: Option<u64>,
    recover_staking_funds_receiver: Addr,
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
            entropy,
            protocol_fee_recipient,
            admin_auth,
            query_auth,
            total_reward_bins: Some(total_reward_bins),
            rewards_distribution_algorithm: rewards_distribution_algorithm
                .unwrap_or(RewardsDistributionAlgorithm::TimeBasedRewards),
            staking_contract_implementation,
            epoch_staking_index,
            epoch_staking_duration,
            expiry_staking_duration,
            recover_staking_funds_receiver,
            max_bins_per_swap: Some(500),
        }
        .test_init(
            LbPair::default(),
            app,
            Addr::unchecked(sender),
            "lb_pair",
            &[],
        )) {
            Ok(contract_info) => contract_info,
            Err(e) => return Err(StdError::generic_err(e.to_string())),
        },
    );
    Ok(lb_pair)
}

pub fn set_contract_status(
    app: &mut App,
    sender: &str,
    lb_pair: &ContractInfo,
    contract_status: ContractStatus,
) -> StdResult<()> {
    match (lb_pair::ExecuteMsg::SetContractStatus { contract_status }.test_exec(
        lb_pair,
        app,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    }
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
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    }
}

pub fn add_native_liquidity(
    app: &mut App,
    sender: &str,
    lb_pair: &ContractInfo,
    liquidity_parameters: LiquidityParameters,
    native_funds: Vec<Coin>,
) -> StdResult<()> {
    match (lb_pair::ExecuteMsg::AddLiquidity {
        liquidity_parameters,
    }
    .test_exec(lb_pair, app, Addr::unchecked(sender), &native_funds))
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
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
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    }
}
pub fn swap_native(
    app: &mut App,
    sender: &str,
    lb_pair: &ContractInfo,
    to: Option<String>,
    offer: TokenAmount,
) -> StdResult<()> {
    match (&lb_pair::ExecuteMsg::SwapTokens {
        expected_return: None,
        to,
        padding: None,
        offer,
    }
    .test_exec(&lb_pair, app, Addr::unchecked(sender), &[]))
    {
        Ok(_) => Ok(()),
        Err(e) => Err(StdError::generic_err(e.root_cause().to_string())),
    }
}

pub fn swap_snip_20(
    app: &mut App,
    sender: &str,
    lb_pair: &ContractInfo,
    to: Option<String>,
    lb_token: &ContractInfo,

    amount: Uint128,
) -> StdResult<Uint128> {
    let msg = to_binary(&lb_pair::InvokeMsg::SwapTokens {
        expected_return: None,
        to,
        padding: None,
    })?;
    match (snip20::ExecuteMsg::Send {
        amount,
        msg: Some(msg),
        memo: None,
        padding: None,
        recipient_code_hash: Some(lb_pair.code_hash.clone()),
        recipient: lb_pair.address.to_string(),
    }
    .test_exec(&lb_token, app, Addr::unchecked(sender), &[]))
    {
        Ok(res) => {
            let total_fee = res
                .events
                .iter()
                .flat_map(|event| &event.attributes)
                .find_map(|attribute| {
                    if attribute.key == "total_fee_amount" {
                        Some(attribute.value.clone()) // Clone the value to give it a 'static lifetime
                    } else {
                        None
                    }
                });

            let mut t_fee = Uint128::zero();
            match total_fee {
                Some(fee) => {
                    t_fee = Uint128::from_str(fee.as_str())?;
                }
                None => {}
            }
            Ok(t_fee)
        }
        Err(e) => Err(StdError::generic_err(e.root_cause().to_string())),
    }
}

pub fn collect_protocol_fees(app: &mut App, sender: &str, lb_pair: &ContractInfo) -> StdResult<()> {
    match (lb_pair::ExecuteMsg::CollectProtocolFees {}.test_exec(
        lb_pair,
        app,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    }
}

pub fn calculate_rewards(app: &mut App, sender: &str, lb_pair: &ContractInfo) -> StdResult<()> {
    match (lb_pair::ExecuteMsg::CalculateRewardsDistribution {}.test_exec(
        lb_pair,
        app,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    }
}

pub fn reset_rewards_epoch(
    app: &mut App,
    sender: &str,
    lb_pair: &ContractInfo,
    distribution: Option<RewardsDistributionAlgorithm>,
    base_rewards_bins: Option<u32>,
) -> StdResult<()> {
    match (lb_pair::ExecuteMsg::ResetRewardsConfig {
        distribution,
        base_rewards_bins,
    }
    .test_exec(lb_pair, app, Addr::unchecked(sender), &[]))
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    }
}

pub fn set_static_fee_parameters(
    app: &mut App,
    sender: &str,
    lb_pair: &ContractInfo,
    base_factor: u16,
    filter_period: u16,
    decay_period: u16,
    reduction_factor: u16,
    variable_fee_control: u32,
    protocol_share: u16,
    max_volatility_accumulator: u32,
) -> StdResult<()> {
    match (lb_pair::ExecuteMsg::SetStaticFeeParameters {
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
    }
    .test_exec(lb_pair, app, Addr::unchecked(sender), &[]))
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    }
}

pub fn query_total_supply(app: &App, lb_pair: &ContractInfo, id: u32) -> StdResult<Uint256> {
    let res = lb_pair::QueryMsg::TotalSupply { id }.test_query(lb_pair, app)?;
    let lb_pair::TotalSupplyResponse { total_supply } = res;
    Ok(total_supply)
}

pub fn query_lb_token(app: &App, lb_pair: &ContractInfo) -> StdResult<ContractInfo> {
    let res = lb_pair::QueryMsg::GetLbToken {}.test_query(lb_pair, app)?;
    let lb_pair::LbTokenResponse { contract: lb_token } = res;
    Ok(lb_token)
}

pub fn query_lb_staking(app: &App, lb_pair: &ContractInfo) -> StdResult<ContractInfo> {
    let res = lb_pair::QueryMsg::GetStakingContract {}.test_query(lb_pair, app)?;
    let lb_pair::StakingResponse {
        contract: staking_contract,
    } = res;
    Ok(staking_contract)
}

pub fn bin_query(app: &App, lb_pair: &ContractInfo, id: u32) -> StdResult<(u128, u128, u32)> {
    let res = lb_pair::QueryMsg::GetBinReserves { id }.test_query(lb_pair, app)?;
    let lb_pair::BinResponse {
        bin_reserve_x,
        bin_reserve_y,
        bin_id,
    } = res;
    Ok((bin_reserve_x, bin_reserve_y, bin_id))
}

pub fn swap_in_query(
    app: &App,
    lb_pair: &ContractInfo,
    amount_out: Uint128,
    swap_for_y: bool,
) -> StdResult<(Uint128, Uint128, Uint128)> {
    let res = lb_pair::QueryMsg::GetSwapIn {
        amount_out,
        swap_for_y,
    }
    .test_query(lb_pair, app)?;
    let lb_pair::SwapInResponse {
        amount_in,
        amount_out_left,
        fee,
    } = res;
    Ok((amount_in, amount_out_left, fee))
}

pub fn swap_out_query(
    app: &App,
    lb_pair: &ContractInfo,
    amount_in: Uint128,
    swap_for_y: bool,
) -> StdResult<(Uint128, Uint128, Uint128)> {
    let res = lb_pair::QueryMsg::GetSwapOut {
        swap_for_y,
        amount_in,
    }
    .test_query(lb_pair, app)?;
    let lb_pair::SwapOutResponse {
        amount_out,
        amount_in_left,
        total_fees,
        shade_dao_fees: _,
        lp_fees: _,
    } = res;
    Ok((amount_out, amount_in_left, total_fees))
}

pub fn query_static_fee_params(
    app: &App,
    lb_pair: &ContractInfo,
) -> StdResult<(u16, u16, u16, u16, u32, u16, u32)> {
    let res = lb_pair::QueryMsg::GetStaticFeeParameters {}.test_query(lb_pair, app)?;
    let lb_pair::StaticFeeParametersResponse {
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
    } = res;
    Ok((
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
    ))
}

pub fn query_variable_fee_params(
    app: &App,
    lb_pair: &ContractInfo,
) -> StdResult<(u32, u32, u32, u64)> {
    let res = lb_pair::QueryMsg::GetVariableFeeParameters {}.test_query(lb_pair, app)?;
    let lb_pair::VariableFeeParametersResponse {
        volatility_accumulator,
        volatility_reference,
        id_reference,
        time_of_last_update,
    } = res;
    Ok((
        volatility_accumulator,
        volatility_reference,
        id_reference,
        time_of_last_update,
    ))
}

pub fn query_factory(app: &App, lb_pair: &ContractInfo) -> StdResult<Addr> {
    let res = lb_pair::QueryMsg::GetFactory {}.test_query(lb_pair, app)?;
    let lb_pair::FactoryResponse { factory } = res;
    Ok(factory)
}

pub fn query_tokens(app: &App, lb_pair: &ContractInfo) -> StdResult<(TokenType, TokenType)> {
    let res = lb_pair::QueryMsg::GetTokens {}.test_query(lb_pair, app)?;
    let lb_pair::TokensResponse { token_x, token_y } = res;
    Ok((token_x, token_y))
}

pub fn query_token_x(app: &App, lb_pair: &ContractInfo) -> StdResult<TokenType> {
    let res = lb_pair::QueryMsg::GetTokenX {}.test_query(lb_pair, app)?;
    let lb_pair::TokenXResponse { token_x } = res;
    Ok(token_x)
}

pub fn query_token_y(app: &App, lb_pair: &ContractInfo) -> StdResult<TokenType> {
    let res = lb_pair::QueryMsg::GetTokenY {}.test_query(lb_pair, app)?;
    let lb_pair::TokenYResponse { token_y } = res;
    Ok(token_y)
}

pub fn query_bin_step(app: &App, lb_pair: &ContractInfo) -> StdResult<u16> {
    let res = lb_pair::QueryMsg::GetBinStep {}.test_query(lb_pair, app)?;
    let lb_pair::BinStepResponse { bin_step } = res;
    Ok(bin_step)
}

pub fn query_reserves(app: &App, lb_pair: &ContractInfo) -> StdResult<(u128, u128)> {
    let res = lb_pair::QueryMsg::GetReserves {}.test_query(lb_pair, app)?;
    let lb_pair::ReservesResponse {
        reserve_x,
        reserve_y,
    } = res;
    Ok((reserve_x, reserve_y))
}

pub fn query_active_id(app: &App, lb_pair: &ContractInfo) -> StdResult<u32> {
    let res = lb_pair::QueryMsg::GetActiveId {}.test_query(lb_pair, app)?;
    let lb_pair::ActiveIdResponse { active_id } = res;
    Ok(active_id)
}

pub fn query_all_bins_updated(
    app: &App,
    lb_pair: &ContractInfo,
    page: Option<u32>,
    page_size: Option<u32>,
) -> StdResult<Vec<u64>> {
    let res =
        lb_pair::QueryMsg::GetBinUpdatingHeights { page, page_size }.test_query(lb_pair, app)?;
    let lb_pair::BinUpdatingHeightsResponse(heights) = res;
    Ok(heights)
}

pub fn query_updated_bins_at_height(
    app: &App,
    lb_pair: &ContractInfo,
    height: u64,
) -> StdResult<Vec<BinResponse>> {
    let res = lb_pair::QueryMsg::GetUpdatedBinAtHeight { height }.test_query(lb_pair, app)?;
    let lb_pair::UpdatedBinsAtHeightResponse(bins) = res;
    Ok(bins)
}

pub fn query_updated_bins_at_multiple_heights(
    app: &App,
    lb_pair: &ContractInfo,
    heights: Vec<u64>,
) -> StdResult<Vec<BinResponse>> {
    let res =
        lb_pair::QueryMsg::GetUpdatedBinAtMultipleHeights { heights }.test_query(lb_pair, app)?;
    let lb_pair::UpdatedBinsAtMultipleHeightResponse(v) = res;

    Ok(v)
}

pub fn query_updated_bins_after_multiple_heights(
    app: &App,
    lb_pair: &ContractInfo,
    height: u64,
    page: Option<u32>,
    page_size: Option<u32>,
) -> StdResult<(Vec<BinResponse>, u64)> {
    let res = lb_pair::QueryMsg::GetUpdatedBinAfterHeight {
        height,
        page,
        page_size,
    }
    .test_query(lb_pair, app)?;
    let lb_pair::UpdatedBinsAfterHeightResponse {
        bins,
        current_block_height,
    } = res;
    Ok((bins, current_block_height))
}

pub fn query_rewards_distribution(
    app: &App,
    lb_pair: &ContractInfo,
    epoch_id: Option<u64>,
) -> StdResult<RewardsDistribution> {
    let res = lb_pair::QueryMsg::GetRewardsDistribution { epoch_id }.test_query(lb_pair, app)?;
    let lb_pair::RewardsDistributionResponse { distribution } = res;
    Ok(distribution)
}

pub fn query_bin_reserves(
    app: &App,
    lb_pair: &ContractInfo,
    id: u32,
) -> StdResult<(u128, u128, u32)> {
    let res = lb_pair::QueryMsg::GetBinReserves { id }.test_query(lb_pair, app)?;
    let lb_pair::BinResponse {
        bin_reserve_x,
        bin_reserve_y,
        bin_id,
    } = res;
    Ok((bin_reserve_x, bin_reserve_y, bin_id))
}

pub fn query_bins_reserves(
    app: &App,
    lb_pair: &ContractInfo,
    ids: Vec<u32>,
) -> StdResult<Vec<BinResponse>> {
    let res = lb_pair::QueryMsg::GetBinsReserves { ids }.test_query(lb_pair, app)?;
    let lb_pair::BinsResponse(reserves) = res;
    Ok(reserves)
}

pub fn query_all_bins_reserves(
    app: &App,
    lb_pair: &ContractInfo,
    id: Option<u32>,
    page: Option<u32>,
    page_size: Option<u32>,
) -> StdResult<(Vec<BinResponse>, u32, u64)> {
    let res = lb_pair::QueryMsg::GetAllBinsReserves {
        id,
        page,
        page_size,
    }
    .test_query(lb_pair, app)?;
    let lb_pair::AllBinsResponse {
        reserves,
        last_id,
        current_block_height,
    } = res;
    Ok((reserves, last_id, current_block_height))
}

pub fn query_next_non_empty_bin(
    app: &App,
    lb_pair: &ContractInfo,
    swap_for_y: bool,
    id: u32,
) -> StdResult<u32> {
    let res = lb_pair::QueryMsg::GetNextNonEmptyBin { swap_for_y, id }.test_query(lb_pair, app)?;
    let lb_pair::NextNonEmptyBinResponse { next_id } = res;
    Ok(next_id)
}

pub fn query_protocol_fees(app: &App, lb_pair: &ContractInfo) -> StdResult<(u128, u128)> {
    let res = lb_pair::QueryMsg::GetProtocolFees {}.test_query(lb_pair, app)?;
    let lb_pair::ProtocolFeesResponse {
        protocol_fee_x,
        protocol_fee_y,
    } = res;
    Ok((protocol_fee_x, protocol_fee_y))
}

pub fn query_oracle_parameters(
    app: &App,
    lb_pair: &ContractInfo,
) -> StdResult<(u8, u16, u64, u64)> {
    let res = lb_pair::QueryMsg::GetOracleParameters {}.test_query(lb_pair, app)?;
    let lb_pair::OracleParametersResponse {
        sample_lifetime,
        size,
        last_updated,
        first_timestamp,
    } = res;
    Ok((sample_lifetime, size, last_updated, first_timestamp))
}

pub fn query_oracle_sample_at(
    app: &App,
    lb_pair: &ContractInfo,
    oracle_id: u16,
) -> StdResult<OracleSampleResponse> {
    let res = lb_pair::QueryMsg::GetOracleSampleAt { oracle_id }.test_query(lb_pair, app)?;
    let lb_pair::OracleSampleAtResponse { sample } = res;
    Ok(sample)
}

pub fn query_oracle_sample_after(
    app: &App,
    lb_pair: &ContractInfo,
    oracle_id: u16,
) -> StdResult<Vec<OracleSampleResponse>> {
    let res = lb_pair::QueryMsg::GetOracleSamplesAfter {
        oracle_id,
        page_size: None,
    }
    .test_query(lb_pair, app)?;
    let lb_pair::OracleSamplesAtResponse { samples } = res;
    Ok(samples)
}

pub fn query_price_from_id(app: &App, lb_pair: &ContractInfo, id: u32) -> StdResult<Uint256> {
    let res = lb_pair::QueryMsg::GetPriceFromId { id }.test_query(lb_pair, app)?;
    let lb_pair::PriceFromIdResponse { price } = res;
    Ok(price)
}

pub fn query_id_from_price(app: &App, lb_pair: &ContractInfo, price: Uint256) -> StdResult<u32> {
    let res = lb_pair::QueryMsg::GetIdFromPrice { price }.test_query(lb_pair, app)?;
    let lb_pair::IdFromPriceResponse { id } = res;
    Ok(id)
}

pub fn query_swap_out(
    app: &App,
    lb_pair: &ContractInfo,
    amount_in: Uint128,
    swap_for_y: bool,
) -> StdResult<(Uint128, Uint128, Uint128)> {
    let res = lb_pair::QueryMsg::GetSwapOut {
        amount_in,
        swap_for_y,
    }
    .test_query(lb_pair, app)?;
    let lb_pair::SwapOutResponse {
        amount_out,
        amount_in_left,
        total_fees,
        shade_dao_fees: _,
        lp_fees: _,
    } = res;
    Ok((amount_out, amount_in_left, total_fees))
}

pub fn query_swap_in(
    app: &App,
    lb_pair: &ContractInfo,
    amount_out: Uint128,
    swap_for_y: bool,
) -> StdResult<(Uint128, Uint128, Uint128)> {
    let res = lb_pair::QueryMsg::GetSwapIn {
        amount_out,
        swap_for_y,
    }
    .test_query(lb_pair, app)?;
    let lb_pair::SwapInResponse {
        amount_in,
        amount_out_left,
        fee,
    } = res;
    Ok((amount_in, amount_out_left, fee))
}
