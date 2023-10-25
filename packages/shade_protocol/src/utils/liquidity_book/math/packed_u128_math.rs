//! ### Liquidity Book Tree Math Library
//! Author: Kent and Haseeb
//!
//! This module contains functions to encode and decode two u128 into a single Bytes32
//! and interact with the encoded Bytes32.
//!
//! u128 is a 128-bit unsigned integer type, which means that its little-endian byte representation is 16 bytes long.
//! A `Bytes32` value is a `[u8; 32]` and can hold 256 bits, or two `u128` values.

use crate::utils::liquidity_book::types::Bytes32;
use cosmwasm_std::StdError;

pub const BASIS_POINT_MAX: u128 = 10_000;

impl PackedUint128Math for Bytes32 {}

pub trait PackedUint128Math: From<[u8; 32]> + AsRef<[u8]> {
    fn min() -> Self {
        [0u8; 32].into()
    }

    fn max() -> Self {
        [u8::MAX; 32].into()
    }

    /// Encodes two `u128` values into a single `Bytes32` value, with the first `u128`
    /// value stored in the first 128 bits of the `Bytes32` value and the second `u128`
    /// value stored in the last 128 bits.
    ///
    /// # Arguments
    ///
    /// * `x1` - The first `u128` value to encode.
    /// * `x2` - The second `u128` value to encode.
    ///
    /// # Returns
    ///
    /// The encoded `Bytes32` value, with `x1` stored in the first 128 bits and `x2`
    /// stored in the last 128 bits.
    fn encode(x1: u128, x2: u128) -> Self {
        let mut z = [0u8; 32];
        let x1_bytes = x1.to_le_bytes();
        let x2_bytes = x2.to_le_bytes();

        z[..16].copy_from_slice(&x1_bytes[..16]);
        z[16..32].copy_from_slice(&x2_bytes[..16]);

        z.into()
    }

    /// Encodes a `u128` value into a single `Bytes32` value, with the `u128` value stored
    /// in either the first or last 128 bits of the `Bytes32` value. The remaining 128 bits are set to zero.
    ///
    /// # Arguments
    ///
    /// * `x` - The `u128` value to encode.
    /// * `first` - Whether to encode as the first or second u128.
    ///
    /// # Returns
    ///
    /// The encoded `Bytes32` value, with the `u128` value stored in either the first
    /// or last 128 bits of the `Bytes32` value. The remaining 128 bits are set to zero.
    fn encode_alt(x: u128, first: bool) -> Self {
        if first {
            Self::encode_first(x).into()
        } else {
            Self::encode_second(x).into()
        }
    }

    /// Encodes a `u128` value into a single `Bytes32` value, with the `u128` value stored
    /// in the first 128 bits of the `Bytes32` value and the last 128 bits set to zero.
    ///
    /// # Arguments
    ///
    /// * `x1` - The `u128` value to encode.
    ///
    /// # Returns
    ///
    /// The encoded `Bytes32` value, with `x1` stored in the first 128 bits and the last
    /// 128 bits set to zero.
    fn encode_first(x1: u128) -> Bytes32 {
        let mut z = [0u8; 32];
        let x1_bytes = x1.to_le_bytes();
        z[..16].copy_from_slice(&x1_bytes[..16]);

        z
    }

    /// Encodes a `u128` value into a single `Bytes32` value, with the `u128` value stored
    /// in the last 128 bits of the `Bytes32` value and the first 128 bits set to zero.
    ///
    /// # Arguments
    ///
    /// * `x2` - The `u128` value to encode.
    ///
    /// # Returns
    ///
    /// The encoded `Bytes32` value, with `x2` stored in the last 128 bits and the first
    /// 128 bits set to zero.
    fn encode_second(x2: u128) -> Bytes32 {
        let mut z = [0u8; 32];
        let x2_bytes = x2.to_le_bytes();
        z[16..32].copy_from_slice(&x2_bytes[..16]);

        z
    }

