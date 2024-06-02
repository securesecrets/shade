use crate::multi::lb_factory::LbFactory;
use lb_libraries::types::ContractImplementation;
use shade_protocol::{
    c_std::{Addr, ContractInfo, StdError, StdResult},
    contract_interfaces::liquidity_book::lb_factory,
    liquidity_book::lb_pair::{LBPair, LBPairInformation, RewardsDistributionAlgorithm},
    multi_test::App,
    swap::core::TokenType,
    utils::{
        asset::{Contract, RawContract},
        ExecuteCallback, InstantiateCallback, MultiTestable, Query,
    },
};

pub fn init(
    app: &mut App,
    sender: &str,
    fee_recipient: Addr,
    admin_auth: RawContract,
    query_auth: RawContract,
    recover_staking_funds_receiver: Addr,
) -> StdResult<ContractInfo> {
    let lb_factory = ContractInfo::from(
        match (lb_factory::InstantiateMsg {
            owner: Some(Addr::unchecked(sender)),
            fee_recipient,
            admin_auth,
            recover_staking_funds_receiver,
            query_auth,
            max_bins_per_swap: Some(500),
        }
        .test_init(
            LbFactory::default(),
            app,
            Addr::unchecked(sender),
            "lb_factory",
            &[],
        )) {
            Ok(contract_info) => contract_info,
            Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
        },
    );
    Ok(lb_factory)
}

pub fn set_lb_pair_implementation(
    app: &mut App,
    sender: &str,
    lb_factory: &ContractInfo,
    id: u64,
    code_hash: String,
) -> StdResult<()> {
    match (lb_factory::ExecuteMsg::SetLbPairImplementation {
        implementation: ContractImplementation { id, code_hash },
    }
    .test_exec(lb_factory, app, Addr::unchecked(sender), &[]))
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    }
}

pub fn set_lb_token_implementation(
    app: &mut App,
    sender: &str,
    lb_factory: &ContractInfo,
    id: u64,
    code_hash: String,
) -> StdResult<()> {
    match (lb_factory::ExecuteMsg::SetLbTokenImplementation {
        implementation: ContractImplementation { id, code_hash },
    }
    .test_exec(lb_factory, app, Addr::unchecked(sender), &[]))
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    }
}

pub fn set_staking_contract_implementation(
    app: &mut App,
    sender: &str,
    lb_factory: &ContractInfo,
    id: u64,
    code_hash: String,
) -> StdResult<()> {
    match (lb_factory::ExecuteMsg::SetStakingContractImplementation {
        implementation: ContractImplementation { id, code_hash },
    }
    .test_exec(lb_factory, app, Addr::unchecked(sender), &[]))
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    }
}

pub fn set_pair_preset(
    app: &mut App,
    sender: &str,
    lb_factory: &ContractInfo,
    bin_step: u16,
    base_factor: u16,
    filter_period: u16,
    decay_period: u16,
    reduction_factor: u16,
    variable_fee_control: u32,
    protocol_share: u16,
    max_volatility_accumulator: u32,
    is_open: bool,
    total_reward_bins: u32,
    rewards_distribution_algorithm: Option<RewardsDistributionAlgorithm>,
    epoch_staking_index: u64,
    epoch_staking_duration: u64,
    expiry_staking_duration: Option<u64>,
) -> StdResult<()> {
    match (lb_factory::ExecuteMsg::SetPairPreset {
        bin_step,
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
        is_open,
        total_reward_bins,
        rewards_distribution_algorithm: rewards_distribution_algorithm
            .unwrap_or(RewardsDistributionAlgorithm::TimeBasedRewards),
        epoch_staking_index,
        epoch_staking_duration,
        expiry_staking_duration,
    }
    .test_exec(lb_factory, app, Addr::unchecked(sender), &[]))
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    }
}

