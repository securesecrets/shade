//! ### Liquidity Book Tree Math Library
//! Author: Kent
//!
//! This module contains functions to encode and decode two u128 into a single Bytes32
//! and interact with the encoded Bytes32.
//!
//! u128 is a 128-bit unsigned integer type, which means that its little-endian byte representation is 16 bytes long.
//! A `Bytes32` value is a `[u8; 32]` and can hold 256 bits, or two `u128` values.

use cosmwasm_std::StdError;

use crate::utils::liquidity_book::types::Bytes32;

pub const BASIS_POINT_MAX: u128 = 10_000;

pub trait Encode {
    fn encode(x1: u128, x2: u128) -> Self;
    fn encode_alt(x1: u128, first: bool) -> Self;
    fn encode_first(x1: u128) -> Self;
    fn encode_second(x2: u128) -> Self;
}

pub trait Decode {
    fn decode(&self) -> (u128, u128);
    fn decode_alt(&self, first: bool) -> u128;
    fn decode_x(&self) -> u128;
    fn decode_y(&self) -> u128;
}

pub trait PackedMath {
    fn add(&self, y: Bytes32) -> Bytes32;
    fn add_alt(&self, y1: u128, y2: u128) -> Bytes32;
    fn sub(&self, y: Bytes32) -> Bytes32;
    fn sub_alt(&self, y1: u128, y2: u128) -> Bytes32;
    fn lt(&self, y: Bytes32) -> bool;
    fn gt(&self, y: Bytes32) -> bool;
    fn scalar_mul_div_basis_point_round_down(&self, multiplier: u128) -> Result<Bytes32, StdError>;
}

