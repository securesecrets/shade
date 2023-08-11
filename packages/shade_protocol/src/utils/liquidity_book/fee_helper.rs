//! ### Liquidity Book Fee Helper Library
//! Author: Kent
//!
//! This library contains functions to calculate fees.

use ethnum::U256;

use super::constants::*;

#[derive(thiserror::Error, Debug)]
pub enum FeeError {
    #[error("Fee Error: Fee too large")]
    FeeTooLarge,
    #[error("Fee Error: Protocol share too large")]
    ProtocolShareTooLarge,
}
pub struct FeeHelper;

impl FeeHelper {
    /// Check that the fee is not too large.
    fn verify_fee(fee: u128) -> Result<(), FeeError> {
        if fee > MAX_FEE {
            return Err(FeeError::FeeTooLarge);
        }

        Ok(())
    }

    /// Check that the protocol share is not too large.
    fn verify_protocol_share(protocol_share: u128) -> Result<(), FeeError> {
        if protocol_share > MAX_PROTOCOL_SHARE as u128 {
            return Err(FeeError::ProtocolShareTooLarge);
        }

        Ok(())
    }

    /// Calculates the fee amount from the amount with fees, rounding up.
    pub fn get_fee_amount_from(amount_with_fees: u128, total_fee: u128) -> Result<u128, FeeError> {
        Self::verify_fee(total_fee)?;

        // Can't overflow, max(result) = (u128::MAX * 0.1e18 + 1e18 - 1) / 1e18 < 2^128
        let fee_amount = (U256::from(amount_with_fees) * total_fee + PRECISION - 1) / PRECISION;

        Ok(fee_amount.as_u128())
    }

    /// Calculates the fee amount that will be charged, rounding up.
    pub fn get_fee_amount(amount: u128, total_fee: u128) -> Result<u128, FeeError> {
        Self::verify_fee(total_fee)?;

        let denominator = PRECISION - total_fee;
        // Can't overflow, max(result) = (u128::MAX * 0.1e18 + (1e18 - 1)) / 0.9e18 < 2^128
        let fee_amount = (U256::from(amount) * total_fee + denominator - 1) / denominator;

        Ok(fee_amount.as_u128())
    }

    /// Calculates the composition fee amount from the amount with fees, rounding down.
    pub fn get_composition_fee(amount_with_fees: u128, total_fee: u128) -> Result<u128, FeeError> {
        Self::verify_fee(total_fee)?;

        let denominator = SQUARED_PRECISION;
        // Can't overflow, max(result) = type(uint128).max * 0.1e18 * 1.1e18 / 1e36 <= 2^128 * 0.11e36 / 1e36 < 2^128
        let composition_fee =
            U256::from(amount_with_fees) * total_fee * (U256::from(total_fee) + PRECISION)
                / denominator;

        Ok(composition_fee.as_u128())
    }

    /// Calculates the protocol fee amount from the fee amount and the protocol share, rounding down.
    pub fn get_protocol_fee_amount(
        fee_amount: u128,
        protocol_share: u128,
    ) -> Result<u128, FeeError> {
        Self::verify_protocol_share(protocol_share)?;

        let protocol_fee_amount = U256::from(fee_amount) * protocol_share / BASIS_POINT_MAX as u128;

        Ok(protocol_fee_amount.as_u128())
    }
}
