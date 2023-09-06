//! ### Liquidity Book Constants Library
//! Author: Kent
//!
//! Set of constants for Liquidity Book contracts.

use ethnum::U256;
// use cosmwasm_std::Uint256;

pub static SCALE_OFFSET: u8 = 128;

// use this one for ethnum U256:
pub static SCALE: U256 = U256::from_words(1, 0);

pub static PRECISION: u128 = 1_000_000_000_000_000_000;
pub static SQUARED_PRECISION: u128 = PRECISION * PRECISION;

pub static MAX_FEE: u128 = 1_00_000_000_000_000_000; // 10%
pub static MAX_PROTOCOL_SHARE: u32 = 2_500; // 25% of the fee

pub static BASIS_POINT_MAX: u32 = 10_000;
