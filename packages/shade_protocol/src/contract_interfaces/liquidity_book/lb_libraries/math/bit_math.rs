//! ### Liquidity Book Bit Math Library
//! Author: Kent
//!
//! Helper library used for bit calculations.

use ethnum::U256;

pub struct BitMath;

impl BitMath {
    // I don't understand why this returns the U256::MAX value instead of u8::MAX
    /// Returns the index of the closest bit on the right of x that is non null.
    /// If there is no closest bit, it returns `U256::MAX`.
    ///
    /// # Arguments
    ///
    /// * `x` - The value as a uint256.
    /// * `bit` - The index of the bit to start searching at.
    pub fn closest_bit_right(x: U256, bit: u8) -> U256 {
        let shift = 255 - bit;
        let x = x << shift;

        if x == U256::ZERO {
            U256::MAX
        } else {
            U256::from(Self::most_significant_bit(x)) - U256::from(shift)
        }
    }

    // I don't understand why this returns the U256::MAX value instead of u8::MAX
    /// Returns the index of the closest bit on the left of x that is non null.
    ///
    /// If there is no closest bit, it returns `U256::MAX`.
    ///
    /// # Arguments
    ///
    /// * `x` - The value as a uint256.
    /// * `bit` - The index of the bit to start searching at.
    pub fn closest_bit_left(x: U256, bit: u8) -> U256 {
        let x = x >> bit;

        if x == U256::ZERO {
            U256::MAX
        } else {
            U256::from(Self::least_significant_bit(x)) + U256::from(bit)
        }
    }

    /// Returns the index of the most significant bit of x.
    ///
    /// This function returns 0 if x is 0.
    ///
    /// # Arguments
    ///
    /// * `x` - The value as a uint256.
    pub fn most_significant_bit(x: U256) -> u8 {
        if x == 0 {
            return 0u8;
        }
        255u8 - x.leading_zeros() as u8
    }

    /// Returns the index of the least significant bit of x.
    ///
    /// This function returns 255 if x is 0.
    ///
    /// # Arguments
    ///
    /// * `x` - The value as a uint256.
    pub fn least_significant_bit(x: U256) -> u8 {
        if x == 0 {
            return 255u8;
        }
        x.trailing_zeros() as u8
    }
}

#[cfg(test)]
mod tests {
    use super::BitMath;
    use ethnum::U256;

    #[test]
    fn test_closest_bit_right() {
        let x = U256::from(0b1011_0010u32);
        let bit = 2;
        let result = BitMath::closest_bit_right(x, bit);
        assert_eq!(result, U256::ONE);
    }

    #[test]
    fn test_closest_bit_right_no_bit() {
        let x = U256::from(0b1000_0000u32);
        let bit = 0;
        let result = BitMath::closest_bit_right(x, bit);
        assert_eq!(result, U256::MAX);
    }

    #[test]
    fn test_closest_bit_left() {
        let x = U256::from(0b0101_1000u32);
        let bit = 1;
        let result = BitMath::closest_bit_left(x, bit);
        assert_eq!(result, U256::from(3u32));
    }

    #[test]
    fn test_closest_bit_left_no_bit() {
        let x = U256::from(0b0000_1000u32);
        let bit = 7;
        let result = BitMath::closest_bit_left(x, bit);
        assert_eq!(result, U256::MAX);
    }

    #[test]
    fn test_most_significant_bit() {
        let x = U256::from(0xffffffffffffffffu128);
        let result = BitMath::most_significant_bit(x);
        assert_eq!(result, 63u8);
    }

    #[test]
    fn test_most_significant_bit_zero() {
        let x = U256::ZERO;
        let result = BitMath::most_significant_bit(x);
        assert_eq!(result, 0u8);
    }

    #[test]
    fn test_least_significant_bit() {
        let x = U256::from(0b0000_1000u32);
        let result = BitMath::least_significant_bit(x);
        assert_eq!(result, 3u8);
    }

    #[test]
    fn test_least_significant_bit_zero() {
        let x = U256::from(0b0000_0000u32);
        let result = BitMath::least_significant_bit(x);
        assert_eq!(result, 255u8);
    }
}

#[cfg(test)]
mod tests2 {
    use super::BitMath;
    use ethnum::U256;

    #[test]
    fn test_closest_bit_right() {
        for i in 0..256u32 {
            assert_eq!(
                BitMath::closest_bit_right(U256::from(1u32) << i, 255),
                U256::from(i),
                "test_ClosestBitRight::1"
            );
        }
    }

    #[test]
    fn test_closest_bit_left() {
        for i in 0..256u32 {
            assert_eq!(
                BitMath::closest_bit_left(U256::from(1u32) << i, 0),
                U256::from(i),
                "test_ClosestBitLeft::1"
            );
        }
    }

    #[test]
    fn test_most_significant_bit() {
        for i in 0..256u32 {
            assert_eq!(
                BitMath::most_significant_bit(U256::from(1u32) << i),
                i as u8,
                "test_MostSignificantBit::1"
            );
        }
    }

    #[test]
    fn test_least_significant_bit() {
        for i in 0..256u32 {
            assert_eq!(
                BitMath::least_significant_bit(U256::from(1u32) << i),
                i as u8,
                "test_LeastSignificantBit::1"
            );
        }
    }
}
