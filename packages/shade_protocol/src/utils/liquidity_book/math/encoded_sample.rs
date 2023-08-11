//! ### Liquidity Book Encoded Library
//! Author: Kent
//!
//! Helper library used for setting and decoding parts of encoded Bytes32.

use crate::utils::liquidity_book::types::Bytes32;
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
#[derive(Copy)]
pub struct EncodedSample(pub Bytes32);

impl EncodedSample {
    pub fn set(self, value: U256, mask: U256, offset: u8) -> Self {
        let mask_shifted = mask << offset;
        let value_shifted = (value & mask) << offset;
        // TODO: IDK if the from/to le_bytes is the appropriate method
        // Comment from Haseeb: This is the most efficient way in my opinion. All other methods are either slow or too complex.
        let new_encoded = (U256::from_le_bytes(self.0) & !mask_shifted) | value_shifted;

        Self(new_encoded.to_le_bytes())
    }

    pub fn set_bool(self, boolean: bool, offset: u8) -> Self {
        Self::set(self, U256::from(boolean as u8), MASK_UINT1, offset)
    }

    pub fn decode(&self, mask: U256, offset: u8) -> U256 {
        // TODO: IDK if the from_le_bytes is the appropriate method
        // Comment from Haseeb: This is the most efficient way in my opinion. All other methods are either slow or too complex.
        let value = (U256::from_le_bytes(self.0) >> offset) & mask;
        value
    }

    pub fn decode_bool(&self, offset: u8) -> bool {
        Self::decode(self, MASK_UINT1, offset).as_u64() != 0
    }

    pub fn decode_uint8(&self, offset: u8) -> u8 {
        Self::decode(self, MASK_UINT8, offset).as_u8()
    }

    // TODO: make a uint12 type
    pub fn decode_uint12(&self, offset: u8) -> u16 {
        Self::decode(self, MASK_UINT12, offset).as_u16()
    }
    // TODO: make a uint14 type
    pub fn decode_uint14(&self, offset: u8) -> u16 {
        Self::decode(self, MASK_UINT14, offset).as_u16()
    }
    // TODO: make a uint16 type
    pub fn decode_uint16(&self, offset: u8) -> u16 {
        Self::decode(self, MASK_UINT16, offset).as_u16()
    }
    // TODO: make a uint20 type
    pub fn decode_uint20(&self, offset: u8) -> u32 {
        Self::decode(self, MASK_UINT20, offset).as_u32()
    }
    // TODO: make a uint24 type
    pub fn decode_uint24(&self, offset: u8) -> u32 {
        Self::decode(self, MASK_UINT24, offset).as_u32()
    }
    // TODO: make a uint40 type
    pub fn decode_uint40(&self, offset: u8) -> u64 {
        Self::decode(self, MASK_UINT40, offset).as_u64()
    }

    pub fn decode_uint64(&self, offset: u8) -> u64 {
        Self::decode(self, MASK_UINT64, offset).as_u64()
    }

    pub fn decode_uint128(&self, offset: u8) -> u128 {
        Self::decode(self, MASK_UINT128, offset).as_u128()
    }
}
