//! ### Liquidity Book U256x256 Math Library
//! Author: Kent
//!
//! Helper library used for full precision calculations.
use std::{ops::Add, str::FromStr};

use cosmwasm_std::Uint512;
use ethnum::U256;

use super::uint256_to_u256::ConvertU256;

#[derive(thiserror::Error, Debug)]
pub enum U256x256MathError {
    #[error("Generic {0}")]
    Generic(String),

    #[error("U256x256 Math Error: MulShiftOverflow")]
    MulShiftOverflow,

    #[error("U256x256 Math Error: MulDivOverflow")]
    MulDivOverflow,
}

pub struct U256x256Math;

impl U256x256Math {
    /// Calculates `floor(x*y/denominator)` with full precision.
    /// The result will be rounded down.
    ///
    /// Credit to Remco Bloemen under MIT license https://xn--2-umb.com/21/muldiv
    ///
    /// # Requirements
    ///
    /// - The denominator cannot be zero
    /// - The result must fit within U256
    ///
    /// # Caveats
    ///
    /// - This function does not work with fixed-point numbers
    ///
    /// # Arguments
    ///
    /// * `x` The multiplicand as U256
    /// * `y` The multiplier as U256
    /// * `denominator` - The divisor as U256
    ///
    /// # Returns
    ///
    /// * `result` - The result as U256
    pub fn mul_div_round_down(
        x: U256,
        y: U256,
        denominator: U256,
    ) -> Result<U256, U256x256MathError> {
        let (prod0, prod1) = Self::_get_mul_prods(x, y)?;

        let result = Self::_get_end_of_div_round_down(x, y, denominator, prod0, prod1)?;
        Ok(result)
    }

    /// Calculates `ceil(x*y/denominator)` with full precision.
    /// The result will be rounded up.
    ///
    /// Credit to Remco Bloemen under MIT license https://xn--2-umb.com/21/muldiv
    ///
    /// # Requirements
    ///
    /// - The denominator cannot be zero
    /// - The result must fit within U256
    ///
    /// # Caveats
    ///
    /// - This function does not work with fixed-point numbers
    ///
    /// # Arguments
    ///
    /// * `x` The multiplicand as U256
    /// * `y` The multiplier as U256
    /// * `denominator` - The divisor as U256
    ///
    /// # Returns
    ///
    /// * `result` - The result as U256
    pub fn mul_div_round_up(
        x: U256,
        y: U256,
        denominator: U256,
    ) -> Result<U256, U256x256MathError> {
        let mut result = Self::mul_div_round_down(x, y, denominator)?;

        if Self::mulmod(x, y, denominator) != 0 {
            result += 1;
        }

        Ok(result)
    }

    /// Calculates `floor(x * y / 2**offset)` with full precision.
    /// The result will be rounded down.
    ///
    /// Credit to Remco Bloemen under MIT license https://xn--2-umb.com/21/muldiv
    ///
    /// # Requirements
    ///
    /// - The offset needs to be strictly lower than 256
    /// - The result must fit within U256
    ///
    /// # Caveats
    ///
    /// - This function does not work with fixed-point numbers
    ///
    /// # Arguments
    ///
    /// * `x` The multiplicand as U256
    /// * `y` The multiplier as U256
    /// * `offset` - The number of bits to shift x as u8
    /// * `denominator` - The divisor as U256
    ///
    /// # Returns
    ///
    /// * `result` - The result as U256
    pub fn mul_shift_round_down(x: U256, y: U256, offset: u8) -> Result<U256, U256x256MathError> {
        let (prod0, prod1) = Self::_get_mul_prods(x, y)?;
        let mut result = U256::new(0);

        if prod0 != 0 {
            result = prod0 >> offset;
        }
        if prod1 != 0 {
            // Make sure the result is less than 2^256.
            if prod1 >= U256::ONE << offset {
                return Err(U256x256MathError::MulShiftOverflow);
            }

            result += prod1 << (256u16 - offset as u16);
        }

        Ok(result)
    }

