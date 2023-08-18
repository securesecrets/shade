//! ### Custom Errors for LB_Factory contract.

#![allow(unused)] // For beginning only.

use cosmwasm_std::Uint128;
use libraries::bin_helper::BinError;
use libraries::fee_helper::FeeError;
use libraries::math::liquidity_configurations::LiquidityConfigurationsError;
use libraries::math::u128x128_math::U128x128MathError;
use libraries::math::u256x256_math::U256x256MathError;
use libraries::oracle_helper::OracleError;
use libraries::pair_parameter_helper::PairParametersError;

#[derive(thiserror::Error, Debug)]
pub enum LBRouterError {
    #[error("Generic {0}")]
    Generic(String),

    #[error("The sender is not WNATIVE")]
    SenderIsNotWNATIVE,

    #[error("Unknown reply {id}")]
    UnknownReplyId { id: u64 },

    #[error("Wrong amounts. Amount: {amount}, Reserve: {reserve}")]
    WrongAmounts { amount: u128, reserve: u128 },

    #[error("Swap overflows for bin id {id}")]
    SwapOverflows { id: u32 },

    #[error("Broken swap safety check")]
    BrokenSwapSafetyCheck,

    #[error("Not factory owner")]
    NotFactoryOwner,

    #[error("Too many tokens in. Excess: {excess}")]
    TooManyTokensIn { excess: u128 },

    #[error("Bin reserve overflows for bin id {id}")]
    BinReserveOverflows { id: u128 },

    #[error("Failed to send WNATIVE to recipient {recipient}. Amount: {amount}")]
    FailedToSendNATIVE { recipient: String, amount: u128 },

    #[error("Amount slippage BP too big. Amount slippage: {amount_slippage}")]
    AmountSlippageBPTooBig { amount_slippage: u128 },

    #[error("Insufficient amount out. Amount out min: {amount_out_min}, Amount out: {amount_out}")]
    InsufficientAmountOut {
        amount_out_min: Uint128,
        amount_out: Uint128,
    },

    #[error("Max amount in exceeded. Amount in max: {amount_in_max}, Amount in: {amount_in}")]
    MaxAmountInExceeded {
        amount_in_max: u128,
        amount_in: u128,
    },

    #[error("Invalid token path. Wrong token: {0}")]
    InvalidTokenPath(String),

    #[error("Invalid version: {0}")]
    InvalidVersion(u32),

    #[error("Wrong WNATIVE liquidity parameters. token_x: {token_x}, token_y: {token_y}, amount_x: {amount_x}, amount_y: {amount_y}, msg_value: {msg_value}")]
    WrongNativeLiquidityParameters {
        token_x: String,
        token_y: String,
        amount_x: u128,
        amount_y: u128,
        msg_value: u128,
    },

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

    #[error("Sent a non-native token. Should use the receive interface in SNIP20.")]
    NonNativeTokenErr,

    #[error("Pair not found")]
    PairNotFound,

    #[error("Current no trade in progress")]
    NoTradeInProgress,
}
