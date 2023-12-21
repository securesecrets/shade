use super::lb_pair::{self, RewardsDistributionAlgorithm};
use crate::{
    contract_interfaces::swap::core::TokenType,
    lb_libraries::types::{ContractInstantiationInfo, LBPair, LBPairInformation},
    utils::asset::RawContract,
};

use crate::utils::{ExecuteCallback, InstantiateCallback, Query};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;
pub use lb_pair::InstantiateMsg as LBPairInstantiateMsg;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin_auth: RawContract,
    pub owner: Option<Addr>,
    pub fee_recipient: Addr,
    pub total_reward_bins: u32,
    pub rewards_distribution_algorithm: RewardsDistributionAlgorithm,
    pub epoch_staking_index: u64,
    pub epoch_staking_duration: u64,
    pub expiry_staking_duration: Option<u64>,
    pub recover_staking_funds_receiver: Addr,
}
impl InstantiateCallback for InstantiateMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
pub enum ExecuteMsg {
    #[serde(rename = "set_lb_pair_implementation")]
    SetLBPairImplementation {
        implementation: ContractInstantiationInfo,
    },
    #[serde(rename = "set_lb_token_implementation")]
    SetLBTokenImplementation {
        implementation: ContractInstantiationInfo,
    },
    #[serde(rename = "set_staking_contract_implementation")]
    SetStakingContractImplementation {
        implementation: ContractInstantiationInfo,
    },
    #[serde(rename = "create_lb_pair")]
    CreateLBPair {
        token_x: TokenType,
        token_y: TokenType,
        // u24
        active_id: u32,
        bin_step: u16,
        viewing_key: String,
        entropy: String,
    },
    // #[serde(rename = "set_lb_pair_ignored")]
    // SetLBPairIgnored {
    //     token_x: TokenType,
    //     token_y: TokenType,
    //     bin_step: u16,
    //     ignored: bool,
    // },
    SetPairPreset {
        bin_step: u16,
        base_factor: u16,
        filter_period: u16,
        decay_period: u16,
        reduction_factor: u16,
        // u24
        variable_fee_control: u32,
        protocol_share: u16,
        // u24
        max_volatility_accumulator: u32,
        total_reward_bins: u32,
        is_open: bool,
    },
    SetPresetOpenState {
        bin_step: u16,
        is_open: bool,
    },
    RemovePreset {
        bin_step: u16,
    },
    SetFeeParametersOnPair {
        token_x: TokenType,
        token_y: TokenType,
        bin_step: u16,
        base_factor: u16,
        filter_period: u16,
        decay_period: u16,
        reduction_factor: u16,
        // u24
        variable_fee_control: u32,
        protocol_share: u16,
        // u24
        max_volatility_accumulator: u32,
    },
    SetFeeRecipient {
        fee_recipient: Addr,
    },

    AddQuoteAsset {
        asset: TokenType,
    },
    RemoveQuoteAsset {
        asset: TokenType,
    },
    ForceDecay {
        pair: LBPair,
    },
}

impl ExecuteCallback for ExecuteMsg {
    const BLOCK_SIZE: usize = 256;
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(MinBinStepResponse)]
    GetMinBinStep {},
    #[returns(FeeRecipientResponse)]
    GetFeeRecipient {},
    #[returns(LBPairImplementationResponse)]
    #[serde(rename = "get_lb_pair_implementation")]
    GetLBPairImplementation {},
    #[returns(LBTokenImplementationResponse)]
    #[serde(rename = "get_lb_token_implementation")]
    GetLBTokenImplementation {},
    #[returns(NumberOfLBPairsResponse)]
    #[serde(rename = "get_number_of_lb_pairs")]
    GetNumberOfLBPairs {},
    #[returns(LBPairAtIndexResponse)]
    #[serde(rename = "get_lb_pair_at_index")]
    GetLBPairAtIndex { index: u32 },
    #[returns(NumberOfQuoteAssetsResponse)]
    GetNumberOfQuoteAssets {},
    #[returns(QuoteAssetAtIndexResponse)]
    GetQuoteAssetAtIndex { index: u32 },
    #[returns(IsQuoteAssetResponse)]
    IsQuoteAsset { token: TokenType },
    #[returns(LBPairInformationResponse)]
    #[serde(rename = "get_lb_pair_information")]
    GetLBPairInformation {
        token_x: TokenType,
        token_y: TokenType,
        bin_step: u16,
    },
    #[returns(PresetResponse)]
    GetPreset { bin_step: u16 },
    #[returns(AllBinStepsResponse)]
    GetAllBinSteps {},
    #[returns(OpenBinStepsResponse)]
    GetOpenBinSteps {},
    #[returns(AllLBPairsResponse)]
    #[serde(rename = "get_all_lb_pairs")]
    GetAllLBPairs {
        token_x: TokenType,
        token_y: TokenType,
    },
}

impl Query for QueryMsg {
    const BLOCK_SIZE: usize = 256;
}

// We define a custom struct for each query response
#[cw_serde]
pub struct MinBinStepResponse {
    pub min_bin_step: u8,
}

#[cw_serde]
pub struct FeeRecipientResponse {
    pub fee_recipient: Addr,
}

#[cw_serde]
pub struct LBPairImplementationResponse {
    pub lb_pair_implementation: ContractInstantiationInfo,
}

#[cw_serde]
pub struct LBTokenImplementationResponse {
    pub lb_token_implementation: ContractInstantiationInfo,
}

#[cw_serde]
pub struct NumberOfLBPairsResponse {
    pub lb_pair_number: u32,
}

#[cw_serde]
pub struct LBPairAtIndexResponse {
    pub lb_pair: LBPair,
}

#[cw_serde]
pub struct NumberOfQuoteAssetsResponse {
    pub number_of_quote_assets: u32,
}

#[cw_serde]
pub struct QuoteAssetAtIndexResponse {
    pub asset: TokenType,
}

#[cw_serde]
pub struct IsQuoteAssetResponse {
    pub is_quote: bool,
}

#[cw_serde]
pub struct LBPairInformationResponse {
    pub lb_pair_information: LBPairInformation,
}

#[cw_serde]
pub struct PresetResponse {
    pub base_factor: u16,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    // u24
    pub variable_fee_control: u32,
    pub protocol_share: u16,
    // u24
    pub max_volatility_accumulator: u32,
    pub is_open: bool,
}

#[cw_serde]
pub struct AllBinStepsResponse {
    pub bin_step_with_preset: Vec<u16>,
}

#[cw_serde]
pub struct OpenBinStepsResponse {
    pub open_bin_steps: Vec<u16>,
}

#[cw_serde]
pub struct AllLBPairsResponse {
    pub lb_pairs_available: Vec<LBPairInformation>,
}