    /// Calculates `floor(x * y / 2**offset)` with full precision.
    /// The result will be rounded up.
    ///
    /// Credit to Remco Bloemen under MIT license https://xn--2-umb.com/21/muldiv
    ///
    /// # Requirements
    ///
    /// - The offset needs to be strictly lower than 256
    /// - The result must fit within U256
    ///
    /// # Caveats
    ///
    /// - This function does not work with fixed-point numbers
    ///
    /// # Arguments
    ///
    /// * `x` The multiplicand as U256
    /// * `y` The multiplier as U256
    /// * `offset` - The number of bits to shift x as u8
    /// * `denominator` - The divisor as U256
    ///
    /// # Returns
    ///
    /// * `result` - The result as U256
    pub fn mul_shift_round_up(x: U256, y: U256, offset: u8) -> Result<U256, U256x256MathError> {
        let mut result = Self::mul_shift_round_down(x, y, offset)?;

        if x.wrapping_mul(y) % (U256::ONE << offset) != 0 {
            result += 1;
        }

        Ok(result)
    }

    /// Calculates `floor(x << offset / y)` with full precision.
    /// The result will be rounded down.
    ///
    /// Credit to Remco Bloemen under MIT license https://xn--2-umb.com/21/muldiv
    ///
    /// # Requirements
    ///
    /// - The offset needs to be strictly lower than 256
    /// - The result must fit within U256
    ///
    /// # Caveats
    ///
    /// - This function does not work with fixed-point numbers
    ///
    /// # Arguments
    ///
    /// * `x` The multiplicand as U256
    /// * `offset` - The number of bits to shift x as u8
    /// * `denominator` - The divisor as U256
    ///
    /// # Returns
    ///
    /// * `result` - The result as U256
    pub fn shift_div_round_down(
        x: U256,
        offset: u8,
        denominator: U256,
    ) -> Result<U256, U256x256MathError> {
        let prod0 = x << offset; // Least significant 256 bits of the product
        let prod1 = x >> (256u16 - offset as u16); // Most significant 256 bits of the product

        let y = U256::ONE
            .checked_shl(offset as u32)
            .ok_or(U256x256MathError::Generic("overflow".to_owned()))?;

        let result = Self::_get_end_of_div_round_down(x, y, denominator, prod0, prod1)?;

        Ok(result)
    }

    /// Calculates `ceil(x << offset / y)` with full precision.
    /// The result will be rounded up.
    ///
    /// Credit to Remco Bloemen under MIT license https://xn--2-umb.com/21/muldiv
    ///
    /// # Requirements
    ///
    /// - The offset needs to be strictly lower than 256
    /// - The result must fit within U256
    ///
    /// # Caveats
    ///
    /// - This function does not work with fixed-point numbers
    ///
    /// # Arguments
    ///
    /// * `x` The multiplicand as U256
    /// * `offset` - The number of bits to shift x as u8
    /// * `denominator` - The divisor as U256
    ///
    /// # Returns
    ///
    /// * `result` - The result as U256
    pub fn shift_div_round_up(
        x: U256,
        offset: u8,
        denominator: U256,
    ) -> Result<U256, U256x256MathError> {
        let mut result = Self::shift_div_round_down(x, offset, denominator)?;

        if x.wrapping_mul(U256::ONE << offset) % denominator != 0 {
            result += 1;
        }

        Ok(result)
    }

    /// Helper function to return the result of `x * y` as 2 U256
    ///
    /// # Arguments
    ///
    /// * `x` - The multiplicand as an U256
    /// * `y` - The multiplier as an U256
    ///
    /// # Returns
    ///
    /// * A tuple containing:
    ///   * `prod0` - The least significant 256 bits of the product
    ///   * `prod1` - The most significant 256 bits of the product

