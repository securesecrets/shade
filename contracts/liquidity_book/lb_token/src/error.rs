//! ### Custom Errors for LB_TOKEN contract.

#![allow(unused)] // For beginning only.

use libraries::bin_helper::BinError;
use libraries::fee_helper::FeeError;
use libraries::math::liquidity_configurations::LiquidityConfigurationsError;
use libraries::math::u128x128_math::U128x128MathError;
use libraries::math::u256x256_math::U256x256MathError;
use libraries::oracle_helper::OracleError;
use libraries::pair_parameter_helper::PairParametersError;

#[derive(thiserror::Error, Debug)]
pub enum LBTokenError {
    #[error("Generic {0}")]
    Generic(String),

    #[error(transparent)]
    CwErr(#[from] cosmwasm_std::StdError),

    #[error("Invalid Error")]
    InvalidInput(String),

    #[error("Insufficient Funds")]
    InsufficientFunds,

    #[error("Insufficient Supply")]
    InsufficientSupply,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Spender Not Approved")]
    SpenderNotApproved,

    #[error("Self Approval")]
    SelfApproval,

    #[error("Already Approved")]
    AlreadyApproved,
}
