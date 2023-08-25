//! ### Custom Errors for LB_Pair contract.

#![allow(unused)] // For beginning only.

use bin_helper::BinError;
use cosmwasm_std::Uint128;
use ethnum::U256;
use fee_helper::FeeError;
use math::liquidity_configurations::LiquidityConfigurationsError;
use math::u128x128_math::U128x128MathError;
use math::u256x256_math::U256x256MathError;
use oracle_helper::OracleError;
use pair_parameter_helper::PairParametersError;
use shade_protocol::lb_libraries::{
    bin_helper, fee_helper, math, oracle_helper, pair_parameter_helper,
};

#[derive(thiserror::Error, Debug)]
pub enum LBPairError {
    #[error("Generic {0}")]
    Generic(String),

    #[error("Zero borrow amount!")]
    ZeroBorrowAmount,

    #[error("Address is zero!")]
    AddressZero,

    #[error("Serilization Failed is zero!")]
    SerializationError,

    #[error("Only the Factory can do that!")]
    OnlyFactory,

    #[error("Only the Protocol Fee Recipient can do that!")]
    OnlyProtocolFeeRecipient,

    #[error("Empty Market Configuration")]
    EmptyMarketConfigs,

    #[error("Flash loan callback failed!")]
    FlashLoanCallbackFailed,

    #[error("Flash loan insufficient amount!")]
    FlashLoanInsufficientAmount,

    #[error("Insufficient amount in!")]
    InsufficientAmountIn,

    #[error("Insufficient amount out!")]
    InsufficientAmountOut,

    #[error("Invalid input!")]
    InvalidInput,

    #[error("Invalid static fee parameters!")]
    InvalidStaticFeeParameters,

    #[error("Not enough liquidity!")]
    OutOfLiquidity,

    #[error("Token not supported!")]
    TokenNotSupported(),

    #[error("Zero amount for bin id: {id}")]
    ZeroAmount { id: u32 },

    #[error("Zero amounts out for bin id: {id} amount to burn: {amount_to_burn} total supply: {total_supply} ")]
    ZeroAmountsOut {
        id: u32,
        // bin_reserves: [u8; 32],
        amount_to_burn: U256,
        total_supply: U256,
        // amounts_out_from_bin: [u8; 32],
    },

    #[error("Zero Shares for bin id: {id}")]
    ZeroShares { id: u32 },

    #[error("Max total fee exceeded!")]
    MaxTotalFeeExceeded,

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

    #[error("Wrong Pair")]
    WrongPair,

    #[error("Deadline exceeded. Deadline: {deadline}, Current timestamp: {current_timestamp}")]
    DeadlineExceeded {
        deadline: u64,
        current_timestamp: u64,
    },

    #[error("Lengths mismatch")]
    LengthsMismatch,

    #[error("Id desired overflows. Id desired: {id_desired}, Id slippage: {id_slippage}")]
    IdDesiredOverflows { id_desired: u32, id_slippage: u32 },

    #[error("Delta id overflows. Delta Id: {delta_id}")]
    DeltaIdOverflows { delta_id: i64 },

    #[error("Id underflow. Id: {id} Delta Id: {delta_id}")]
    IdUnderflows { id: u32, delta_id: u32 },

    #[error("Id overflows. Id: {id}")]
    IdOverflows { id: u32 },

    #[error("Id slippage caught. Active id desired: {active_id_desired}, Id slippage: {id_slippage}, Active id: {active_id}")]
    IdSlippageCaught {
        active_id_desired: u32,
        id_slippage: u32,
        active_id: u32,
    },

    #[error("Pair not created: {token_x} and {token_y}, binStep: {bin_step}")]
    PairNotCreated {
        token_x: String,
        token_y: String,
        bin_step: u16,
    },
    #[error("Amount slippage caught. AmountXMin: {amount_x_min}, AmountX: {amount_x}, AmountYMin: {amount_y_min}, AmountY: {amount_y}")]
    AmountSlippageCaught {
        amount_x_min: Uint128,
        amount_x: Uint128,
        amount_y_min: Uint128,
        amount_y: Uint128,
    },
}