    pub fn _get_mul_prods(x: U256, y: U256) -> Result<(U256, U256), U256x256MathError> {
        let k: U256 = U256::MAX;

        // Calculate (x * y) % k
        let mm = Self::mulmod(x, y, k);
        // Calculate x * y
        let prod0 = x.overflowing_mul(y).0;
        // Calculate prod1
        let prod1 = mm.overflowing_sub(prod0).0.overflowing_sub(if mm < prod0 {
            U256::ONE
        } else {
            U256::ZERO
        });

        Ok((prod0, prod1.0))
    }

    pub fn mulmod(a: U256, b: U256, modulo: U256) -> U256 {
        // Convert to U512 for internal calculations
        let mut res = Uint512::zero();
        let mut a = Uint512::from(a.u256_to_uint256()) % Uint512::from(modulo.u256_to_uint256());
        let mut b = Uint512::from(b.u256_to_uint256());

        while b > Uint512::zero() {
            if b % Uint512::from(2u128) == Uint512::from(1u128) {
                res = (res + a) % Uint512::from(modulo.u256_to_uint256());
            }

            a = (a * Uint512::from(2u128)) % Uint512::from(modulo.u256_to_uint256());
            b /= Uint512::from(2u128);
        }
        let ret =
            U256::from_str_prefixed(&(res % Uint512::from(modulo.u256_to_uint256())).to_string())
                .unwrap();

        // Convert the result back to U256 before returning
        U256::from(ret)
    }

