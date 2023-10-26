//! ### Custom Errors for LB_Factory contract.

#![allow(unused)] // For beginning only.

use bin_helper::BinError;
use cosmwasm_std::Addr;
use fee_helper::FeeError;
use math::{
    liquidity_configurations::LiquidityConfigurationsError,
    u128x128_math::U128x128MathError,
    u256x256_math::U256x256MathError,
};
use oracle_helper::OracleError;
use pair_parameter_helper::PairParametersError;
use shade_protocol::lb_libraries::{
    bin_helper,
    fee_helper,
    math,
    oracle_helper,
    pair_parameter_helper,
};

#[derive(thiserror::Error, Debug)]
pub enum LBFactoryError {
    #[error("{0}!")]
    Generic(String),

    #[error("Only the Owner can do that!")]
    OnlyOwner,

    #[error("Tokens are identical! Both addresses are {token}!")]
    IdenticalAddresses { token: String },

    #[error("Quote Asset {quote_asset} is not whitelisted!")]
    QuoteAssetNotWhitelisted { quote_asset: String },

    #[error("Quote Asset {quote_asset} is already whitelisted!")]
    QuoteAssetAlreadyWhitelisted { quote_asset: String },

    #[error("LBPair ({token_x}, {token_y}, bin_step: {bin_step}) does not exist!")]
    LBPairDoesNotExist {
        token_x: String,
        token_y: String,
        bin_step: u16,
    },

    #[error("LBPair ({token_x}, {token_y}, bin_step: {bin_step}) not created!")]
    LBPairNotCreated {
        token_x: String,
        token_y: String,
        bin_step: u16,
    },

    #[error("LBPair ({token_x}, {token_y}, bin_step: {bin_step}) already exists!")]
    LBPairAlreadyExists {
        token_x: String,
        token_y: String,
        bin_step: u16,
    },

    #[error("Flash Loan Fee above max: {fee} > {max_fee}!")]
    FlashLoanFeeAboveMax { fee: u8, max_fee: u8 },

    #[error("Bin step {bin_step} is too low!")]
    BinStepTooLow { bin_step: u16 },

    #[error("Preset {bin_step} is locked for users! {user} is not the owner!")]
    PresetIsLockedForUsers { user: Addr, bin_step: u16 },

    #[error("LBPair.ignored is already in the same state!")]
    LBPairIgnoredIsAlreadyInTheSameState,

    #[error("Bin step {bin_step} has no preset!")]
    BinStepHasNoPreset { bin_step: u16 },

    #[error("Preset open state is already in the same state!")]
    PresetOpenStateIsAlreadyInTheSameState,

    #[error("Fee recipient is already {fee_recipient}!")]
    SameFeeRecipient { fee_recipient: Addr },

    #[error("Flash loan fee is already {fee}!")]
    SameFlashLoanFee { fee: u8 },

    #[error(
        "LBPair safety check failed. {lb_pair_implementation} factory address does not match this one!"
    )]
    LBPairSafetyCheckFailed { lb_pair_implementation: Addr },

    #[error(
        "LBFactory safety check failed. {lb_factory_implementation} factory address does not match this one!"
    )]
    LBFactorySafetyCheckFailed { lb_factory_implementation: Addr },

    #[error("LB implementation is already set to code ID {lb_implementation}!")]
    SameImplementation { lb_implementation: u64 },

    #[error("The LBPair implementation has not been set yet!")]
    ImplementationNotSet,

    #[error("Transaction is blocked by contract status")]
    TransactionBlock(),

    #[error(transparent)]
    CwErr(#[from] cosmwasm_std::StdError),

    #[error(transparent)]
    BinErr(#[from] BinError),

    #[error(transparent)]
    FeeErr(#[from] FeeError),

    #[error(transparent)]
    OracleErr(#[from] OracleError),

    #[error(transparent)]
    ParamsErr(#[from] PairParametersError),

    #[error(transparent)]
    LiquidityConfigErr(#[from] LiquidityConfigurationsError),

    #[error(transparent)]
    U128Err(#[from] U128x128MathError),

    #[error(transparent)]
    U256Err(#[from] U256x256MathError),
}