impl Encode for Bytes32 {
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
    fn encode(x1: u128, x2: u128) -> Bytes32 {
        let mut z = [0u8; 32];
        let x1_bytes = x1.to_le_bytes();
        let x2_bytes = x2.to_le_bytes();
        for i in 0..16 {
            z[i] = x1_bytes[i];
            z[i + 16] = x2_bytes[i];
        }

        z
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
    fn encode_alt(x: u128, first: bool) -> Bytes32 {
        if first {
            Self::encode_first(x)
        } else {
            Self::encode_second(x)
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
        for i in 0..16 {
            z[i] = x1_bytes[i];
        }
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
        for i in 0..16 {
            z[i + 16] = x2_bytes[i];
        }
        z
    }
}

impl Decode for Bytes32 {
    /// Decodes a `Bytes32` value into two `u128` values as follows:
    /// - [0 - 128[: x1
    /// - [128 - 256[: x2
    ///
    /// # Returns
    ///
    /// A tuple of two `u128` values representing the decoded `x1` and `x2` values.
    fn decode(&self) -> (u128, u128) {
        let mut x1_bytes = [0u8; 16];
        let mut x2_bytes = [0u8; 16];
        for i in 0..16 {
            x1_bytes[i] = self[i];
            x2_bytes[i] = self[i + 16];
        }
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
        let mut x_bytes = [0u8; 16];
        for i in 0..16 {
            x_bytes[i] = self[i];
        }
        u128::from_le_bytes(x_bytes)
    }

    /// Decodes a `Bytes32` value into a `u128` value as the second chunk.
    ///
    /// # Returns
    ///
    /// A `u128` value representing the decoded `x2` value.
    fn decode_y(&self) -> u128 {
        let mut y_bytes = [0u8; 16];
        for i in 0..16 {
            y_bytes[i] = self[i + 16];
        }
        u128::from_le_bytes(y_bytes)
    }
}

impl PackedMath for Bytes32 {
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
    fn add(&self, y: Bytes32) -> Bytes32 {
        let (x1, x2) = self.decode();
        let (y1, y2) = y.decode();
        let z1 = x1.checked_add(y1).expect("Addition overflowed");
        let z2 = x2.checked_add(y2).expect("Addition overflowed");

        Bytes32::encode(z1, z2)
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
    fn add_alt(&self, y1: u128, y2: u128) -> Bytes32 {
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
    fn sub(&self, y: Bytes32) -> Bytes32 {
        let (x1, x2) = self.decode();

        let (y1, y2) = y.decode();

        let z1 = x1.checked_sub(y1).expect("Subtraction underflowed");
        let z2 = x2.checked_sub(y2).expect("Subtraction underflowed");

        Bytes32::encode(z1, z2)
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
    fn sub_alt(&self, y1: u128, y2: u128) -> Bytes32 {
        let y = Bytes32::encode(y1, y2);
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
    fn lt(&self, y: Bytes32) -> bool {
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
    fn gt(&self, y: Bytes32) -> bool {
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
    fn scalar_mul_div_basis_point_round_down(&self, multiplier: u128) -> Result<Bytes32, StdError> {
        if multiplier == 0 {
            return Ok(PackedU128::min());
        }

        if multiplier > BASIS_POINT_MAX {
            return Err(StdError::GenericErr {
                msg: format!(
                    "multiplier: {} > BASIS_POINT_MAX: {}",
                    multiplier, BASIS_POINT_MAX
                ),
            });
        }

        let (x1, x2) = self.decode();

        let z1 = x1 * multiplier / BASIS_POINT_MAX;
        let z2 = x2 * multiplier / BASIS_POINT_MAX;

        Ok(Bytes32::encode(z1, z2))
    }
}

pub struct PackedU128;

impl PackedU128 {
    pub const fn new() -> Bytes32 {
        [0u8; 32]
    }

    pub const fn min() -> Bytes32 {
        [0u8; 32]
    }

    pub const fn max() -> Bytes32 {
        [u8::MAX; 32]
    }

    // NOTE: the rest of these functions can be used by importing the PackedMath trait instead.
    // They are duplicated here just in case.

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
    pub fn add(x: Bytes32, y: Bytes32) -> Bytes32 {
        let (x1, x2) = x.decode();
        let (y1, y2) = y.decode();
        let z1 = x1.checked_add(y1).expect("Addition overflowed");
        let z2 = x2.checked_add(y2).expect("Addition overflowed");

        Bytes32::encode(z1, z2)
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
    pub fn add_alt(x: Bytes32, y1: u128, y2: u128) -> Bytes32 {
        let y = Bytes32::encode(y1, y2);
        Self::add(x, y)
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
    pub fn sub(x: Bytes32, y: Bytes32) -> Bytes32 {
        let (x1, x2) = x.decode();
        let (y1, y2) = y.decode();
        let z1 = x1.checked_sub(y1).expect("Subtraction underflowed");
        let z2 = x2.checked_sub(y2).expect("Subtraction underflowed");

        Bytes32::encode(z1, z2)
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
    pub fn sub_alt(x: [u8; 32], y1: u128, y2: u128) -> [u8; 32] {
        let y = Bytes32::encode(y1, y2);
        Self::sub(x, y)
    }

    /// Returns whether any of the u128 of x is strictly less than the corresponding u128 of y.
    ///
    /// `x` and `y` are both encoded as Bytes32 with the following structure:
    /// - `[0-15]`: first `u128` value
    /// - `[16-31]`: second `u128` value
    ///
    /// # Arguments
    ///
    /// * `x` - A Bytes32 encoding a pair of `u128` values.
    /// * `y` - A Bytes32 encoding a pair of `u128` values.
    ///
    /// # Returns
    ///
    /// * `true` if `x` is less than `y`, and `false` otherwise.
    pub fn lt(x: Bytes32, y: Bytes32) -> bool {
        let (x1, x2) = x.decode();
        let (y1, y2) = y.decode();

        x1 < y1 || x2 < y2
    }

    /// Returns whether any of the u128 of x is strictly greater than the corresponding u128 of y.
    ///
    /// `x` and `y` are both encoded as Bytes32 with the following structure:
    /// - `[0-15]`: first `u128` value
    /// - `[16-31]`: second `u128` value
    ///
    /// # Arguments
    ///
    /// * `x` - A Bytes32 encoding a pair of `u128` values.
    /// * `y` - A Bytes32 encoding a pair of `u128` values.
    ///
    /// # Returns
    ///
    /// * `true` if `x` is greater than `y`, and `false` otherwise.
    pub fn gt(x: Bytes32, y: Bytes32) -> bool {
        let (x1, x2) = x.decode();
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
    pub fn scalar_mul_div_basis_point_round_down(
        x: Bytes32,
        multiplier: u128,
    ) -> Result<Bytes32, StdError> {
        if multiplier == 0 {
            return Ok([0u8; 32]);
        }

        if multiplier > BASIS_POINT_MAX {
            return Err(StdError::GenericErr {
                msg: format!(
                    "multiplier: {} > BASIS_POINT_MAX: {}",
                    multiplier, BASIS_POINT_MAX
                ),
            });
        }

        let (x1, x2) = x.decode();

        let z1 = x1 * multiplier / BASIS_POINT_MAX;
        let z2 = x2 * multiplier / BASIS_POINT_MAX;

        Ok(Bytes32::encode(z1, z2))
    }
}