    // # TODO: double check this
    /// Helper function to return the result of `x * y / denominator` with full precision
    ///
    ///
    /// # Arguments
    ///
    /// * `x` - The multiplicand as an uint256 (U256)
    /// * `y` - The multiplier as an uint256 (U256)
    /// * `denominator` - The divisor as an uint256 (U256)
    /// * `prod0` - The least significant 256 bits of the product
    /// * `prod1` - The most significant 256 bits of the product
    ///
    /// # Returns
    ///
    /// * `result` - The result as an uint256 (U256)
    pub fn _get_end_of_div_round_down(
        x: U256,
        y: U256,
        mut denominator: U256,
        mut prod0: U256,
        mut prod1: U256,
    ) -> Result<U256, U256x256MathError> {
        let result: U256;

        // Handle non-overflow cases, 256 by 256 division
        if prod1 == 0 {
            result = prod0 / denominator;
            Ok(result)
        } else {
            // Make sure the result is less than 2^256. Also prevents denominator == 0
            if prod1 >= denominator {
                return Err(U256x256MathError::MulDivOverflow);
            }

            // Make division exact by subtracting the remainder from [prod1 prod0].

            // Compute remainder using mulmod.
            let remainder = Self::mulmod(x, y, denominator);

            // Subtract 256 bit number from 512 bit number.
            if remainder > prod0 {
                prod1 -= U256::ONE;
            }
            prod0 = prod0.overflowing_sub(remainder).0;

            // Factor powers of two out of denominator and compute largest power of two divisor of denominator. Always >= 1
            // See https://cs.stackexchange.com/q/138556/92363

            // Does not overflow because the denominator cannot be zero at this stage in the function
            let mut lpotdod: U256 = denominator & (!denominator + U256::ONE);
            // Divide denominator by lpotdod.
            denominator = denominator.overflowing_div(lpotdod).0;

            // Divide [prod1 prod0] by lpotdod.
            let prod0: U256 = prod0.overflowing_div(lpotdod).0;

            // Flip lpotdod such that it is 2^256 / lpotdod. If lpotdod is zero, then it becomes one
            if lpotdod != U256::MIN {
                let two_pow_256: Uint512 =
                    Uint512::from(U256::MAX.u256_to_uint256()).add(Uint512::one());
                let mut temp: Uint512 = two_pow_256 / Uint512::from(lpotdod.u256_to_uint256());
                if temp >= two_pow_256 {
                    // temp = (temp >> 256) - Uint512::one();
                    temp >>= 256;
                    lpotdod = U256::MAX
                        .overflowing_add(U256::from_str(&temp.to_string()).unwrap())
                        .0;
                } else {
                    lpotdod = U256::from_str(&temp.to_string()).unwrap()
                }
            } else {
                lpotdod = U256::ONE;
            }
            let prod0 = prod0 | prod1.overflowing_mul(lpotdod).0;

            let mut inverse = U256::from(3u128).overflowing_mul(denominator).0 ^ 2;

            for _ in 0..7 {
                // inverse mod 2^8 to 2^256
                // inverse *= 2 - denominator * inverse; // inverse mod 2^8
                // inverse *= 2 - denominator * inverse; // inverse mod 2^16
                // inverse *= 2 - denominator * inverse; // inverse mod 2^32
                // inverse *= 2 - denominator * inverse; // inverse mod 2^64
                // inverse *= 2 - denominator * inverse; // inverse mod 2^128
                // inverse *= 2 - denominator * inverse; // inverse mod 2^256
                let denom_times_inverse = denominator.overflowing_mul(inverse).0;
                let subtraction_result = U256::from(2u128).overflowing_sub(denom_times_inverse).0;
                inverse = inverse.overflowing_mul(subtraction_result).0;
            }

            // Because the division is now exact we can divide by multiplying with the modular inverse of denominator.
            // This will give us the correct result modulo 2^256. Since the preconditions guarantee that the outcome is
            // less than 2^256, this is the final result. We don't need to compute the high bits of the result and prod1
            // is no longer required.
            result = prod0.overflowing_mul(inverse).0;

            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {

    use ethnum::U256;

    use crate::utils::liquidity_book::{
        constants::{PRECISION, SCALE_OFFSET},
        math::u256x256_math::U256x256Math,
    };

    #[test]
    fn test_get_mul_product() {
        let x =
            U256::from_str_prefixed("42008768997448919173843294709597899956404323600000").unwrap();
        let y =
            U256::from_str_prefixed("42008768997448919173843294709597899956404323600000").unwrap();
        let _z = U256x256Math::_get_mul_prods(x, y);
    }

    #[test]
    #[should_panic]
    fn test_mul_div_round_down_div_by_zero() {
        let x = U256::from(1u128);
        let y = U256::from(1u128);
        let denominator = U256::MIN; // Zero
        let _ = U256x256Math::mul_div_round_down(x, y, denominator).unwrap();
    }
    #[test]
    fn test_mul_div_round_down_min() {
        let x = U256::MIN;
        let y = U256::MIN;
        let denominator = U256::MAX;

        let res = U256x256Math::mul_div_round_down(x, y, denominator).unwrap();
        assert_eq!(res, U256::MIN);
    }
    #[test]
    fn test_mul_div_round_down_max() {
        let x = U256::MAX;
        let y = U256::MAX;
        let denominator = U256::from(1u128);

        let res = U256x256Math::mul_div_round_down(x, y, denominator);

        assert!(res.is_err());
    }

    #[test]
    fn test_mul_div_round_down_denom_less_than_prod1() {
        let x = U256::MAX;
        let y = U256::from(2u128);
        let denominator = U256::from(1u128);

        let res = U256x256Math::mul_div_round_down(x, y, denominator);
        assert!(res.is_err());
    }

    #[test]
    fn test_mul_div_round_down_max_result() {
        let x = U256::MAX;
        let y = U256::from(2u128);
        let denominator = U256::from(2u128);

        let res = U256x256Math::mul_div_round_down(x, y, denominator).unwrap();
        assert_eq!(res, U256::MAX);
    }

    #[test]
    fn test_mul_div_round_down_all_max() {
        let x = U256::MAX;
        let y = U256::MAX;
        let denominator = U256::MAX;

        let res = U256x256Math::mul_div_round_down(x, y, denominator).unwrap();
        assert_eq!(res, U256::MAX);
    }

    #[test]
    fn test_mul_div_round_up() {
        let x = U256::from(1000u128);
        let y = U256::from(1000u128);
        let denominator = U256::from(100u128);

        let res = U256x256Math::mul_div_round_down(x, y, denominator).unwrap();
        assert_eq!(res, U256::from(10000u128)); // Replace with expected result
    }

    #[test]
    fn test_mul_shift_div_round_down() {
        let x = U256::from_words(1000u128, 1000u128);
        let y = U256::from(PRECISION);
        let shift = SCALE_OFFSET;

        let res = U256x256Math::mul_shift_round_down(x, y, shift).unwrap();
        assert_eq!(res, U256::from(1000000000000000000000u128)); // Replace with expected result
    }

    #[test]
    fn test_mul_div_round_up_max() {
        let x = U256::MAX;
        let y = U256::MAX;
        let denominator = U256::from(1u128);

        let res = U256x256Math::mul_div_round_up(x, y, denominator);
        assert!(res.is_err());
    }

    #[test]
    fn test_mul_div_round_up_min() {
        let x = U256::MIN;
        let y = U256::MIN;
        let denominator = U256::from(1u128);

        let res = U256x256Math::mul_div_round_up(x, y, denominator).unwrap();
        assert_eq!(res, U256::MIN);
    }

    #[test]
    #[should_panic]
    fn test_mul_div_round_up_denom_zero() {
        let x = U256::from(1u128);
        let y = U256::from(1u128);
        let denominator = U256::MIN; // Zero

        let res = U256x256Math::mul_div_round_up(x, y, denominator);
        assert!(res.is_err());
    }

    #[test]
    fn test_mul_div_round_up_overflow() {
        let x = U256::MAX;
        let y = U256::from(2u128);
        let denominator = U256::from(1u128);

        let res = U256x256Math::mul_div_round_up(x, y, denominator);
        assert!(res.is_err());
    }

    #[test]
    fn test_mul_div_round_up_not_evenly_divisible() {
        let x = U256::from(10u128);
        let y = U256::from(10u128);
        let denominator = U256::from(3u128);

        let res = U256x256Math::mul_div_round_up(x, y, denominator).unwrap();
        assert_eq!(res, U256::from(34u128)); // Because 10*10/3 = 33.333... so it should round up to 34
    }

    #[test]
    fn test_mul_shift_div_round_up() {
        let x = U256::from_words(1000u128, 1000u128);
        let y = U256::from(PRECISION);
        let shift = SCALE_OFFSET;

        let res = U256x256Math::mul_shift_round_up(x, y, shift).unwrap();
        assert_eq!(res, U256::from(1000000000000000000001u128)); // Replace with expected result
    }

    #[test]
    fn test_shift_div_round_down() {
        let x = U256::from(1000u128);
        let shift = 10u8;
        let denominator = U256::from(100u128);

        let shifted = x << shift;
        let (prod0, prod1) = U256x256Math::_get_mul_prods(shifted, U256::ONE).unwrap();

        assert_eq!(prod0, U256::from(1024000u128)); // Replace with expected result
        assert_eq!(prod1, U256::from(0u128)); // Replace with expected result

        if denominator != U256::ZERO {
            if prod1 != U256::ZERO && denominator <= prod1 {
                panic!("Overflow error!");
            } else {
                let res = U256x256Math::shift_div_round_down(x, shift, denominator).unwrap();
                assert_eq!(res, 10240);
            }
        } else {
            panic!("Denominator is zero!");
        }
    }

    #[test]
    fn test_shift_div_round_up() {
        let x = U256::from(1000u128);
        let shift = 10u8;
        let denominator = U256::from(100u128);

        let shifted = x << shift;
        let (prod0, prod1) = U256x256Math::_get_mul_prods(shifted, U256::ONE).unwrap();

        assert_eq!(prod0, U256::from(1024000u128)); // Replace with expected result
        assert_eq!(prod1, U256::from(0u128)); // Replace with expected result

        if denominator != U256::ZERO {
            if prod1 != U256::ZERO && denominator <= prod1 {
                panic!("Overflow error!");
            } else {
                let res = U256x256Math::shift_div_round_down(x, shift, denominator).unwrap();
                assert_eq!(res, 10240); // Replace with expected result
            }
        } else {
            panic!("Denominator is zero!");
        }
    }
}
