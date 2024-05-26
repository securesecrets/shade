//! ### Liquidity Book U256x256 Math Library
//! Author: Kent and Haseeb
//!
//! Helper library used for full precision calculations.

use ethnum::U256;
use primitive_types::{U128, U512};
use std::ops::BitXor;

pub trait U256ToU512Conversion {
    fn to_u512(&self) -> U512;
}

impl U256ToU512Conversion for U256 {
    fn to_u512(&self) -> U512 {
        U512::from_little_endian(&self.to_le_bytes())
    }
}

pub trait U512ToU256Conversion {
    fn to_u256(&self) -> U256;
}

impl U512ToU256Conversion for U512 {
    fn to_u256(&self) -> U256 {
        if self <= &U512::zero() {
            U256::ZERO
        } else if self > &U512::from_little_endian(&U256::MAX.to_le_bytes()) {
            U256::MAX
        } else {
            let lo: u128 = U128([self.0[0], self.0[1]]).as_u128();
            let hi: u128 = U128([self.0[2], self.0[3]]).as_u128();
            U256::from_words(hi, lo)
        }
    }
}

/// Computes (x * y) % k where the addition is performed with arbitrary precision and does not wrap around at 2^256.
fn mulmod(x: U256, y: U256, k: U256) -> U256 {
    if k == U256::ZERO {
        return U256::ZERO;
    }

    if let Some(z) = x.checked_mul(y) {
        return z % k;
    }

    let x: &U512 = &x.to_u512();
    let y: &U512 = &y.to_u512();
    let k: &U512 = &k.to_u512();
    let z: U512 = (x * y) % k;
    z.to_u256()
}

#[derive(thiserror::Error, Debug)]
pub enum U256x256MathError {
    #[error("Generic {0}")]
    Generic(String),

    #[error("U256x256 Math Error: MulShiftOverflow")]
    MulShiftOverflow,

    #[error("U256x256 Math Error: MulDivOverflow")]
    MulDivOverflow,

    #[error("Value greater than u128")]
    U128Overflow,
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
        // Edge case where all three values are equal (not sure why this would ever happen)
        if x == y && y == denominator {
            return Ok(x);
        }

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

