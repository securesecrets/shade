//! ### Liquidity Book Fee Helper Library
//! Author: Kent and Haseeb
//!
//! This library contains functions to calculate fees.

use ethnum::U256;

use super::constants::*;

#[derive(thiserror::Error, Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_fee() {
        // Test a fee that is below MAX_FEE
        let fee = MAX_FEE - 1;
        let result = FeeHelper::verify_fee(fee);
        assert!(
            result.is_ok(),
            "Fee should be valid when it is less than MAX_FEE"
        );

        // Test a fee that is equal to MAX_FEE
        let fee = MAX_FEE;
        let result = FeeHelper::verify_fee(fee);
        assert!(
            result.is_ok(),
            "Fee should be valid when it is equal to MAX_FEE"
        );

        // Test a fee that is greater than MAX_FEE
        let fee = MAX_FEE + 1;
        let result = FeeHelper::verify_fee(fee);
        assert!(
            result.is_err(),
            "Fee should be invalid when it is greater than MAX_FEE"
        );

        // Verify that the correct Error type is returned
        match result {
            Ok(_) => panic!("This should have returned an Err"),
            Err(e) => assert_eq!(e, FeeError::FeeTooLarge),
        }
    }

    #[test]
    fn test_verify_protocol_share() {
        // Test a protocol_share that is below MAX_PROTOCOL_SHARE
        let protocol_share = MAX_PROTOCOL_SHARE as u128 - 1;
        let result = FeeHelper::verify_protocol_share(protocol_share);
        assert!(
            result.is_ok(),
            "Protocol share should be valid when it is less than MAX_PROTOCOL_SHARE"
        );

        // Test a protocol_share that is equal to MAX_PROTOCOL_SHARE
        let protocol_share = MAX_PROTOCOL_SHARE as u128;
        let result = FeeHelper::verify_protocol_share(protocol_share);
        assert!(
            result.is_ok(),
            "Protocol share should be valid when it is equal to MAX_PROTOCOL_SHARE"
        );

        // Test a protocol_share that is greater than MAX_PROTOCOL_SHARE
        let protocol_share = MAX_PROTOCOL_SHARE as u128 + 1;
        let result = FeeHelper::verify_protocol_share(protocol_share);
        assert!(
            result.is_err(),
            "Protocol share should be invalid when it is greater than MAX_PROTOCOL_SHARE"
        );

        // Verify that the correct Error type is returned
        match result {
            Ok(_) => panic!("This should have returned an Err"),
            Err(e) => assert_eq!(e, FeeError::ProtocolShareTooLarge),
        }
    }

    #[test]
    fn test_get_fee_amount_from() {
        // Test with valid fee and amount
        let total_fee = 10;
        let amount_with_fees = 100;
        let expected_fee_amount =
            (U256::from(amount_with_fees) * total_fee + PRECISION - 1) / PRECISION;

        let result = FeeHelper::get_fee_amount_from(amount_with_fees, total_fee);
        assert!(result.is_ok(), "Should succeed with valid fee and amount");
        assert_eq!(
            result,
            Ok(expected_fee_amount.as_u128()),
            "Fee amount should match expected value"
        );
        assert_eq!(result, Ok(1u128));

        // Test with valid fee and amount
        let total_fee = MAX_FEE;
        let amount_with_fees = u128::MAX;
        let expected_fee_amount =
            (U256::from(amount_with_fees) * total_fee + PRECISION - 1) / PRECISION;
        let result = FeeHelper::get_fee_amount_from(amount_with_fees, total_fee);
        assert!(result.is_ok(), "Should succeed with valid fee and amount");
        assert_eq!(
            result.unwrap(),
            expected_fee_amount.as_u128(),
            "Fee amount should match expected value"
        );

        let result = FeeHelper::get_fee_amount_from(amount_with_fees, total_fee);
        assert!(result.is_ok(), "Should succeed with valid fee and amount");
        assert_eq!(
            result.unwrap(),
            expected_fee_amount.as_u128(),
            "Fee amount should match expected value"
        );

        // Test with fee greater than MAX_FEE
        let total_fee = MAX_FEE + 1;
        let result = FeeHelper::get_fee_amount_from(amount_with_fees, total_fee);
        assert!(result.is_err(), "Should fail with fee greater than MAX_FEE");

        // Verify that the correct Error type is returned
        match result {
            Ok(_) => panic!("This should have returned an Err"),
            Err(e) => assert_eq!(e, FeeError::FeeTooLarge),
        }
    }

    #[test]
    fn test_get_fee_amount() -> Result<(), FeeError> {
        // Test typical case
        let amount = 1000u128;
        let total_fee = 100u128; // 0.1% fee
        let fee_amount = FeeHelper::get_fee_amount(amount, total_fee)?;
        assert_eq!(fee_amount, 1); // fee should be 1

        // Test when amount is zero
        let fee_amount_zero = FeeHelper::get_fee_amount(0, total_fee)?;
        assert_eq!(fee_amount_zero, 0); // fee should be zero

        // Test when fee is zero
        let fee_amount_no_fee = FeeHelper::get_fee_amount(amount, 0)?;
        assert_eq!(fee_amount_no_fee, 0); // fee should be zero

        // Test when fee is maximum allowed
        let fee_amount_max_fee = FeeHelper::get_fee_amount(amount, MAX_FEE)?;
        assert!(fee_amount_max_fee > 0); // fee should be greater than zero

        // Test error scenario: Fee too large
        let result = FeeHelper::get_fee_amount(amount, MAX_FEE + 1);
        assert!(matches!(result, Err(FeeError::FeeTooLarge)));

        Ok(())
    }

    #[test]
    fn test_get_composition_fee() -> Result<(), FeeError> {
        // Test typical case
        let amount_with_fees = MAX_FEE;
        let total_fee = MAX_FEE / 10; // 0.1% fee
        let comp_fee = FeeHelper::get_composition_fee(amount_with_fees, total_fee)?;
        assert_eq!(comp_fee, 1010000000000000); // fee should be 1

        // Test when amount_with_fees is zero
        let comp_fee_zero = FeeHelper::get_composition_fee(0, total_fee)?;
        assert_eq!(comp_fee_zero, 0); // fee should be zero

        // Test when total_fee is zero
        let comp_fee_no_fee = FeeHelper::get_composition_fee(amount_with_fees, 0)?;
        assert_eq!(comp_fee_no_fee, 0); // fee should be zero

        // Test when total_fee is maximum
        let comp_fee_max_fee = FeeHelper::get_composition_fee(amount_with_fees, MAX_FEE)?;
        assert!(comp_fee_max_fee > 0); // fee should be greater than zero

        // Test error scenario: Fee too large
        let result = FeeHelper::get_composition_fee(amount_with_fees, MAX_FEE + 1);
        assert!(matches!(result, Err(FeeError::FeeTooLarge)));

        Ok(())
    }

    #[test]
    fn test_get_protocol_fee_amount() -> Result<(), FeeError> {
        // Test typical case
        let fee_amount = 1000u128;
        let protocol_share = 50u128; // 0.5% protocol share
        let protocol_fee = FeeHelper::get_protocol_fee_amount(fee_amount, protocol_share)?;
        assert_eq!(protocol_fee, 5); // fee should be 5

        // Test when fee_amount is zero
        let protocol_fee_zero = FeeHelper::get_protocol_fee_amount(0, protocol_share)?;
        assert_eq!(protocol_fee_zero, 0); // fee should be zero

        // Test when protocol_share is zero
        let protocol_fee_no_share = FeeHelper::get_protocol_fee_amount(fee_amount, 0)?;
        assert_eq!(protocol_fee_no_share, 0); // fee should be zero

        // Test when protocol_share is maximum
        let protocol_share = MAX_PROTOCOL_SHARE as u128;
        let protocol_fee_max_share =
            FeeHelper::get_protocol_fee_amount(fee_amount, protocol_share)?;
        assert!(protocol_fee_max_share > 0); // fee should be greater than zero

        // Test error scenario: Protocol share too large
        let result = FeeHelper::get_protocol_fee_amount(fee_amount, protocol_share + 1);
        assert!(matches!(result, Err(FeeError::ProtocolShareTooLarge)));

        Ok(())
    }
}
