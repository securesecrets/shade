//! ### Liquidity Book Encoded Library
//! Author: Kent and Haseeb
//!
//! Helper library used for setting and decoding parts of encoded Bytes32.

use crate::liquidity_book::lb_libraries::types::Bytes32;
use cosmwasm_schema::cw_serde;
use ethnum::U256;

pub const MASK_UINT1: U256 = U256::new(0x1u128);
pub const MASK_UINT8: U256 = U256::new(0xffu128);
pub const MASK_UINT12: U256 = U256::new(0xfffu128);
pub const MASK_UINT14: U256 = U256::new(0x3fffu128);
pub const MASK_UINT16: U256 = U256::new(0xffffu128);
pub const MASK_UINT20: U256 = U256::new(0xfffffu128);
pub const MASK_UINT24: U256 = U256::new(0xffffffu128);
pub const MASK_UINT40: U256 = U256::new(0xffffffffffu128);
pub const MASK_UINT64: U256 = U256::new(0xffffffffffffffffu128);
pub const MASK_UINT128: U256 = U256::new(0xffffffffffffffffffffffffffffffffu128);

#[cw_serde]
#[derive(Copy, Default)]
pub struct EncodedSample(pub Bytes32);

impl EncodedSample {
    /// Internal function to set a value in an encoded bytes32 using a mask and offset
    pub fn set(&mut self, value: U256, mask: U256, offset: u8) -> &mut Self {
        let mask_shifted = mask << offset;
        let value_shifted = (value & mask) << offset;
        self.0 = ((U256::from_le_bytes(self.0) & !mask_shifted) | value_shifted).to_le_bytes();
        self
    }

    /// Internal function to set a bool in an encoded bytes32 using an offset
    pub fn set_bool(&mut self, boolean: bool, offset: u8) -> &mut Self {
        Self::set(self, U256::from(boolean as u8), MASK_UINT1, offset)
    }

    /// Internal function to decode a bytes32 sample using a mask and offset
    pub fn decode(&self, mask: U256, offset: u8) -> U256 {
        (U256::from_le_bytes(self.0) >> offset) & mask
    }

    /// Internal function to decode a bytes32 sample into a bool using an offset
    pub fn decode_bool(&self, offset: u8) -> bool {
        Self::decode(self, MASK_UINT1, offset).as_u64() != 0
    }

    /// Internal function to decode a bytes32 sample into a uint8 using an offset
    pub fn decode_uint8(&self, offset: u8) -> u8 {
        Self::decode(self, MASK_UINT8, offset).as_u8()
    }

    /// Internal function to decode a bytes32 sample into a uint12 using an offset
    /// The decoded value as a uint16, since uint12 is not supported
    pub fn decode_uint12(&self, offset: u8) -> u16 {
        Self::decode(self, MASK_UINT12, offset).as_u16()
    }

    /// Internal function to decode a bytes32 sample into a uint14 using an offset
    /// The decoded value as a uint16, since uint14 is not supported
    pub fn decode_uint14(&self, offset: u8) -> u16 {
        Self::decode(self, MASK_UINT14, offset).as_u16()
    }

    /// Internal function to decode a bytes32 sample into a uint16 using an offset
    pub fn decode_uint16(&self, offset: u8) -> u16 {
        Self::decode(self, MASK_UINT16, offset).as_u16()
    }

    /// Internal function to decode a bytes32 sample into a uint20 using an offset
    /// The decoded value as a uint32, since uint20 is not supported
    pub fn decode_uint20(&self, offset: u8) -> u32 {
        Self::decode(self, MASK_UINT20, offset).as_u32()
    }

    /// Internal function to decode a bytes32 sample into a uint24 using an offset
    /// The decoded value as a uint32, since uint24 is not supported
    pub fn decode_uint24(&self, offset: u8) -> u32 {
        Self::decode(self, MASK_UINT24, offset).as_u32()
    }

    /// Internal function to decode a bytes32 sample into a uint40 using an offset
    /// The decoded value as a uint64, since uint40 is not supported
    pub fn decode_uint40(&self, offset: u8) -> u64 {
        Self::decode(self, MASK_UINT40, offset).as_u64()
    }

