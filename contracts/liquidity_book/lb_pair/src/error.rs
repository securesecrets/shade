//! ### Custom Errors for LB_Pair contract.

use shade_protocol::{
    c_std::{StdError, Uint128, Uint256},
    lb_libraries::{
        bin_helper::BinError,
        fee_helper::FeeError,
        math::{
            liquidity_configurations::LiquidityConfigurationsError,
            u128x128_math::U128x128MathError,
            u256x256_math::U256x256MathError,
        },
        oracle_helper::OracleError,
        pair_parameter_helper::PairParametersError,
    },
};

#[derive(thiserror::Error, Debug)]
pub enum LBPairError {
    // Generic Errors
    #[error("Generic {0}")]
    Generic(String),
    #[error("Zero borrow amount!")]
    ZeroBorrowAmount,
    #[error("Address is zero!")]
    AddressZero,
    #[error("Serilization Failed is zero!")]
    SerializationError,
    #[error("Invalid input!")]
    InvalidInput,
    #[error("value greater than u24!")]
    U24Overflow,
    #[error("Token not supported!")]
    TokenNotSupported(),
    #[error("Transaction is blocked by contract status")]
    TransactionBlock(),
    #[error("Not enough funds")]
    NotEnoughFunds,

    // Permission Errors
    #[error("Only the Factory can do that!")]
    OnlyFactory,
    #[error("Only the Protocol Fee Recipient can do that!")]
    OnlyProtocolFeeRecipient,

    // Market Configuration Errors
    #[error("Empty Market Configuration")]
    EmptyMarketConfigs,
    #[error("Invalid static fee parameters!")]
    InvalidStaticFeeParameters,

    // Liquidity and Flash Loan Errors
    #[error("Not enough liquidity!")]
    OutOfLiquidity,
    #[error("Flash loan callback failed!")]
    FlashLoanCallbackFailed,
    #[error("Flash loan insufficient amount!")]
    FlashLoanInsufficientAmount,
    #[error("Insufficient amount in!")]
    InsufficientAmountIn,
    #[error("Insufficient amount out!")]
    InsufficientAmountOut,

    // Oracle Errors
    #[error("Oracle not active!")]
    OracleNotActive,

    // Interface and Callback Errors
    #[error("Use the receive interface")]
    UseReceiveInterface,
    #[error("Receiver callback \"msg\" parameter cannot be empty.")]
    ReceiverMsgEmpty,

    // Time and Deadline Errors
    #[error("Deadline exceeded. Deadline: {deadline}, Current timestamp: {current_timestamp}")]
    DeadlineExceeded {
        deadline: u64,
        current_timestamp: u64,
    },

    // Specific Errors with Parameters
    #[error("Zero amount for bin id: {id}")]
    ZeroAmount { id: u32 },
    #[error("Zero Shares for bin id: {id}")]
    ZeroShares { id: u32 },
    #[error("Max total fee exceeded!")]
    MaxTotalFeeExceeded,
    #[error("Wrong Pair")]
    WrongPair,

    // Error Wrappings from Dependencies
    #[error(transparent)]
    CwErr(#[from] StdError),
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

    // Complex Scenarios and Calculations Errors
    #[error(
        "Zero amounts out for bin id: {id} amount to burn: {amount_to_burn} total supply: {total_supply}"
    )]
    ZeroAmountsOut {
        id: u32,
        amount_to_burn: Uint256,
        total_supply: Uint256,
    },
    // Id and Calculation Related Errors
    #[error("Id desired overflows. Id desired: {id_desired}, Id slippage: {id_slippage}")]
    IdDesiredOverflows { id_desired: u32, id_slippage: u32 },
    #[error("Delta id overflows. Delta Id: {delta_id}")]
    DeltaIdOverflows { delta_id: i64 },
    #[error("Id underflow. Id: {id} Delta Id: {delta_id}")]
    IdUnderflows { id: u32, delta_id: u32 },
    #[error("Id overflows. Id: {id}")]
    IdOverflows { id: u32 },
    #[error("could not get bin reserves for active id: {active_id}")]
    ZeroBinReserve { active_id: u32 },
    #[error("Lengths mismatch")]
    LengthsMismatch,
    #[error("time_of_last_update was later than look_up_timestamp")]
    LastUpdateTimestampGreaterThanLookupTimestamp,

    // Slippage and Trading Errors
    #[error(
        "Amount left unswapped. : Amount Left In: {amount_left_in}, Total Amount: {total_amount}, swapped_amount: {swapped_amount}"
    )]
    AmountInLeft {
        amount_left_in: Uint128,
        total_amount: Uint128,
        swapped_amount: Uint128,
    },
    #[error(
        "Id slippage caught. Active id desired: {active_id_desired}, Id slippage: {id_slippage}, Active id: {active_id}"
    )]
    IdSlippageCaught {
        active_id_desired: u32,
        id_slippage: u32,
        active_id: u32,
    },
    #[error(
        "Amount slippage caught. AmountXMin: {amount_x_min}, AmountX: {amount_x}, AmountYMin: {amount_y_min}, AmountY: {amount_y}"
    )]
    AmountSlippageCaught {
        amount_x_min: Uint128,
        amount_x: Uint128,
        amount_y_min: Uint128,
        amount_y: Uint128,
    },
    #[error("Pair not created: {token_x} and {token_y}, binStep: {bin_step}")]
    PairNotCreated {
        token_x: String,
        token_y: String,
        bin_step: u16,
    },
    #[error("No matching token in pair")]
    NoMatchingTokenInPair,
}
