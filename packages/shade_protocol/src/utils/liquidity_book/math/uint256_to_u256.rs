//! ### Liquidity Book ConvertUint256 Math Library
//! Author: Haseeb
//!

use cosmwasm_std::{Uint128, Uint256};
use ethnum::U256;
use std::convert::TryInto;

pub trait ConvertUint256 {
    fn split_uint256(&self) -> (Uint128, Uint128);
    fn uint256_to_u256(&self) -> U256;
}

pub trait ConvertU256 {
    fn split_u256(&self) -> (u128, u128);
    fn u256_to_uint256(&self) -> Uint256;
}

impl ConvertUint256 for Uint256 {
    fn split_uint256(&self) -> (Uint128, Uint128) {
        let bytes = self.to_be_bytes();
        let lower_bytes = &bytes[16..32];
        let upper_bytes = &bytes[0..16];

        let lower = Uint128::new(u128::from_be_bytes(
            lower_bytes.try_into().expect("split_uint256 lower"),
        ));
        let upper = Uint128::new(u128::from_be_bytes(
            upper_bytes.try_into().expect("split_uint256 upper"),
        ));

        (upper, lower)
    }

    fn uint256_to_u256(&self) -> U256 {
        let (upper, lower) = self.split_uint256();

        U256::from_words(upper.u128(), lower.u128())
    }
}

impl ConvertU256 for U256 {
    fn split_u256(&self) -> (u128, u128) {
        let bytes = self.to_be_bytes();
        let lower_bytes = &bytes[16..32];
        let upper_bytes = &bytes[0..16];

        let lower = u128::from_be_bytes(lower_bytes.try_into().expect("split_u256 lower"));
        let upper = u128::from_be_bytes(upper_bytes.try_into().expect("split_u256 upper"));

        (upper, lower)
    }

    fn u256_to_uint256(&self) -> Uint256 {
        let (upper, lower) = self.split_u256();
        let upper_uint256 = Uint256::from(upper) << 128;
        let lower_uint256 = Uint256::from(lower);

        upper_uint256 + lower_uint256
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{Uint128, Uint256};
    use ethnum::U256;
    use std::str::FromStr;

    #[test]
    fn test_split_uint256() {
        let value = Uint256::from_str("340282366920938463463374607431768211455").unwrap(); // Maximum u128
        let (upper, lower) = value.split_uint256();
        assert_eq!(Uint128::from(0u128), upper);
        assert_eq!(Uint128::MAX, lower);
        let combined = (Uint256::from(upper) << 128) + (Uint256::from(lower));
        assert_eq!(value, combined);
    }

    #[test]
    fn test_uint256_to_u256() {
        let value = Uint256::from_str("340282366920938463463374607431768211455").unwrap(); // Maximum u128
        let result = value.uint256_to_u256();
        assert_eq!(
            U256::from_str("340282366920938463463374607431768211455").unwrap(),
            result
        );
    }

    #[test]
    fn test_split_u256() {
        let value = U256::from_str("340282366920938463463374607431768211455").unwrap(); // Maximum u128
        let (upper, lower) = value.split_u256();
        assert_eq!(0, upper);
        assert_eq!(u128::MAX, lower);
        let combined = (U256::from(upper) << 128) + (U256::from(lower));
        assert_eq!(value, combined);
    }

    #[test]
    fn test_u256_to_uint256() {
        let value = U256::from_str("340282366920938463463374607431768211455").unwrap(); // Maximum u128
        let result = value.u256_to_uint256();
        assert_eq!(
            Uint256::from_str("340282366920938463463374607431768211455").unwrap(),
            result
        );
    }
    #[test]
    fn test_split_uint256_max() {
        let value = Uint256::MAX;
        let (upper, lower) = value.split_uint256();
        assert_eq!(Uint128::MAX, upper);
        assert_eq!(Uint128::MAX, lower);
        let combined = (Uint256::from(upper) << 128) + (Uint256::from(lower));
        assert_eq!(value, combined);
    }

    #[test]
    fn test_uint256_to_u256_max() {
        let value = Uint256::MAX;
        let result = value.uint256_to_u256();
        assert_eq!(U256::MAX, result);
    }

    #[test]
    fn test_split_u256_max() {
        let value = U256::MAX;
        let (upper, lower) = value.split_u256();
        assert_eq!(u128::MAX, upper);
        assert_eq!(u128::MAX, lower);
        let combined = (U256::from(upper) << 128) + (U256::from(lower));
        assert_eq!(value, combined);
    }

    #[test]
    fn test_u256_to_uint256_max() {
        let value = U256::MAX;
        let result = value.u256_to_uint256();
        assert_eq!(Uint256::MAX, result);
    }
}