    /// Decodes a `Bytes32` value into two `u128` values as follows:
    /// - [0 - 128[: x1
    /// - [128 - 256[: x2
    ///
    /// # Returns
    ///
    /// A tuple of two `u128` values representing the decoded `x1` and `x2` values.
    fn decode(&self) -> (u128, u128) {
        let bytes = self.as_ref();
        let mut x1_bytes = [0u8; 16];
        let mut x2_bytes = [0u8; 16];
        x1_bytes[..16].copy_from_slice(&bytes[..16]);
        x2_bytes[..16].copy_from_slice(&bytes[16..32]);
        let x1 = u128::from_le_bytes(x1_bytes);
        let x2 = u128::from_le_bytes(x2_bytes);

        (x1, x2)
    }

    /// Decodes a `Bytes32` value into a `u128` value as either the first or second chunk.
    ///
    /// # Arguments
    ///
    /// * `first` - A boolean value indicating whether to decode the first chunk (`true`) or the second chunk (`false`)
    ///
    /// # Returns
    ///
    /// A `u128` value representing the decoded `x1` or `x2` value, depending on the value of `first`.
    fn decode_alt(&self, first: bool) -> u128 {
        if first {
            Self::decode_x(self)
        } else {
            Self::decode_y(self)
        }
    }

    /// Decodes a `Bytes32` value into a `u128` value as the first chunk.
    ///
    /// # Returns
    ///
    /// A `u128` value representing the decoded `x1` value.
    fn decode_x(&self) -> u128 {
        let bytes = self.as_ref();
        let mut x_bytes = [0u8; 16];
        x_bytes[..16].copy_from_slice(&bytes[..16]);
        u128::from_le_bytes(x_bytes)
    }

    /// Decodes a `Bytes32` value into a `u128` value as the second chunk.
    ///
    /// # Returns
    ///
    /// A `u128` value representing the decoded `x2` value.
    fn decode_y(&self) -> u128 {
        let bytes = self.as_ref();
        let mut y_bytes = [0u8; 16];
        y_bytes[..16].copy_from_slice(&bytes[16..32]);
        u128::from_le_bytes(y_bytes)
    }

    /// Adds two `Bytes32` values encoded as follows:
    /// - [0 - 128[: x1
    /// - [128 - 256[: x2
    /// - [0 - 128[: y1
    /// - [128 - 256[: y2
    ///
    /// # Arguments
    ///
    /// * `x` - The first `Bytes32` value represented as a `[u8; 32]` array
    /// * `y` - The second `Bytes32` value represented as a `[u8; 32]` array
    ///
    /// # Returns
    ///
    /// A `Bytes32` value represented as a `[u8; 32]` array, encoding the sum of `x` and `y` as follows:
    /// - [0 - 128[: x1 + y1
    /// - [128 - 256[: x2 + y2
    ///
    /// # Panics
    ///
    /// This function panics if the addition overflows.
    fn add(&self, y: Bytes32) -> Self {
        let (x1, x2) = self.decode();
        let (y1, y2) = y.decode();
        let z1 = x1.checked_add(y1).expect("Addition overflowed");
        let z2 = x2.checked_add(y2).expect("Addition overflowed");

        Self::encode(z1, z2)
    }

    /// Adds a `Bytes32` value encoded as follows:
    /// - [0 - 128[: x1
    /// - [128 - 256[: x2
    /// - and two `u128` values `y1` and `y2`.
    ///
    /// # Arguments
    ///
    /// * `x` - The `Bytes32` value represented as a `[u8; 32]` array
    /// * `y1` - The first `u128` value
    /// * `y2` - The second `u128` value
    ///
    /// # Returns
    ///
    /// A `Bytes32` value represented as a `[u8; 32]` array, encoding the sum of `x` and `(y1, y2)` as follows:
    /// - [0 - 128[: x1 + y1
    /// - [128 - 256[: x2 + y2
    ///
    /// # Panics
    ///
    /// This function panics if the addition overflows.
    fn add_alt(&self, y1: u128, y2: u128) -> Self {
        let y = Bytes32::encode(y1, y2);
        Self::add(self, y)
    }