        if mulmod(x, y, denominator) != 0 {
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

        if mulmod(x, y, U256::ONE << offset) != 0 {
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
        let result =
            Self::_get_end_of_div_round_down(x, U256::ONE << offset, denominator, prod0, prod1)?;

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

        if mulmod(x, U256::ONE << offset, denominator) != 0 {
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
    fn _get_mul_prods(x: U256, y: U256) -> Result<(U256, U256), U256x256MathError> {
        // 512-bit multiply [prod1 prod0] = x * y. Compute the product mod 2^256 and mod 2^256 - 1, then use
        // use the Chinese Remainder Theorem to reconstruct the 512 bit result. The result is stored in two 256
        // variables such that product = prod1 * 2^256 + prod0.

        let mm = mulmod(x, y, U256::MAX);
        let prod0 = x.wrapping_mul(y);
        let prod1 =
            mm.wrapping_sub(prod0)
                .wrapping_sub(if mm < prod0 { U256::ONE } else { U256::ZERO });

        Ok((prod0, prod1))
    }

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
        denominator: U256,
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
            let remainder = mulmod(x, y, denominator);

            // Subtract 256 bit number from 512 bit number.
            if remainder > prod0 {
                prod1 = prod1.wrapping_sub(U256::ONE)
            }
            prod0 = prod0.wrapping_sub(remainder);

            // Factor powers of two out of denominator and compute largest power of two divisor of denominator. Always >= 1
            // See https://cs.stackexchange.com/q/138556/92363

            // Does not overflow because the denominator cannot be zero at this stage in the function
            let mut lpotdod = denominator & (!denominator + U256::ONE);
            // println!("lpotdod 0: {:#?}", lpotdod);

            // Divide denominator by lpotdod.
            let denominator = denominator / lpotdod;

            // Divide [prod1 prod0] by lpotdod.
            let prod0 = prod0 / lpotdod;

            // Flip lpotdod such that it is 2^256 / lpotdod. If lpotdod is zero, then it becomes one

            lpotdod = (U256::ZERO.wrapping_sub(lpotdod) / lpotdod).wrapping_add(U256::ONE);

            // Shift in bits from prod1 into prod0
            let prod0 = prod0 | (prod1.wrapping_mul(lpotdod));
            // println!("prod0 bit-shifted: {:#?}", prod0);

            // Invert denominator mod 2^256. Now that denominator is an odd number, it has an inverse modulo 2^256 such
            // that denominator * inv = 1 mod 2^256. Compute the inverse by starting with a seed that is correct for
            // four bits. That is, denominator * inv = 1 mod 2^4
            let mut inverse = U256::from(3u8).wrapping_mul(denominator).bitxor(2);

            // Use the Newton-Raphson iteration to improve the precision. Thanks to Hensel's lifting lemma, this also works
            // in modular arithmetic, doubling the correct bits in each step
            inverse = inverse
                .wrapping_mul(U256::from(2u8).wrapping_sub(denominator.wrapping_mul(inverse))); // inverse mod 2^8
            inverse = inverse
                .wrapping_mul(U256::from(2u8).wrapping_sub(denominator.wrapping_mul(inverse))); // inverse mod 2^16
            inverse = inverse
                .wrapping_mul(U256::from(2u8).wrapping_sub(denominator.wrapping_mul(inverse))); // inverse mod 2^32
            inverse = inverse
                .wrapping_mul(U256::from(2u8).wrapping_sub(denominator.wrapping_mul(inverse))); // inverse mod 2^64
            inverse = inverse
                .wrapping_mul(U256::from(2u8).wrapping_sub(denominator.wrapping_mul(inverse))); // inverse mod 2^128
            inverse = inverse
                .wrapping_mul(U256::from(2u8).wrapping_sub(denominator.wrapping_mul(inverse))); // inverse mod 2^256

            // Because the division is now exact we can divide by multiplying with the modular inverse of denominator.
            // This will give us the correct result modulo 2^256. Since the preconditions guarantee that the outcome is
            // less than 2^256, this is the final result. We don't need to compute the high bits of the result and prod1
            // is no longer required.
            result = prod0.wrapping_mul(inverse);

            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ethnum::U256;

    use super::U256x256Math;
    use crate::constants::{PRECISION, SCALE_OFFSET};

    #[test]
    fn test_get_mul_prods() {
        let x =
            U256::from_str_prefixed("42008768997448919173843294709597899956404323600000").unwrap();
        let y =
            U256::from_str_prefixed("42008768997448919173843294709597899956404323600000").unwrap();
        let mul_prods = U256x256Math::_get_mul_prods(x, y).unwrap();

        let expected = (
            U256::from_str(
                "106836064098642271423399912068207628716993854336347640715097787605024273743872",
            )
            .unwrap(),
            U256::from_str("15240563360629869600998").unwrap(),
        );

        assert_eq!(mul_prods, expected)
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

        let res = U256x256Math::mul_div_round_up(x, y, denominator).unwrap();
        assert_eq!(res, U256::from(10000u128));
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
    fn test_mul_shift_round_down() {
        let x = U256::from_words(1000u128, 1000u128);
        let y = U256::from(PRECISION);
        let shift = SCALE_OFFSET;

        let res = U256x256Math::mul_shift_round_down(x, y, shift).unwrap();
        assert_eq!(res, U256::from(1000000000000000000000u128));
    }

    #[test]
    fn test_mul_shift_round_up() {
        let x = U256::from_words(1000u128, 1000u128);
        let y = U256::from(PRECISION);
        let shift = SCALE_OFFSET;

        let res = U256x256Math::mul_shift_round_up(x, y, shift).unwrap();
        assert_eq!(res, U256::from(1000000000000000000001u128));
    }

    #[test]
    fn test_shift_div_round_down() {
        let x = U256::from(1000u128);
        let shift = 10u8;
        let denominator = U256::from(100u128);

        let shifted = x << shift;
        let (prod0, prod1) = U256x256Math::_get_mul_prods(shifted, U256::ONE).unwrap();

        assert_eq!(prod0, U256::from(1024000u128));
        assert_eq!(prod1, U256::from(0u128));

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

        assert_eq!(prod0, U256::from(1024000u128));
        assert_eq!(prod1, U256::from(0u128));

        if denominator != U256::ZERO {
            if prod1 != U256::ZERO && denominator <= prod1 {
                panic!("Overflow error!");
            } else {
                let res = U256x256Math::shift_div_round_up(x, shift, denominator).unwrap();
                assert_eq!(res, 10240);
            }
        } else {
            panic!("Denominator is zero!");
        }
    }
}
