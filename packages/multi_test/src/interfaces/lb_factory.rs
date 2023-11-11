use crate::multi::lb_factory::LbFactory;
use shade_protocol::{
    c_std::{Addr, ContractInfo, StdError, StdResult},
    contract_interfaces::liquidity_book::lb_factory,
    lb_libraries::{
        tokens::TokenType,
        types::{ContractInstantiationInfo, LBPair, LBPairInformation},
    },
    multi_test::App,
    utils::{
        asset::{Contract, RawContract},
        ExecuteCallback,
        InstantiateCallback,
        MultiTestable,
        Query,
    },
};

pub fn init(
    app: &mut App,
    sender: &str,
    fee_recipient: Addr,
    flash_loan_fee: u8,
    admin_auth: RawContract,
) -> StdResult<Contract> {
    let lb_factory = Contract::from(
        match (lb_factory::InstantiateMsg {
            owner: Some(Addr::unchecked(sender)),
            fee_recipient,
            flash_loan_fee,
            admin_auth,
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
    match (lb_factory::ExecuteMsg::SetLBPairImplementation {
        lb_pair_implementation: ContractInstantiationInfo { id, code_hash },
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
    match (lb_factory::ExecuteMsg::SetLBTokenImplementation {
        lb_token_implementation: ContractInstantiationInfo { id, code_hash },
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
    match (lb_factory::ExecuteMsg::CreateLBPair {
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

pub fn query_flash_loan_fee(app: &mut App, lb_factory: &ContractInfo) -> StdResult<u8> {
    match (lb_factory::QueryMsg::GetFlashLoanFee {}.test_query(lb_factory, app)) {
        Ok(lb_factory::FlashLoanFeeResponse { flash_loan_fee }) => Ok(flash_loan_fee),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn query_lb_pair_implementation(
    app: &mut App,
    lb_factory: &ContractInfo,
) -> StdResult<ContractInstantiationInfo> {
    match (lb_factory::QueryMsg::GetLBPairImplementation {}.test_query(lb_factory, app)) {
        Ok(lb_factory::LBPairImplementationResponse {
            lb_pair_implementation,
        }) => Ok(lb_pair_implementation),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn query_lb_token_implementation(
    app: &mut App,
    lb_factory: &ContractInfo,
) -> StdResult<ContractInstantiationInfo> {
    match (lb_factory::QueryMsg::GetLBTokenImplementation {}.test_query(lb_factory, app)) {
        Ok(lb_factory::LBTokenImplementationResponse {
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

pub fn query_max_flash_loan_fee(app: &mut App, lb_factory: &ContractInfo) -> StdResult<u8> {
    match (lb_factory::QueryMsg::GetMaxFlashLoanFee {}.test_query(lb_factory, app)) {
        Ok(lb_factory::MaxFlashLoanFeeResponse { max_fee }) => Ok(max_fee),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn query_number_of_lb_pairs(app: &mut App, lb_factory: &ContractInfo) -> StdResult<u32> {
    match (lb_factory::QueryMsg::GetNumberOfLBPairs {}.test_query(lb_factory, app)) {
        Ok(lb_factory::NumberOfLBPairsResponse { lb_pair_number }) => Ok(lb_pair_number),
        Err(e) => Err(StdError::generic_err(e.to_string())),
    }
}

pub fn query_all_lb_pairs(
    app: &mut App,
    lb_factory: &ContractInfo,
    token_x: TokenType,
    token_y: TokenType,
) -> StdResult<Vec<LBPairInformation>> {
    match (lb_factory::QueryMsg::GetAllLBPairs { token_x, token_y }.test_query(lb_factory, app)) {
        Ok(lb_factory::AllLBPairsResponse { lb_pairs_available }) => Ok(lb_pairs_available),
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
    match (lb_factory::QueryMsg::GetLBPairInformation {
        token_x,
        token_y,
        bin_step,
    }
    .test_query(lb_factory, app))
    {
        Ok(lb_factory::LBPairInformationResponse {
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