    /// Subtracts two `Bytes32` values encoded as follows:
    /// - [0 - 128[: x1
    /// - [128 - 256[: x2
    /// - [0 - 128[: y1
    /// - [128 - 256[: y2
    ///
    /// # Arguments
    ///
    /// * `x` - The first `Bytes32` value represented as a `[u8; 32]` array
    /// * `y` - The second `Bytes32` value represented as a `[u8; 32]` array
    ///
    /// # Returns
    ///
    /// A `Bytes32` value represented as a `[u8; 32]` array, encoding the difference between `x` and `y` as follows:
    /// - [0 - 128[: x1 - y1
    /// - [128 - 256[: x2 - y2
    ///
    /// # Panics
    ///
    /// This function panics if the subtraction underflows.
    fn sub(&self, y: Self) -> Self {
        let (x1, x2) = self.decode();
        let (y1, y2) = y.decode();

        let z1 = x1.checked_sub(y1).expect("Subtraction underflowed");
        let z2 = x2.checked_sub(y2).expect("Subtraction underflowed");

        Self::encode(z1, z2)
    }

    /// Subtracts a `Bytes32` value encoded as follows:
    /// - [0 - 128[: x1
    /// - [128 - 256[: x2
    /// - and two `u128` values `y1` and `y2`.
    ///
    /// # Arguments
    ///
    /// * `x` - The `Bytes32` value represented as a `[u8; 32]` array
    /// * `y1` - The first `u128` value
    /// * `y2` - The second `u128` value
    ///
    /// # Returns
    ///
    /// A `Bytes32` value represented as a `[u8; 32]` array, encoding the difference between `x` and `(y1, y2)` as follows:
    /// - [0 - 128[: x1 - y1
    /// - [128 - 256[: x2 - y2
    ///
    /// # Panics
    ///
    /// This function panics if the subtraction underflows.
    fn sub_alt(&self, y1: u128, y2: u128) -> Self {
        let y = Self::encode(y1, y2);
        Self::sub(self, y)
    }

    /// Returns whether any of the u128 of x is strictly less than the corresponding u128 of y.
    ///
    /// `x` and `y` are both encoded as `Bytes32` with the following structure:
    /// - `[0-15]`: first `u128` value
    /// - `[16-31]`: second `u128` value
    ///
    /// # Arguments
    ///
    /// * `x` - A `Bytes32` encoding a pair of `u128` values.
    /// * `y` - A `Bytes32` encoding a pair of `u128` values.
    ///
    /// # Returns
    ///
    /// * `true` if `x` is less than `y`, and `false` otherwise.
    fn lt(&self, y: Self) -> bool {
        let (x1, x2) = self.decode();
        let (y1, y2) = y.decode();

        x1 < y1 || x2 < y2
    }

    /// Returns whether any of the u128 of x is strictly greater than the corresponding u128 of y.
    ///
    /// `x` and `y` are both encoded as `Bytes32` with the following structure:
    /// - `[0-15]`: first `u128` value
    /// - `[16-31]`: second `u128` value
    ///
    /// # Arguments
    ///
    /// * `x` - A `Bytes32` encoding a pair of `u128` values.
    /// * `y` - A `Bytes32` encoding a pair of `u128` values.
    ///
    /// # Returns
    ///
    /// * `true` if `x` is greater than `y`, and `false` otherwise.
    fn gt(&self, y: Self) -> bool {
        let (x1, x2) = self.decode();
        let (y1, y2) = y.decode();

        x1 > y1 || x2 > y2
    }

