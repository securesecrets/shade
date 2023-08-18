//! ### Custom Errors for LB_Factory contract.

#![allow(unused)] // For beginning only.

use libraries::bin_helper::BinError;
use libraries::fee_helper::FeeError;
use libraries::math::liquidity_configurations::LiquidityConfigurationsError;
use libraries::math::u128x128_math::U128x128MathError;
use libraries::math::u256x256_math::U256x256MathError;
use libraries::oracle_helper::OracleError;
use libraries::pair_parameter_helper::PairParametersError;

#[derive(thiserror::Error, Debug)]
pub enum LBQuoterError {
    #[error("Generic {0}")]
    Generic(String),

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
