//! ### Liquidity Book Price Helper Library
//! Author: Kent
//!
//! This library contains functions to calculate prices.

use ethnum::{I256, U256};

use super::constants::*;
use super::math::u128x128_math::{U128x128Math, U128x128MathError};
use super::math::u256x256_math::{U256x256Math, U256x256MathError};

// represents a 24 bit number (uint24, which we're not using yet)
const REAL_ID_SHIFT: I256 = I256::new(1 << 23);

#[derive(thiserror::Error, Debug)]
pub enum PriceError {
    #[error(transparent)]
    U128MathErr(#[from] U128x128MathError),

    #[error(transparent)]
    U256MathErr(#[from] U256x256MathError),
}

pub struct PriceHelper;

impl PriceHelper {
    /// Calculates the price as a 128.128-binary fixed-point number from the id and the bin step.
    pub fn get_price_from_id(id: u32, bin_step: u16) -> Result<U256, U128x128MathError> {
        let base = Self::get_base(bin_step);
        let exponent = Self::get_exponent(id);

        U128x128Math::pow(base, exponent)
    }

    // TODO: make unique type for fixed-point numbers?
    /// Calculates the id from the price and the bin step.
    ///
    /// # Arguments
    ///
    /// * `price` - The price as a 128.128-binary fixed-point number
    /// * `bin_step` - The bin step
    pub fn get_id_from_price(price: U256, bin_step: u16) -> Result<u32, U128x128MathError> {
        let base = Self::get_base(bin_step);

        let real_id = U128x128Math::log2(price)? / U128x128Math::log2(base)?;

        Ok((REAL_ID_SHIFT + real_id).as_u32())
    }

    /// Calculates the base from the bin step, which is `1 + binStep / BASIS_POINT_MAX`.
    pub fn get_base(bin_step: u16) -> U256 {
        let base = SCALE + (U256::from(bin_step) << SCALE_OFFSET) / BASIS_POINT_MAX as u128;

        base
    }

    /// Calculates the exponent from the id, which is `id - REAL_ID_SHIFT`.
    pub fn get_exponent(id: u32) -> I256 {
        I256::from(id) - REAL_ID_SHIFT
    }

    /// Converts a price with 18 decimals to a 128.128-binary fixed-point number.
    pub fn convert_decimal_price_to128x128(price: U256) -> Result<U256, U256x256MathError> {
        U256x256Math::shift_div_round_down(price, SCALE_OFFSET, PRECISION.into())
    }

    /// Converts a 128.128-binary fixed-point number to a price with 18 decimals.
    pub fn convert128x128_price_to_decimal(price128x128: U256) -> Result<U256, U256x256MathError> {
        U256x256Math::mul_shift_round_down(price128x128, PRECISION.into(), SCALE_OFFSET)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_price_from_id() {
        let id = 8574931;
        let bin_step = 1u16;
        let price = PriceHelper::get_price_from_id(id, bin_step).unwrap();

        assert!(price > U256::ZERO);
        //42008768657166552252904831246223292524636112144
        // println!("price {}", price);
    }

    #[test]
    fn test_get_id_from_price() {
        let price = U256::from(100u128);
        let bin_step = 5u16;
        let id = PriceHelper::get_id_from_price(price, bin_step).unwrap();

        assert!(id > 0);
    }

    #[test]
    fn test_get_base() {
        let bin_step = 5u16;
        let base = PriceHelper::get_base(bin_step);

        assert!(base > U256::ZERO);
    }

    #[test]
    fn test_get_exponent() {
        let id = 50u32;
        let exponent = PriceHelper::get_exponent(id);

        assert!(exponent > I256::ZERO);
    }

    #[test]
    fn test_convert_decimal_price_to128x128() {
        let price = U256::from(100u128);
        let converted_price = PriceHelper::convert_decimal_price_to128x128(price).unwrap();

        assert!(converted_price > U256::ZERO);
    }

    #[test]
    fn test_convert128x128_price_to_decimal() {
        let price128x128 = U256::from(100u128);
        let converted_price = PriceHelper::convert128x128_price_to_decimal(price128x128).unwrap();

        assert!(converted_price > U256::ZERO);
    }
}