    /// Multiplies an encoded Bytes32 by a u128 then divides the result by 10_000, rounding down.
    ///
    /// The result can't overflow as the multiplier needs to be smaller or equal to 10_000.
    ///
    /// # Arguments
    ///
    /// * `x` - The Bytes32 encoded as follows:
    ///     * `[0 - 128[` : x1
    ///     * `[128 - 256[` : x2
    /// * `multiplier` - The u128 to multiply by (must be smaller or equal to 10_000).
    ///
    /// # Returns
    ///
    /// Returns the product of x and multiplier encoded as follows:
    /// * `[0 - 128[` : floor((x1 * multiplier) / 10_000)
    /// * `[128 - 256[` : floor((x2 * multiplier) / 10_000)
    ///
    /// # Panics
    ///
    /// This function will panic if the `multiplier` argument is larger than the constant `BASIS_POINT_MAX`.
    fn scalar_mul_div_basis_point_round_down(&self, multiplier: u128) -> Result<Self, StdError> {
        if multiplier == 0 {
            return Ok(Self::min());
        }

        // TODO - Consider removing this. I think the check happens elsewhere.
        if multiplier > BASIS_POINT_MAX {
            return Err(StdError::GenericErr {
                msg: format!(
                    "multiplier: {} > BASIS_POINT_MAX: {}",
                    multiplier, BASIS_POINT_MAX
                ),
            });
        }

        let (x1, x2) = self.decode();

        // TODO - Is there a chance this overflows during the calculation?
        let z1 = x1 * multiplier / BASIS_POINT_MAX;
        let z2 = x2 * multiplier / BASIS_POINT_MAX;

        Ok(Self::encode(z1, z2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        // Test typical case
        let x1: u128 = 42;
        let x2: u128 = 24;
        let encoded = Bytes32::encode(x1, x2);
        let expected: [u8; 32] = [
            42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 24, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        assert_eq!(encoded, expected);

        // Test edge case: Zero values
        let encoded_zero = Bytes32::encode(0, 0);
        let expected_zero: [u8; 32] = [0; 32];
        assert_eq!(encoded_zero, expected_zero);

        // Test edge case: Maximum values
        let max_value = u128::MAX;
        let encoded_max = Bytes32::encode(max_value, max_value);
        let expected_max: [u8; 32] = [255; 32];
        assert_eq!(encoded_max, expected_max);
    }

    #[test]
    fn test_encode_alt() {
        // Test case: Storing the value in the first 128 bits
        let x1: u128 = 56;
        let encoded_first = Bytes32::encode_alt(x1, true);
        let expected_first: [u8; 32] = [
            56, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];
        assert_eq!(encoded_first, expected_first);

        // Test case: Storing the value in the last 128 bits
        let x2: u128 = 65;
        let encoded_second = Bytes32::encode_alt(x2, false);
        let expected_second: [u8; 32] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 65, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ];
        assert_eq!(encoded_second, expected_second);
    }

    #[test]
    fn test_encode_first() {
        // Test case: Storing the value 100 in the first 128 bits
        let x: u128 = 100;
        let encoded = Bytes32::encode_first(x);
        let expected: [u8; 32] = [
            100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        assert_eq!(encoded, expected);

        // Edge case: Storing the value 0 in the first 128 bits
        let x: u128 = 0;
        let encoded = Bytes32::encode_first(x);
        let expected: [u8; 32] = [0; 32]; // All zeros
        assert_eq!(encoded, expected);

        // Edge case: Storing the maximum u128 value in the first 128 bits
        let x: u128 = u128::MAX;
        let encoded = Bytes32::encode_first(x);
        let mut expected: [u8; 32] = [0; 32];
        expected[0..16].copy_from_slice(&x.to_le_bytes());
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_encode_second() {
        // Test case: Storing the value 100 in the last 128 bits
        let x: u128 = 100;
        let encoded = Bytes32::encode_second(x);
        let expected: [u8; 32] = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        assert_eq!(encoded, expected);

        // Edge case: Storing the value 0 in the last 128 bits
        let x: u128 = 0;
        let encoded = Bytes32::encode_second(x);
        let expected: [u8; 32] = [0; 32]; // All zeros
        assert_eq!(encoded, expected);

        // Edge case: Storing the maximum u128 value in the last 128 bits
        let x: u128 = u128::MAX;
        let encoded = Bytes32::encode_second(x);
        let mut expected: [u8; 32] = [0; 32];
        expected[16..32].copy_from_slice(&x.to_le_bytes());
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_decode() {
        // Test case 1: Decode a Bytes32 with x1 = 100 and x2 = 200
        let bytes: Bytes32 = [
            100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 200, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let (x1, x2) = Bytes32::decode(&bytes);
        assert_eq!(x1, 100);
        assert_eq!(x2, 200);

        // Edge case: Decode a Bytes32 with all zeros
        let bytes: Bytes32 = [0; 32];
        let (x1, x2) = Bytes32::decode(&bytes);
        assert_eq!(x1, 0);
        assert_eq!(x2, 0);

        // Edge case: Decode a Bytes32 with all maximum u8 values
        let bytes: Bytes32 = [u8::MAX; 32];
        let (x1, x2) = Bytes32::decode(&bytes);
        assert_eq!(x1, u128::MAX);
        assert_eq!(x2, u128::MAX);
    }

    #[test]
    fn test_decode_alt() {
        // Test case 1: Decode a Bytes32 with x1 = 100 and x2 = 200
        let bytes: Bytes32 = [
            100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 200, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0,
        ];
        let val = Bytes32::decode_alt(&bytes, true);
        assert_eq!(val, 100);
        let val = Bytes32::decode_alt(&bytes, false);
        assert_eq!(val, 200);

        // Edge case: Decode a Bytes32 with all zeros
        let bytes: Bytes32 = [0; 32];
        let val = Bytes32::decode_alt(&bytes, true);
        assert_eq!(val, 0);
        let val = Bytes32::decode_alt(&bytes, false);
        assert_eq!(val, 0);

        // Edge case: Decode a Bytes32 with all maximum u8 values
        let bytes: Bytes32 = [u8::MAX; 32];
        let val = Bytes32::decode_alt(&bytes, true);
        assert_eq!(val, u128::MAX);
        let val = Bytes32::decode_alt(&bytes, false);
        assert_eq!(val, u128::MAX);
    }

    #[test]
    fn test_decode_x() {
        // Test case 1: Basic test case
        let bytes = Bytes32::encode(15, 25); // 15 is in the x position
        assert_eq!(Bytes32::decode_x(&bytes), 15);

        // Test case 2: Test with max value
        let bytes = Bytes32::encode(u128::MAX, 10);
        assert_eq!(Bytes32::decode_x(&bytes), u128::MAX);

        // Test case 3: Test with zero in x position
        let bytes = Bytes32::encode(0, 10);
        assert_eq!(Bytes32::decode_x(&bytes), 0);
    }

    #[test]
    fn test_decode_y() {
        // Test case 1: Basic test case
        let bytes = Bytes32::encode(15, 25); // 25 is in the y position
        assert_eq!(Bytes32::decode_y(&bytes), 25);

        // Test case 2: Test with max value
        let bytes = Bytes32::encode(10, u128::MAX);
        assert_eq!(Bytes32::decode_y(&bytes), u128::MAX);

        // Test case 3: Test with zero in y position
        let bytes = Bytes32::encode(10, 0);
        assert_eq!(Bytes32::decode_y(&bytes), 0);
    }

    #[test]
    fn test_add() {
        // Basic Test Case
        let bytes1 = Bytes32::encode(15, 25);
        let bytes2 = Bytes32::encode(10, 20);
        let result = Bytes32::add(&bytes1, bytes2);
        assert_eq!(result.decode(), (25, 45));

        // Max Value Test Case
        let bytes1 = Bytes32::encode(u128::MAX - 1, 0);
        let bytes2 = Bytes32::encode(1, 0);
        let result = Bytes32::add(&bytes1, bytes2);
        assert_eq!(result.decode(), (u128::MAX, 0));

        // Zero Test Case
        let bytes1 = Bytes32::encode(15, 25);
        let bytes2 = Bytes32::encode(0, 0);
        let result = Bytes32::add(&bytes1, bytes2);
        assert_eq!(result.decode(), (15, 25));
    }

    #[test]
    fn test_add_alt() {
        // Basic Test Case
        let bytes1 = Bytes32::encode(15, 25);
        let result = Bytes32::add_alt(&bytes1, 10, 20);
        assert_eq!(result.decode(), (25, 45));

        // Max Value Test Case
        let bytes1 = Bytes32::encode(u128::MAX - 1, 0);
        let result = Bytes32::add_alt(&bytes1, 1, 0);
        assert_eq!(result.decode(), (u128::MAX, 0));

        // Zero Test Case
        let bytes1 = Bytes32::encode(15, 25);
        let result = Bytes32::add_alt(&bytes1, 0, 0);
        assert_eq!(result.decode(), (15, 25));
    }

    #[test]
    fn test_sub() {
        // Basic Test Case
        let bytes1 = Bytes32::encode(25, 45);
        let bytes2 = Bytes32::encode(10, 20);
        let result = Bytes32::sub(&bytes1, bytes2);
        assert_eq!(result.decode(), (15, 25));

        // Underflow Test Case
        let bytes1 = Bytes32::encode(0, 0);
        let bytes2 = Bytes32::encode(10, 20);
        // This should panic
        // let result = Bytes32::sub(&bytes1, bytes2);

        // Zero Test Case
        let bytes1 = Bytes32::encode(25, 45);
        let bytes2 = Bytes32::encode(0, 0);
        let result = Bytes32::sub(&bytes1, bytes2);
        assert_eq!(result.decode(), (25, 45));
    }

    #[test]
    fn test_sub_alt() {
        // Basic Test Case
        let bytes1 = Bytes32::encode(25, 45);
        let result = Bytes32::sub_alt(&bytes1, 10, 20);
        assert_eq!(result.decode(), (15, 25));

        // Zero Test Case
        let bytes1 = Bytes32::encode(25, 45);
        let result = Bytes32::sub_alt(&bytes1, 0, 0);
        assert_eq!(result.decode(), (25, 45));
    }

    #[test]
    #[should_panic]
    fn test_sub_alt_panic() {
        // Underflow Test Case
        let bytes1 = Bytes32::encode(0, 0);
        // This should panic
        let result = Bytes32::sub_alt(&bytes1, 10, 20);
    }

    #[test]
    fn test_lt() {
        // True Case
        let bytes1 = Bytes32::encode(10, 20);
        let bytes2 = Bytes32::encode(25, 45);
        assert!(<[u8; 32] as PackedUint128Math>::lt(&bytes1, bytes2));

        // False Case
        let bytes1 = Bytes32::encode(25, 45);
        let bytes2 = Bytes32::encode(10, 20);
        assert!(!<[u8; 32] as PackedUint128Math>::lt(&bytes1, bytes2));

        // Equal Case
        let bytes1 = Bytes32::encode(10, 20);
        let bytes2 = Bytes32::encode(10, 20);
        assert!(!<[u8; 32] as PackedUint128Math>::lt(&bytes1, bytes2));
    }

    #[test]
    fn test_gt() {
        // True Case
        let bytes1 = Bytes32::encode(25, 45);
        let bytes2 = Bytes32::encode(10, 20);
        assert!(<[u8; 32] as PackedUint128Math>::gt(&bytes1, bytes2));
        // False Case
        let bytes1 = Bytes32::encode(10, 20);
        let bytes2 = Bytes32::encode(25, 45);
        assert!(!<[u8; 32] as PackedUint128Math>::gt(&bytes1, bytes2));
        // Equal Case
        let bytes1 = Bytes32::encode(25, 45);
        let bytes2 = Bytes32::encode(25, 45);
        assert!(!<[u8; 32] as PackedUint128Math>::gt(&bytes1, bytes2));
    }
}