    /// Internal function to decode a bytes32 sample into a uint64 using an offset
    pub fn decode_uint64(&self, offset: u8) -> u64 {
        Self::decode(self, MASK_UINT64, offset).as_u64()
    }

    /// Internal function to decode a bytes32 sample into a uint128 using an offset
    pub fn decode_uint128(&self, offset: u8) -> u128 {
        Self::decode(self, MASK_UINT128, offset).as_u128()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set() {
        struct TestCase {
            original_value: [u8; 32],
            value: U256,
            mask: U256,
            offset: u8,
            expected_value: U256,
        }

        let test_cases = vec![
            // Minimum values
            TestCase {
                original_value: Bytes32::default(),
                value: U256::new(0),
                mask: U256::new(0),
                offset: 0,
                expected_value: U256::MIN,
            },
            // Maximum values
            TestCase {
                original_value: [0xff; 32],
                value: U256::MAX,
                mask: U256::MAX,
                offset: 255,
                expected_value: U256::MAX,
            },
            // Custom test case
            TestCase {
                original_value: U256::from(
                    0x0000000000000000000000000000000000000000000000000000abcd12345678u64,
                )
                .to_le_bytes(),
                value: 0x0Fu64.into(),
                mask: 0xFFu64.into(),
                offset: 8u8,
                expected_value:
                    0x0000000000000000000000000000000000000000000000000000abcd12340f78u64.into(),
            },
        ];

        for test_case in test_cases {
            let encoded = *EncodedSample(test_case.original_value).set(
                test_case.value,
                test_case.mask,
                test_case.offset,
            );

            let mask_shifted = test_case.mask << test_case.offset;
            let value_shifted = (test_case.value & test_case.mask) << test_case.offset;
            let expected =
                (U256::from_le_bytes(test_case.original_value) & !mask_shifted) | value_shifted;

            assert_eq!(expected, test_case.expected_value, "test_Set::1");
            assert_eq!(encoded.0, expected.to_le_bytes(), "test_Set::2");
        }
    }

    #[test]
    fn test_set_bool() {
        let original_value =
            U256::from(0x0000000000000000000000000000000000000000000000000000abcd12345678u64)
                .to_le_bytes();

        let mut bytes_32 = EncodedSample(original_value);
        let boolean = false;
        let offset: u8 = 5;

        let result = bytes_32.set_bool(boolean, offset);

        let expected_result = EncodedSample(
            U256::from(0x0000000000000000000000000000000000000000000000000000abcd12345658u64)
                .to_le_bytes(),
        );

        assert_eq!(
            *result, expected_result,
            "The result of set_bool did not match the expected value."
        );
    }

    #[test]
    fn test_decode_bool() {
        struct TestCase {
            original_value: [u8; 32],
            offset: u8,
            expected_value: bool,
        }

        let test_cases = vec![
            // Example test case
            TestCase {
                original_value: U256::from(
                    0x0000000000000000000000000000000000000000000000000000000000000020u64,
                )
                .to_le_bytes(), // Example value with bit at offset 5 set to true
                offset: 8,
                expected_value: false,
            },
            TestCase {
                original_value: U256::from(
                    0x0000000000000000000000000000000000000000000000000000000000000020u64,
                )
                .to_le_bytes(), // Example value with bit at offset 5 set to true
                offset: 5,
                expected_value: true,
            },
        ];

        for test_case in test_cases {
            // Shift right by the offset and check the least significant bit
            let decoded_bool =
                ((U256::from_le_bytes(test_case.original_value) >> test_case.offset) & MASK_UINT1)
                    != U256::from(0u64);
            assert_eq!(
                decoded_bool, test_case.expected_value,
                "test_decode_bool::1"
            );

            let decoded_bool_fetched =
                EncodedSample(test_case.original_value).decode_bool(test_case.offset);

            assert_eq!(decoded_bool, decoded_bool_fetched, "test_decode_bool::2");
        }
    }

    #[test]
    fn test_decode_uint8() {
        struct TestCase {
            original_value: [u8; 32],
            offset: u8,
            expected_value: u8,
        }

        let test_cases = vec![
            // Example test case for minimum value
            TestCase {
                original_value: [0; 32],
                offset: 0,
                expected_value: 0,
            },
            // Example test case for maximum value
            TestCase {
                original_value: [0xff; 32],
                offset: 0,
                expected_value: 0xff,
            },
            // Failing test case
            TestCase {
                original_value: U256::from(
                    0x0000000000000000000000000000000000000000000000000000abcd12345678u64,
                )
                .to_le_bytes(),
                offset: 4,
                expected_value: 103, // 67 -> 0110 0111 -> 103
            },
        ];

        for test_case in test_cases {
            let decoded_u8 =
                (U256::from_le_bytes(test_case.original_value) >> test_case.offset) & MASK_UINT8;
            assert_eq!(
                decoded_u8.as_u8(),
                test_case.expected_value,
                "test_decode_u8::1"
            );
            // let decoded_u8_fetched = test_case.original_value.decode
            let decoded_u8_fetched =
                EncodedSample(test_case.original_value).decode_uint8(test_case.offset);
            assert_eq!(decoded_u8.as_u8(), decoded_u8_fetched, "test_decode_u8::2");
        }
    }

    #[test]
    fn test_decode_uint12() {
        struct TestCase {
            original_value: [u8; 32],
            offset: u8,
            expected_value: u16,
        }

        let test_cases = vec![
            TestCase {
                original_value: [0; 32],
                offset: 0,
                expected_value: 0,
            },
            TestCase {
                original_value: U256::from(
                    0x0000000000000000000000000000000000000000000000000000000000000fffu64,
                )
                .to_le_bytes(),
                offset: 0,
                expected_value: 0x0fff,
            },
            // Add additional test cases as needed
        ];

        for test_case in test_cases {
            let decoded_u12 =
                EncodedSample(test_case.original_value).decode_uint12(test_case.offset);
            assert_eq!(decoded_u12, test_case.expected_value, "test_decode_uint12");
        }
    }

    #[test]
    fn test_decode_uint14() {
        struct TestCase {
            original_value: [u8; 32],
            offset: u8,
            expected_value: u16,
        }

        let test_cases = vec![
            TestCase {
                original_value: [0; 32],
                offset: 0,
                expected_value: 0,
            },
            TestCase {
                original_value: U256::from(
                    0x0000000000000000000000000000000000000000000000000000000000003fffu64,
                )
                .to_le_bytes(),

                offset: 0,
                expected_value: 0x3fff,
            },
            // Add additional test cases as needed
        ];

        for test_case in test_cases {
            let decoded_u14 =
                EncodedSample(test_case.original_value).decode_uint14(test_case.offset);
            assert_eq!(decoded_u14, test_case.expected_value, "test_decode_uint14");
        }
    }

    #[test]
    fn test_decode_uint16() {
        struct TestCase {
            original_value: [u8; 32],
            offset: u8,
            expected_value: u16,
        }

        let test_cases = vec![
            // Minimum value
            TestCase {
                original_value: [0; 32],
                offset: 0,
                expected_value: 0,
            },
            // Maximum value
            TestCase {
                original_value: [0xff; 32],
                offset: 0,
                expected_value: 0xffff,
            },
            // Custom test case
            TestCase {
                original_value: U256::from(
                    0x0000000000000000000000000000000000000000000000000000abcd12345678u64,
                )
                .to_le_bytes(),
                offset: 4,
                expected_value: 0x4567, // Decoded from the least significant 16 bits
            },
        ];

        for test_case in test_cases {
            let decoded_u16 =
                (U256::from_le_bytes(test_case.original_value) >> test_case.offset) & MASK_UINT16;
            assert_eq!(
                decoded_u16.as_u16(),
                test_case.expected_value,
                "test_decode_u16::1"
            );
            let decoded_u16_fetched =
                EncodedSample(test_case.original_value).decode_uint16(test_case.offset);
            assert_eq!(
                decoded_u16.as_u16(),
                decoded_u16_fetched,
                "test_decode_u16::2"
            );
        }
    }

    #[test]
    fn test_decode_uint20() {
        struct TestCase {
            original_value: [u8; 32],
            offset: u8,
            expected_value: u32,
        }

        let test_cases = vec![
            TestCase {
                original_value: [0; 32],
                offset: 0,
                expected_value: 0,
            },
            TestCase {
                original_value: U256::from(
                    0x00000000000000000000000000000000000000000000000000000000000fffffu64,
                )
                .to_le_bytes(),

                offset: 0,
                expected_value: 0xfffff,
            },
            // Add additional test cases as needed
        ];

        for test_case in test_cases {
            let decoded_u20 =
                EncodedSample(test_case.original_value).decode_uint20(test_case.offset);
            assert_eq!(decoded_u20, test_case.expected_value, "test_decode_uint20");
        }
    }

    #[test]
    fn test_decode_uint24() {
        struct TestCase {
            original_value: [u8; 32],
            offset: u8,
            expected_value: u32,
        }

        let test_cases = vec![
            TestCase {
                original_value: [0; 32],
                offset: 0,
                expected_value: 0,
            },
            TestCase {
                original_value: U256::from(
                    0x0000000000000000000000000000000000000000000000000000000000ffffffu64,
                )
                .to_le_bytes(),
                offset: 0,
                expected_value: 0xffffff,
            },
            // Add additional test cases as needed
        ];

        for test_case in test_cases {
            let decoded_u24 =
                EncodedSample(test_case.original_value).decode_uint24(test_case.offset);
            assert_eq!(decoded_u24, test_case.expected_value, "test_decode_uint24");
        }
    }

    #[test]
    fn test_decode_uint40() {
        struct TestCase {
            original_value: [u8; 32],
            offset: u8,
            expected_value: u64,
        }

        let test_cases = vec![
            TestCase {
                original_value: [0; 32],
                offset: 0,
                expected_value: 0,
            },
            TestCase {
                original_value: U256::from(
                    0x000000000000000000000000000000000000000000000000000000ffffffffffu64,
                )
                .to_le_bytes(),
                offset: 0,
                expected_value: 0xffffffffff,
            },
            // Add additional test cases as needed
        ];

        for test_case in test_cases {
            let decoded_u40 =
                EncodedSample(test_case.original_value).decode_uint40(test_case.offset);
            assert_eq!(decoded_u40, test_case.expected_value, "test_decode_uint40");
        }
    }

    #[test]
    fn test_decode_uint64() {
        struct TestCase {
            original_value: [u8; 32],
            offset: u8,
            expected_value: u64,
        }

        let test_cases = vec![
            TestCase {
                original_value: [0; 32],
                offset: 0,
                expected_value: 0,
            },
            TestCase {
                original_value: U256::from(
                    0x000000000000000000000000000000000000000000000000ffffffffffffffffu64,
                )
                .to_le_bytes(),

                offset: 0,
                expected_value: 0xffffffffffffffff,
            },
            // Add additional test cases as needed
        ];

        for test_case in test_cases {
            let decoded_u64 =
                EncodedSample(test_case.original_value).decode_uint64(test_case.offset);
            assert_eq!(decoded_u64, test_case.expected_value, "test_decode_uint64");
        }
    }

    #[test]
    fn test_decode_uint128() {
        struct TestCase {
            original_value: [u8; 32],
            offset: u8,
            expected_value: u128,
        }

        let test_cases = vec![
            TestCase {
                original_value: [0; 32],
                offset: 0,
                expected_value: 0,
            },
            TestCase {
                original_value: [0xff; 32],
                offset: 0,
                expected_value: 0xffffffffffffffffffffffffffffffff,
            },
            // Add additional test cases as needed
        ];

        for test_case in test_cases {
            let decoded_u128 =
                EncodedSample(test_case.original_value).decode_uint128(test_case.offset);
            assert_eq!(
                decoded_u128, test_case.expected_value,
                "test_decode_uint128"
            );
        }
    }
}