pub fn remove_preset(
    app: &mut App,
    sender: &str,
    lb_factory: &ContractInfo,
    bin_step: u16,
) -> StdResult<()> {
    match (lb_factory::ExecuteMsg::RemovePreset { bin_step }.test_exec(
        lb_factory,
        app,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    }
}

pub fn add_quote_asset(
    app: &mut App,
    sender: &str,
    lb_factory: &ContractInfo,
    asset: TokenType,
) -> StdResult<()> {
    match (lb_factory::ExecuteMsg::AddQuoteAsset { asset }.test_exec(
        lb_factory,
        app,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(e) => {
            return Err(StdError::generic_err(e.root_cause().to_string()));
        }
    }
}

pub fn remove_quote_asset(
    app: &mut App,
    sender: &str,
    lb_factory: &ContractInfo,
    asset: TokenType,
) -> StdResult<()> {
    match (lb_factory::ExecuteMsg::RemoveQuoteAsset { asset }.test_exec(
        lb_factory,
        app,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(e) => {
            return Err(StdError::generic_err(e.root_cause().to_string()));
        }
    }
}

pub fn create_lb_pair(
    app: &mut App,
    sender: &str,
    lb_factory: &ContractInfo,
    bin_step: u16,
    active_id: u32,
    token_x: TokenType,
    token_y: TokenType,
    viewing_key: String,
    entropy: String,
) -> StdResult<()> {
    match (lb_factory::ExecuteMsg::CreateLbPair {
        token_x,
        token_y,
        active_id,
        bin_step,
        viewing_key,
        entropy,
    }
    .test_exec(lb_factory, app, Addr::unchecked(sender), &[]))
    {
        Ok(_) => Ok(()),
        Err(e) => {
            return Err(StdError::generic_err(e.root_cause().to_string()));
        }
    }
}

pub fn set_fees_parameters_on_pair(
    app: &mut App,
    sender: &str,
    lb_factory: &ContractInfo,
    token_x: TokenType,
    token_y: TokenType,
    bin_step: u16,
    base_factor: u16,
    filter_period: u16,
    decay_period: u16,
    reduction_factor: u16,
    variable_fee_control: u32,
    protocol_share: u16,
    max_volatility_accumulator: u32,
) -> StdResult<()> {
    match (lb_factory::ExecuteMsg::SetFeeParametersOnPair {
        token_x,
        token_y,
        bin_step,
        base_factor,
        filter_period,
        decay_period,
        reduction_factor,
        variable_fee_control,
        protocol_share,
        max_volatility_accumulator,
    }
    .test_exec(lb_factory, app, Addr::unchecked(sender), &[]))
    {
        Ok(_) => Ok(()),
        Err(e) => return Err(StdError::generic_err(e.root_cause().to_string())),
    }
}

pub fn set_preset_open_state(
    app: &mut App,
    sender: &str,
    lb_factory: &ContractInfo,
    bin_step: u16,
    is_open: bool,
) -> StdResult<()> {
    match (lb_factory::ExecuteMsg::SetPresetOpenState { bin_step, is_open }.test_exec(
        lb_factory,
        app,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(e) => {
            return Err(StdError::generic_err(e.root_cause().to_string()));
        }
    }
}

pub fn set_fee_recipient(
    app: &mut App,
    sender: &str,
    lb_factory: &ContractInfo,
    fee_recipient: Addr,
) -> StdResult<()> {
    match (lb_factory::ExecuteMsg::SetFeeRecipient { fee_recipient }.test_exec(
        lb_factory,
        app,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(e) => {
            return Err(StdError::generic_err(e.root_cause().to_string()));
        }
    }
}

pub fn force_decay(
    app: &mut App,
    sender: &str,
    lb_factory: &ContractInfo,
    pair: LBPair,
) -> StdResult<()> {
    match (lb_factory::ExecuteMsg::ForceDecay { pair }.test_exec(
        lb_factory,
        app,
        Addr::unchecked(sender),
        &[],
    )) {
        Ok(_) => Ok(()),
        Err(e) => {
            return Err(StdError::generic_err(e.root_cause().to_string()));
        }
    }
}

pub fn query_lb_pair_implementation(
    app: &mut App,
    lb_factory: &ContractInfo,
) -> StdResult<ContractImplementation> {
    match (lb_factory::QueryMsg::GetLbPairImplementation {}.test_query(lb_factory, app)) {
        Ok(lb_factory::LbPairImplementationResponse {
            lb_pair_implementation,
        }) => Ok(lb_pair_implementation),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn query_lb_token_implementation(
    app: &mut App,
    lb_factory: &ContractInfo,
) -> StdResult<ContractImplementation> {
    match (lb_factory::QueryMsg::GetLbTokenImplementation {}.test_query(lb_factory, app)) {
        Ok(lb_factory::LbTokenImplementationResponse {
            lb_token_implementation,
        }) => Ok(lb_token_implementation),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn query_min_bin_step(app: &mut App, lb_factory: &ContractInfo) -> StdResult<u8> {
    match (lb_factory::QueryMsg::GetMinBinStep {}.test_query(lb_factory, app)) {
        Ok(lb_factory::MinBinStepResponse { min_bin_step }) => Ok(min_bin_step),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn query_fee_recipient(app: &mut App, lb_factory: &ContractInfo) -> StdResult<Addr> {
    match (lb_factory::QueryMsg::GetFeeRecipient {}.test_query(lb_factory, app)) {
        Ok(lb_factory::FeeRecipientResponse { fee_recipient }) => Ok(fee_recipient),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn query_number_of_lb_pairs(app: &mut App, lb_factory: &ContractInfo) -> StdResult<u32> {
    match (lb_factory::QueryMsg::GetNumberOfLbPairs {}.test_query(lb_factory, app)) {
        Ok(lb_factory::NumberOfLbPairsResponse { lb_pair_number }) => Ok(lb_pair_number),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn query_all_lb_pairs(
    app: &mut App,
    lb_factory: &ContractInfo,
    token_x: TokenType,
    token_y: TokenType,
) -> StdResult<Vec<LBPairInformation>> {
    match (lb_factory::QueryMsg::GetAllLbPairs { token_x, token_y }.test_query(lb_factory, app)) {
        Ok(lb_factory::AllLbPairsResponse { lb_pairs_available }) => Ok(lb_pairs_available),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn query_lb_pair_information(
    app: &mut App,
    lb_factory: &ContractInfo,
    token_x: TokenType,
    token_y: TokenType,
    bin_step: u16,
) -> StdResult<LBPairInformation> {
    match (lb_factory::QueryMsg::GetLbPairInformation {
        token_x,
        token_y,
        bin_step,
    }
    .test_query(lb_factory, app))
    {
        Ok(lb_factory::LbPairInformationResponse {
            lb_pair_information,
        }) => Ok(lb_pair_information),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn query_all_bin_steps(app: &mut App, lb_factory: &ContractInfo) -> StdResult<Vec<u16>> {
    match (lb_factory::QueryMsg::GetAllBinSteps {}.test_query(lb_factory, app)) {
        Ok(lb_factory::AllBinStepsResponse {
            bin_step_with_preset,
        }) => Ok(bin_step_with_preset),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn query_preset(
    app: &mut App,
    lb_factory: &ContractInfo,
    bin_step: u16,
) -> StdResult<lb_factory::PresetResponse> {
    match (lb_factory::QueryMsg::GetPreset { bin_step }.test_query(lb_factory, app)) {
        Ok(response) => Ok(response),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn query_number_of_quote_assets(app: &mut App, lb_factory: &ContractInfo) -> StdResult<u32> {
    match (lb_factory::QueryMsg::GetNumberOfQuoteAssets {}.test_query(lb_factory, app)) {
        Ok(lb_factory::NumberOfQuoteAssetsResponse {
            number_of_quote_assets,
        }) => Ok(number_of_quote_assets),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn query_is_quote_asset(
    app: &mut App,
    lb_factory: &ContractInfo,
    token: TokenType,
) -> StdResult<bool> {
    match (lb_factory::QueryMsg::IsQuoteAsset { token }.test_query(lb_factory, app)) {
        Ok(lb_factory::IsQuoteAssetResponse { is_quote }) => Ok(is_quote),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn query_quote_asset_at_index(
    app: &mut App,
    lb_factory: &ContractInfo,
    index: u32,
) -> StdResult<TokenType> {
    match (lb_factory::QueryMsg::GetQuoteAssetAtIndex { index }.test_query(lb_factory, app)) {
        Ok(lb_factory::QuoteAssetAtIndexResponse { asset }) => Ok(asset),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}
