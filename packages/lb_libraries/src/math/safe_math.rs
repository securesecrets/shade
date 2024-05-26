use ethnum::U256;

use crate::pair_parameter_helper::PairParametersError;

use super::{u128x128_math::U128x128MathError, u24::U24, u256x256_math::U256x256MathError};

#[derive(thiserror::Error, Debug)]
pub enum SafeError {
    #[error("Value greater than u24")]
    U24Overflow,

    #[error(transparent)]
    PairParametersError(#[from] PairParametersError),

    #[error(transparent)]
    U128x128MathError(#[from] U128x128MathError),

    #[error(transparent)]
    U256x256MathError(#[from] U256x256MathError),
}

impl Safe for u128 {}
impl Safe for u32 {}
impl Safe for U256 {}

pub trait Safe {
    fn safe24<T: Into<u32> + Copy, F>(x: T, err: F) -> Result<T, F> {
        if x.into() > U24::MAX {
            return Err(err);
        }
        Ok(x)
    }

    fn safe128<T: Into<u128> + Copy, F>(x: T, err: F) -> Result<T, F> {
        if x.into() > u128::MAX {
            return Err(err);
        }
        Ok(x)
    }
}

#[cfg(test)]
mod tests {
    use ethnum::U256;

    use super::*;

    #[test]
    fn test_safe24_within_bounds() {
        let value: U256 = U256::from(1_000_000u128); // Well within U24::MAX
        assert_eq!(
            u32::safe24(value.as_u32(), U128x128MathError::IdShiftOverflow).unwrap(),
            value.as_u32()
        );

        let value: u32 = 1_000_000; // Well within U24::MAX
        assert_eq!(
            u32::safe24(value, U128x128MathError::IdShiftOverflow).unwrap(),
            value
        );

        let value: u16 = 65_535; // Maximum value for u16, also within U24::MAX
        assert_eq!(
            u32::safe24(value, U128x128MathError::IdShiftOverflow).unwrap(),
            value
        );

        let value: u8 = 255; // Maximum value for u8, within U24::MAX
        assert_eq!(
            u32::safe24(value, U128x128MathError::IdShiftOverflow).unwrap(),
            value
        );
    }

    #[test]
    fn test_safe24_at_bounds() {
        let value = U24::MAX; // Exactly at the boundary
        assert_eq!(
            u32::safe24(value, U128x128MathError::IdShiftOverflow).unwrap(),
            value
        );
    }

    #[test]
    fn test_safe24_above_bounds() {
        let value: u32 = U24::MAX + 1; // Just above the valid range
        assert!(u32::safe24(value, U128x128MathError::IdShiftOverflow).is_err());

        let value: u32 = u32::MAX; // Maximum u32 value, well above U24::MAX
        assert!(u32::safe24(value, U128x128MathError::IdShiftOverflow).is_err());
    }
}
